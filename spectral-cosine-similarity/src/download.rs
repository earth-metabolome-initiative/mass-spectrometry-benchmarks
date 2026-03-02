use md5::Md5;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use crate::progress::StageProgress;

pub const DATASET_PATH: &str = "fixtures/ALL_GNPS_cleaned.mgf";
pub const DATASET_FILENAME: &str = "ALL_GNPS_cleaned.mgf";

const DATASET_PART_PATH: &str = "fixtures/ALL_GNPS_cleaned.mgf.part";
const BYTES_PER_MIB: f64 = 1_048_576.0;
const DOWNLOAD_BAR_WIDTH: usize = 24;
const DOWNLOAD_USER_AGENT: &str = "spectral-cosine-similarity/0.1 (+https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks)";
const MAX_DOWNLOAD_ATTEMPTS_PER_URL: usize = 3;

const ZENODO_API_URL: &str =
    "https://zenodo.org/api/records/11193898/files/ALL_GNPS_cleaned.mgf/content";
const ZENODO_DIRECT_URL: &str =
    "https://zenodo.org/records/11193898/files/ALL_GNPS_cleaned.mgf?download=1";

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DigestKind {
    Sha256,
    Md5,
}

impl DigestKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Md5 => "md5",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DownloadSource {
    name: &'static str,
    urls: &'static [&'static str],
    expected_digest: &'static str,
    digest_kind: DigestKind,
}

const PINNED_SOURCE: DownloadSource = DownloadSource {
    name: "Zenodo record 11193898 / ALL_GNPS_cleaned.mgf",
    urls: &[ZENODO_API_URL, ZENODO_DIRECT_URL],
    expected_digest: "3382b7ec8843532256481820bb6e6c0c",
    digest_kind: DigestKind::Md5,
};

fn sha256_hex(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 256 * 1024];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

fn md5_hex(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = [0u8; 256 * 1024];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

fn digest_hex(path: &Path, digest_kind: DigestKind) -> std::io::Result<String> {
    match digest_kind {
        DigestKind::Sha256 => sha256_hex(path),
        DigestKind::Md5 => md5_hex(path),
    }
}

fn has_expected_digest(path: &Path, source: &DownloadSource) -> std::io::Result<(bool, String)> {
    let actual = digest_hex(path, source.digest_kind)?;
    Ok((actual == source.expected_digest, actual))
}

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(p) = progress.as_deref_mut() {
        p.set_substep(message);
    } else {
        eprintln!("{message}");
    }
}

fn format_download_progress(downloaded: u64, total_bytes: Option<u64>) -> String {
    match total_bytes {
        Some(total) if total > 0 => {
            let ratio = (downloaded as f64 / total as f64).clamp(0.0, 1.0);
            let filled =
                ((ratio * DOWNLOAD_BAR_WIDTH as f64).round() as usize).min(DOWNLOAD_BAR_WIDTH);
            let bar = format!(
                "[{}{}]",
                "=".repeat(filled),
                "-".repeat(DOWNLOAD_BAR_WIDTH - filled)
            );
            format!(
                "[download] {bar} {:>5.1}% ({:.1}/{:.1} MB)",
                ratio * 100.0,
                downloaded as f64 / BYTES_PER_MIB,
                total as f64 / BYTES_PER_MIB
            )
        }
        _ => format!(
            "[download] Downloaded {:.1} MB",
            downloaded as f64 / BYTES_PER_MIB
        ),
    }
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::FORBIDDEN
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn download_with_fallback_urls(
    client: &reqwest::blocking::Client,
    source: &DownloadSource,
    progress: &mut Option<&mut dyn StageProgress>,
) -> (reqwest::blocking::Response, &'static str) {
    let mut failures: Vec<String> = Vec::new();

    for (url_index, &url) in source.urls.iter().enumerate() {
        if url_index > 0 {
            emit(
                progress,
                &format!("[download] Falling back to alternate URL: {url}"),
            );
        }

        for attempt in 1..=MAX_DOWNLOAD_ATTEMPTS_PER_URL {
            if attempt > 1 {
                emit(
                    progress,
                    &format!(
                        "[download] Retry {attempt}/{MAX_DOWNLOAD_ATTEMPTS_PER_URL} for {url}"
                    ),
                );
            }

            match client.get(url).send() {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return (response, url);
                    }

                    failures.push(format!("URL {url} attempt {attempt}: HTTP {status}"));

                    if should_retry_status(status) && attempt < MAX_DOWNLOAD_ATTEMPTS_PER_URL {
                        thread::sleep(Duration::from_secs(attempt as u64));
                        continue;
                    }
                    break;
                }
                Err(err) => {
                    failures.push(format!("URL {url} attempt {attempt}: {err}"));
                    if attempt < MAX_DOWNLOAD_ATTEMPTS_PER_URL {
                        thread::sleep(Duration::from_secs(attempt as u64));
                        continue;
                    }
                    break;
                }
            }
        }
    }

    let details = failures.join("\n  - ");
    panic!(
        "failed to download {} from all candidate URLs.\n  - {}",
        source.name, details
    );
}

