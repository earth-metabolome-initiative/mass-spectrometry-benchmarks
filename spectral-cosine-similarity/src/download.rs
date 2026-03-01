use std::io::Write;
use std::path::Path;
use std::time::Duration;

const GNPS_URL: &str = "https://external.gnps2.org/gnpslibrary/GNPS-LIBRARY.mgf";
const MGF_PATH: &str = "fixtures/GNPS-LIBRARY.mgf";

/// Download GNPS-LIBRARY.mgf if not already present.
pub fn run() {
    let path = Path::new(MGF_PATH);
    if path.exists() {
        eprintln!("[download] {} already present, skipping", MGF_PATH);
        return;
    }

    eprintln!("[download] Downloading GNPS-LIBRARY.mgf...");

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

    let mut file = std::fs::File::create(path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", MGF_PATH));

    let mut total: u64 = 0;
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = std::io::Read::read(&mut response, &mut buf)
            .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", MGF_PATH));
        total += n as u64;
    }

    eprintln!(
        "[download] Saved {} ({:.1} MB)",
        MGF_PATH,
        total as f64 / 1_048_576.0
    );
}
