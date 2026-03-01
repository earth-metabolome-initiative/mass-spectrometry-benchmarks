#![cfg(feature = "python-tests")]

use diesel::dsl::count_star;
use diesel::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

use spectral_cosine_similarity::schema::{experiments, implementations, results, spectra};
use spectral_cosine_similarity::{compute, report};

mod common;

#[test]
fn tiny_full_pipeline_produces_expected_rows_and_artifacts() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 3);
    let uv_cache_dir = TempDir::new().expect("failed to create temporary uv cache directory");
    let uv_cache = uv_cache_dir.path().to_path_buf();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let db_path = test_db.db_path.to_string_lossy().to_string();
    let uv_cache_for_first_run = uv_cache.clone();
    compute::run_with_matchms(&mut test_db.conn, Some(3), move |_| {
        let status = Command::new("uv")
            .current_dir(&manifest_dir)
            .env("UV_CACHE_DIR", &uv_cache_for_first_run)
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

    let output_dir = TempDir::new().expect("failed to create temporary output directory");
    report::run_to_dir(&mut test_db.conn, output_dir.path());

    assert!(output_dir.path().join("timing_by_peaks.svg").exists());
    assert!(output_dir.path().join("mse_score_by_peaks.svg").exists());
    assert!(output_dir.path().join("tables_by_peaks.md").exists());

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

    assert_eq!(spectra_count, 3);
    assert_eq!(experiments_count, 4);
    assert_eq!(implementations_count, 9);
    let n_pairs = spectra_count * (spectra_count + 1) / 2;
    let expected_results = n_pairs * experiments_count * implementations_count;
    assert_eq!(results_count, expected_results);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let db_path = test_db.db_path.to_string_lossy().to_string();
    let uv_cache_for_second_run = uv_cache.clone();
    compute::run_with_matchms(&mut test_db.conn, Some(3), move |_| {
        let status = Command::new("uv")
            .current_dir(&manifest_dir)
            .env("UV_CACHE_DIR", &uv_cache_for_second_run)
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

    let results_count_after_rerun = results::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count results after rerun");
    assert_eq!(results_count_after_rerun, expected_results);
}
