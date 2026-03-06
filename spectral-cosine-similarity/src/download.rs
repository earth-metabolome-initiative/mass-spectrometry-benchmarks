use md5::Md5;
use reqwest::StatusCode;
use sha2::Digest;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

pub const DATASET_PATH: &str = "fixtures/ALL_GNPS_cleaned.mgf";
pub const DATASET_FILENAME: &str = "ALL_GNPS_cleaned.mgf";

const DATASET_PART_PATH: &str = "fixtures/ALL_GNPS_cleaned.mgf.part";
const BYTES_PER_MIB: f64 = 1_048_576.0;
const DOWNLOAD_USER_AGENT: &str = "spectral-cosine-similarity/0.1 (+https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks)";
const MAX_DOWNLOAD_ATTEMPTS: usize = 3;
const EXPECTED_MD5: &str = "3382b7ec8843532256481820bb6e6c0c";

const ZENODO_API_URL: &str =
    "https://zenodo.org/api/records/11193898/files/ALL_GNPS_cleaned.mgf/content";
const ZENODO_DIRECT_URL: &str =
    "https://zenodo.org/records/11193898/files/ALL_GNPS_cleaned.mgf?download=1";

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

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::FORBIDDEN
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn try_download(
    client: &reqwest::blocking::Client,
    urls: &[&'static str],
) -> (reqwest::blocking::Response, &'static str) {
    let mut failures: Vec<String> = Vec::new();

    for (url_index, &url) in urls.iter().enumerate() {
        if url_index > 0 {
            eprintln!("[download] Falling back to alternate URL: {url}");
        }

        for attempt in 1..=MAX_DOWNLOAD_ATTEMPTS {
            if attempt > 1 {
                eprintln!("[download] Retry {attempt}/{MAX_DOWNLOAD_ATTEMPTS} for {url}");
            }

            match client.get(url).send() {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return (response, url);
                    }

                    failures.push(format!("URL {url} attempt {attempt}: HTTP {status}"));

                    if should_retry_status(status) && attempt < MAX_DOWNLOAD_ATTEMPTS {
                        thread::sleep(Duration::from_secs(attempt as u64));
                        continue;
                    }
                    break;
                }
                Err(err) => {
                    failures.push(format!("URL {url} attempt {attempt}: {err}"));
                    if attempt < MAX_DOWNLOAD_ATTEMPTS {
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
        "failed to download from all candidate URLs.\n  - {}",
        details
    );
}

pub fn run() {
    let final_path = Path::new(DATASET_PATH);
    let part_path = Path::new(DATASET_PART_PATH);

    if part_path.exists() {
        eprintln!("[download] Removing stale partial file {DATASET_PART_PATH}");
        std::fs::remove_file(part_path).unwrap_or_else(|e| {
            panic!(
                "failed to remove stale partial file {}: {e}",
                DATASET_PART_PATH
            )
        });
    }

    if final_path.exists() {
        eprintln!("[download] Checking md5 digest for existing {DATASET_PATH}");
        match md5_hex(final_path) {
            Ok(actual) if actual == EXPECTED_MD5 => {
                eprintln!(
                    "[download] {} already verified (md5={actual}), skipping",
                    DATASET_PATH
                );
                return;
            }
            Ok(actual) => {
                eprintln!(
                    "[download] Existing {} failed md5 verification (expected {EXPECTED_MD5}, actual {actual}); redownloading",
                    DATASET_PATH
                );
                std::fs::remove_file(final_path).unwrap_or_else(|e| {
                    panic!("failed to remove invalid existing {}: {e}", DATASET_PATH)
                });
            }
            Err(e) => {
                panic!("failed to verify existing {} md5 digest: {e}", DATASET_PATH);
            }
        }
    }

    eprintln!(
        "[download] Downloading to {} (primary URL: {})",
        DATASET_PART_PATH, ZENODO_API_URL
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(DOWNLOAD_USER_AGENT)
        .timeout(Duration::from_secs(600))
        .build()
        .expect("failed to build HTTP client");

    let urls: &[&str] = &[ZENODO_API_URL, ZENODO_DIRECT_URL];
    let (mut response, downloaded_from_url) = try_download(&client, urls);

    let mut part_file = File::create(part_path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", DATASET_PART_PATH));

    let total_bytes = response.content_length();
    let mut total: u64 = 0;
    let mut buf = vec![0u8; 256 * 1024];
    let mut last_reported_at = Instant::now();
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
        if last_reported_at.elapsed() >= Duration::from_millis(500) {
            let msg = match total_bytes {
                Some(tb) if tb > 0 => format!(
                    "[download] {:.1}/{:.1} MB ({:.0}%)",
                    total as f64 / BYTES_PER_MIB,
                    tb as f64 / BYTES_PER_MIB,
                    total as f64 / tb as f64 * 100.0
                ),
                _ => format!("[download] {:.1} MB", total as f64 / BYTES_PER_MIB),
            };
            eprintln!("{msg}");
            last_reported_at = Instant::now();
        }
    }

    part_file
        .sync_all()
        .unwrap_or_else(|e| panic!("failed to sync {}: {e}", DATASET_PART_PATH));

    eprintln!("[download] Verifying md5 digest");
    let actual_digest = md5_hex(part_path)
        .unwrap_or_else(|e| panic!("failed to verify md5 for {}: {e}", DATASET_PART_PATH));
    if actual_digest != EXPECTED_MD5 {
        let _ = std::fs::remove_file(part_path);
        panic!(
            "downloaded file digest mismatch for {}; source: {}\nexpected md5: {}\nactual md5: {}",
            DATASET_PART_PATH, downloaded_from_url, EXPECTED_MD5, actual_digest
        );
    }
    eprintln!(
        "[download] Digest verified for {} (md5: {actual_digest}, source: {downloaded_from_url})",
        DATASET_PART_PATH
    );

    std::fs::rename(part_path, final_path).unwrap_or_else(|e| {
        panic!(
            "failed to promote {} -> {}: {e}",
            DATASET_PART_PATH, DATASET_PATH
        )
    });

    eprintln!(
        "[download] Saved {} ({:.1} MB)",
        DATASET_PATH,
        total as f64 / BYTES_PER_MIB
    );
}

#[cfg(test)]
mod tests {
    use super::md5_hex;
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
    fn md5_hex_matches_known_value() {
        let path = temp_path("md5-test");
        let mut file = std::fs::File::create(&path).expect("create temp file");
        file.write_all(b"abc").expect("write temp file");
        file.sync_all().expect("sync temp file");

        let digest = md5_hex(&path).expect("hash file");
        assert_eq!(digest, "900150983cd24fb0d6963f7d28e17f72");

        std::fs::remove_file(&path).expect("remove temp file");
    }
}