/// Download the pinned benchmark MGF file with an atomic `.part` file and digest verification.
pub fn run(allow_unverified_download: bool) {
    run_with_progress(allow_unverified_download, None);
}

/// Download the pinned benchmark MGF file with optional progress updates.
pub fn run_with_progress(
    allow_unverified_download: bool,
    mut progress: Option<&mut dyn StageProgress>,
) {
    let source = PINNED_SOURCE;
    let final_path = Path::new(DATASET_PATH);
    let part_path = Path::new(DATASET_PART_PATH);
    let digest_kind = source.digest_kind.as_str();

    if part_path.exists() {
        emit(
            &mut progress,
            &format!("[download] Removing stale partial file {DATASET_PART_PATH}"),
        );
        std::fs::remove_file(part_path).unwrap_or_else(|e| {
            panic!(
                "failed to remove stale partial file {}: {e}",
                DATASET_PART_PATH
            )
        });
    }

    if final_path.exists() {
        emit(
            &mut progress,
            &format!("[download] Checking {digest_kind} digest for existing {DATASET_PATH}"),
        );
        match has_expected_digest(final_path, &source) {
            Ok((true, actual_digest)) => {
                emit(
                    &mut progress,
                    &format!(
                        "[download] {} already verified ({}={}), skipping",
                        DATASET_PATH, digest_kind, actual_digest
                    ),
                );
                return;
            }
            Ok((false, actual_digest)) => {
                emit(
                    &mut progress,
                    &format!(
                        "[download] Existing {} failed {} verification (expected {}, actual {}); redownloading",
                        DATASET_PATH, digest_kind, source.expected_digest, actual_digest
                    ),
                );
                std::fs::remove_file(final_path).unwrap_or_else(|e| {
                    panic!("failed to remove invalid existing {}: {e}", DATASET_PATH)
                });
            }
            Err(e) => {
                if allow_unverified_download {
                    eprintln!(
                        "[download] WARNING: failed to verify existing {} {} digest ({e}); proceeding without verification",
                        DATASET_PATH, digest_kind
                    );
                    return;
                }
                panic!(
                    "failed to verify existing {} {} digest: {e}",
                    DATASET_PATH, digest_kind
                );
            }
        }
    }

    emit(
        &mut progress,
        &format!(
            "[download] Downloading {} to {} (primary URL: {})",
            source.name, DATASET_PART_PATH, source.urls[0]
        ),
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(DOWNLOAD_USER_AGENT)
        .timeout(Duration::from_secs(600))
        .build()
        .expect("failed to build HTTP client");

    let (mut response, downloaded_from_url) =
        download_with_fallback_urls(&client, &source, &mut progress);

    let mut part_file = File::create(part_path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", DATASET_PART_PATH));

    let total_bytes = response.content_length();
    let mut total: u64 = 0;
    let mut buf = vec![0u8; 256 * 1024];
    let mut last_reported_total = 0u64;
    let mut last_reported_at = Instant::now();
    emit(&mut progress, &format_download_progress(0, total_bytes));
    loop {
        let n = response
            .read(&mut buf)
            .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
        if n == 0 {
            break;
        }
        part_file
            .write_all(&buf[..n])
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", DATASET_PART_PATH));
        total += n as u64;
        if total.saturating_sub(last_reported_total) >= 4 * 1024 * 1024
            || last_reported_at.elapsed() >= Duration::from_millis(250)
        {
            emit(&mut progress, &format_download_progress(total, total_bytes));
            last_reported_total = total;
            last_reported_at = Instant::now();
        }
    }
    emit(&mut progress, &format_download_progress(total, total_bytes));

    part_file
        .sync_all()
        .unwrap_or_else(|e| panic!("failed to sync {}: {e}", DATASET_PART_PATH));

    emit(
        &mut progress,
        &format!("[download] Verifying {digest_kind} digest"),
    );
    let (verified, actual_digest) = has_expected_digest(part_path, &source).unwrap_or_else(|e| {
        panic!(
            "failed to verify {} digest for {}: {e}",
            digest_kind, DATASET_PART_PATH
        )
    });
    if !verified && !allow_unverified_download {
        let _ = std::fs::remove_file(part_path);
        panic!(
            "downloaded file digest mismatch for {}; source: {}\nexpected {}: {}\nactual {}: {}\nrefusing to continue in strict mode (use --allow-unverified-download to bypass verification)",
            DATASET_PART_PATH,
            downloaded_from_url,
            digest_kind,
            source.expected_digest,
            digest_kind,
            actual_digest
        );
    }

    if !verified {
        eprintln!(
            "[download] WARNING: digest mismatch for {} (expected {}: {}, actual {}: {}), keeping file because --allow-unverified-download was set",
            DATASET_PART_PATH, digest_kind, source.expected_digest, digest_kind, actual_digest
        );
    } else {
        eprintln!(
            "[download] Digest verified for {} ({}: {}, source: {})",
            DATASET_PART_PATH, digest_kind, actual_digest, downloaded_from_url
        );
    }

    std::fs::rename(part_path, final_path).unwrap_or_else(|e| {
        panic!(
            "failed to promote {} -> {}: {e}",
            DATASET_PART_PATH, DATASET_PATH
        )
    });

    emit(
        &mut progress,
        &format!(
            "[download] Saved {} ({:.1} MB)",
            DATASET_PATH,
            total as f64 / BYTES_PER_MIB
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::{DigestKind, DownloadSource, has_expected_digest, md5_hex, sha256_hex};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{nanos}.tmp"))
    }

    #[test]
    fn sha256_hex_matches_known_value() {
        let path = temp_path("sha256-test");
        let mut file = std::fs::File::create(&path).expect("create temp file");
        file.write_all(b"abc").expect("write temp file");
        file.sync_all().expect("sync temp file");

        let digest = sha256_hex(&path).expect("hash file");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        std::fs::remove_file(&path).expect("remove temp file");
    }

    #[test]
    fn md5_hex_matches_known_value() {
        let path = temp_path("md5-test");
        let mut file = std::fs::File::create(&path).expect("create temp file");
        file.write_all(b"abc").expect("write temp file");
        file.sync_all().expect("sync temp file");

        let digest = md5_hex(&path).expect("hash file");
        assert_eq!(digest, "900150983cd24fb0d6963f7d28e17f72");

        std::fs::remove_file(&path).expect("remove temp file");
    }

    #[test]
    fn has_expected_digest_reports_match_and_mismatch() {
        let path = temp_path("digest-dispatch-test");
        let mut file = std::fs::File::create(&path).expect("create temp file");
        file.write_all(b"abc").expect("write temp file");
        file.sync_all().expect("sync temp file");

        let matching_md5_source = DownloadSource {
            name: "test",
            urls: &["https://example.test"],
            expected_digest: "900150983cd24fb0d6963f7d28e17f72",
            digest_kind: DigestKind::Md5,
        };
        let (md5_match, md5_actual) =
            has_expected_digest(&path, &matching_md5_source).expect("hash file");
        assert!(md5_match);
        assert_eq!(md5_actual, matching_md5_source.expected_digest);

        let mismatching_sha_source = DownloadSource {
            name: "test",
            urls: &["https://example.test"],
            expected_digest: "deadbeef",
            digest_kind: DigestKind::Sha256,
        };
        let (sha_match, sha_actual) =
            has_expected_digest(&path, &mismatching_sha_source).expect("hash file");
        assert!(!sha_match);
        assert_eq!(
            sha_actual,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        std::fs::remove_file(&path).expect("remove temp file");
    }
}
