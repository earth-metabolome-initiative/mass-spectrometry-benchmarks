use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use crate::progress::StageProgress;

const GNPS_URL: &str = "https://external.gnps2.org/gnpslibrary/GNPS-LIBRARY.mgf";
const MGF_PATH: &str = "fixtures/GNPS-LIBRARY.mgf";
const MGF_PART_PATH: &str = "fixtures/GNPS-LIBRARY.mgf.part";
const GNPS_SHA256: &str = "6979883b305c7d7bb1fc9c96ca8ffa21f9fb8f131444a4ac57759f73cbe46c4c";

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

fn has_expected_checksum(path: &Path) -> std::io::Result<bool> {
    Ok(sha256_hex(path)? == GNPS_SHA256)
}

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(p) = progress.as_deref_mut() {
        p.set_substep(message);
    } else {
        eprintln!("{message}");
    }
}

/// Download GNPS-LIBRARY.mgf with an atomic `.part` file and checksum verification.
pub fn run(allow_unverified_download: bool) {
    run_with_progress(allow_unverified_download, None);
}

/// Download GNPS-LIBRARY.mgf with optional progress updates.
pub fn run_with_progress(
    allow_unverified_download: bool,
    mut progress: Option<&mut dyn StageProgress>,
) {
    let final_path = Path::new(MGF_PATH);
    let part_path = Path::new(MGF_PART_PATH);

    if part_path.exists() {
        emit(
            &mut progress,
            &format!("[download] Removing stale partial file {MGF_PART_PATH}"),
        );
        std::fs::remove_file(part_path).unwrap_or_else(|e| {
            panic!("failed to remove stale partial file {}: {e}", MGF_PART_PATH)
        });
    }

    if final_path.exists() {
        emit(
            &mut progress,
            &format!("[download] Checking checksum for existing {MGF_PATH}"),
        );
        match has_expected_checksum(final_path) {
            Ok(true) => {
                emit(
                    &mut progress,
                    &format!("[download] {MGF_PATH} already verified, skipping"),
                );
                return;
            }
            Ok(false) => {
                emit(
                    &mut progress,
                    &format!("[download] Existing {MGF_PATH} failed checksum; redownloading"),
                );
                std::fs::remove_file(final_path).unwrap_or_else(|e| {
                    panic!("failed to remove invalid existing {}: {e}", MGF_PATH)
                });
            }
            Err(e) => {
                if allow_unverified_download {
                    eprintln!(
                        "[download] WARNING: failed to verify existing {} checksum ({e}); proceeding without verification",
                        MGF_PATH
                    );
                    return;
                }
                panic!("failed to verify existing {} checksum: {e}", MGF_PATH);
            }
        }
    }

    emit(
        &mut progress,
        &format!("[download] Downloading GNPS-LIBRARY.mgf to {MGF_PART_PATH}"),
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .expect("failed to build HTTP client");

    let mut response = client
        .get(GNPS_URL)
        .send()
        .unwrap_or_else(|e| panic!("failed to download GNPS-LIBRARY: {e}"));

    if !response.status().is_success() {
        panic!(
            "failed to download GNPS-LIBRARY: HTTP {}",
            response.status()
        );
    }

    let mut part_file = File::create(part_path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", MGF_PART_PATH));

    let mut total: u64 = 0;
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = response
            .read(&mut buf)
            .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
        if n == 0 {
            break;
        }
        part_file
            .write_all(&buf[..n])
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", MGF_PART_PATH));
        total += n as u64;
    }

    part_file
        .sync_all()
        .unwrap_or_else(|e| panic!("failed to sync {}: {e}", MGF_PART_PATH));

    emit(&mut progress, "[download] Verifying checksum");
    let verified = has_expected_checksum(part_path)
        .unwrap_or_else(|e| panic!("failed to verify checksum for {}: {e}", MGF_PART_PATH));
    if !verified && !allow_unverified_download {
        let _ = std::fs::remove_file(part_path);
        panic!(
            "downloaded file checksum mismatch for {}; refusing to continue in strict mode",
            MGF_PART_PATH
        );
    }

    if !verified {
        eprintln!(
            "[download] WARNING: checksum mismatch for {}, keeping file because --allow-unverified-download was set",
            MGF_PART_PATH
        );
    } else {
        eprintln!("[download] Checksum verified for {}", MGF_PART_PATH);
    }

    std::fs::rename(part_path, final_path)
        .unwrap_or_else(|e| panic!("failed to promote {} -> {}: {e}", MGF_PART_PATH, MGF_PATH));

    emit(
        &mut progress,
        &format!(
            "[download] Saved {MGF_PATH} ({:.1} MB)",
            total as f64 / 1_048_576.0
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;
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
}
