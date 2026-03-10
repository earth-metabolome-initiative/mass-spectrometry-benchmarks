#![cfg(feature = "python-tests")]

use diesel::dsl::count_star;
use diesel::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

use spectral_cosine_similarity::schema::{
    experiments, implementations, results, spectra, tanimoto_results,
};
use spectral_cosine_similarity::{compute, fingerprint, report};

mod common;

#[test]
fn tiny_full_pipeline_produces_expected_rows_and_artifacts() {
    let requested_pairs: i64 = 6;
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn);
    let uv_cache_dir = TempDir::new().expect("failed to create temporary uv cache directory");
    let uv_cache = uv_cache_dir.path().to_path_buf();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let db_path = test_db.db_path.to_string_lossy().to_string();

    compute::run_with_python_runner(&mut test_db.conn, requested_pairs as usize, move || {
        let status = Command::new("uv")
            .current_dir(&manifest_dir)
            .env("UV_CACHE_DIR", &uv_cache)
            .args([
                "run",
                "python3",
                "scripts/python_reference_compute.py",
                &db_path,
            ])
            .status()
            .expect("failed to run python reference compute script");
        assert!(
            status.success(),
            "python reference compute script failed: {status}"
        );
    });

    fingerprint::run(&mut test_db.conn);

    let output_dir = TempDir::new().expect("failed to create temporary output directory");
    let report_config = report::ReportConfig {
        output_dir: output_dir.path().to_path_buf(),
        ..report::ReportConfig::default()
    };
    report::generate(&mut test_db.conn, &report_config);

    assert!(output_dir.path().join("timing.svg").exists());
    assert!(output_dir.path().join("rmse.svg").exists());
    assert!(output_dir.path().join("correlation.svg").exists());
    assert!(output_dir.path().join("tables.md").exists());

    let spectra_count = spectra::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count spectra");
    let experiments_count = experiments::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count experiments");
    let implementations_count = implementations::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count implementations");
    let results_count = results::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count results");
    let tanimoto_count = tanimoto_results::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count tanimoto results");

    assert_eq!(spectra_count, 5);
    assert_eq!(experiments_count, 1);
    assert_eq!(implementations_count, 16);
    let expected_results = requested_pairs * experiments_count * implementations_count;
    assert_eq!(results_count, expected_results);
    assert_eq!(
        tanimoto_count,
        requested_pairs * 6,
        "tanimoto_results should have 6 rows per pair (one per fingerprint algorithm)"
    );

    let markdown =
        std::fs::read_to_string(output_dir.path().join("tables.md")).expect("read tables.md");
    assert!(
        markdown.contains("Correlation"),
        "markdown should contain correlation section"
    );
}
