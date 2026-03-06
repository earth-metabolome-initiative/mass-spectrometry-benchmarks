#![cfg(feature = "python-tests")]

use diesel::dsl::count_star;
use diesel::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

use spectral_cosine_similarity::schema::{experiments, implementations, results, spectra};
use spectral_cosine_similarity::{compute, db, report};

mod common;

#[test]
fn tiny_full_pipeline_produces_expected_rows_and_artifacts() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 3);
    let uv_cache_dir = TempDir::new().expect("failed to create temporary uv cache directory");
    let uv_cache = uv_cache_dir.path().to_path_buf();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let db_path = test_db.db_path.to_string_lossy().to_string();
    compute::run_with_python_runner(&mut test_db.conn, 3, 6, move || {
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

    let output_dir = TempDir::new().expect("failed to create temporary output directory");
    let report_config = report::ReportConfig {
        output_dir: output_dir.path().to_path_buf(),
        ..report::ReportConfig::default()
    };
    report::generate(&mut test_db.conn, &report_config);

    assert!(output_dir.path().join("timing.svg").exists());
    assert!(output_dir.path().join("rmse.svg").exists());
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

    assert_eq!(spectra_count, 3);
    assert_eq!(experiments_count, 1);
    assert_eq!(implementations_count, 15);
    let n_pairs = spectra_count * (spectra_count + 1) / 2;
    let expected_results = n_pairs * experiments_count * implementations_count;
    assert_eq!(results_count, expected_results);
}

#[test]
fn compute_honors_max_spectra_when_db_contains_more_rows() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 5);

    compute::run_with_python_runner(&mut test_db.conn, 3, 6, || {});

    let experiments_count = experiments::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count experiments");
    let limited_spectra: i64 = 3;
    let n_pairs = limited_spectra * (limited_spectra + 1) / 2;
    let expected_rust_rows = n_pairs * experiments_count;

    for algorithm_name in [
        "CosineHungarian",
        "CosineGreedy",
        "LinearCosine",
        "ModifiedCosine",
        "ModifiedGreedyCosine",
        "ModifiedLinearCosine",
        "CosineHungarianMerged",
        "ModifiedCosineMerged",
        "EntropySimilarityWeighted",
        "EntropySimilarityUnweighted",
    ] {
        let rust_impl_id = db::get_implementation_id(
            &mut test_db.conn,
            algorithm_name,
            "mass-spectrometry-traits",
        );
        let rows = results::table
            .filter(results::implementation_id.eq(rust_impl_id))
            .select(count_star())
            .first::<i64>(&mut test_db.conn)
            .expect("failed to count rust results for limited run");
        assert_eq!(rows, expected_rust_rows);
    }
}
