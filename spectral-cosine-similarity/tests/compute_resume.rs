use diesel::dsl::count_star;
use diesel::prelude::*;

use spectral_cosine_similarity::schema::{experiments, implementations, results, spectra};
use spectral_cosine_similarity::{compute, db};

mod common;

#[test]
fn compute_rust_results_resume_without_duplicates() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 3);

    compute::run_with_matchms(&mut test_db.conn, Some(3), |_| {});

    let spectra_count = spectra::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count spectra");
    let experiments_count = experiments::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count experiments");
    let n_pairs = spectra_count * (spectra_count + 1) / 2;
    let expected_rust_rows = n_pairs * experiments_count;

    for algorithm_name in [
        "CosineHungarian",
        "CosineGreedy",
        "ModifiedCosine",
        "ModifiedGreedyCosine",
        "EntropySimilarityWeighted",
        "EntropySimilarityUnweighted",
    ] {
        let rust_impl_id = db::get_implementation_id(
            &mut test_db.conn,
            algorithm_name,
            "mass-spectrometry-traits",
        );
        let count_after_first_run = results::table
            .filter(results::implementation_id.eq(rust_impl_id))
            .select(count_star())
            .first::<i64>(&mut test_db.conn)
            .expect("failed to count rust results after first compute run");
        assert_eq!(count_after_first_run, expected_rust_rows);
    }

    compute::run_with_matchms(&mut test_db.conn, Some(3), |_| {});

    for algorithm_name in [
        "CosineHungarian",
        "CosineGreedy",
        "ModifiedCosine",
        "ModifiedGreedyCosine",
        "EntropySimilarityWeighted",
        "EntropySimilarityUnweighted",
    ] {
        let rust_impl_id = db::get_implementation_id(
            &mut test_db.conn,
            algorithm_name,
            "mass-spectrometry-traits",
        );
        let count_after_second_run = results::table
            .filter(results::implementation_id.eq(rust_impl_id))
            .select(count_star())
            .first::<i64>(&mut test_db.conn)
            .expect("failed to count rust results after second compute run");
        assert_eq!(count_after_second_run, expected_rust_rows);
    }
}

#[test]
fn compute_honors_max_spectra_when_db_contains_more_rows() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 5);

    compute::run_with_matchms(&mut test_db.conn, Some(3), |_| {});

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
        "ModifiedCosine",
        "ModifiedGreedyCosine",
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

#[test]
fn estimate_remaining_work_includes_non_reference_python_implementations() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 3);

    let spectra_count = spectra::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count spectra");
    let experiments_count = experiments::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count experiments");
    let n_pairs = spectra_count * (spectra_count + 1) / 2;
    let implementations_count = implementations::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count implementations");

    let expected_total = (n_pairs * experiments_count * implementations_count) as u64;
    let remaining = compute::estimate_remaining_work(&mut test_db.conn, Some(3));
    assert_eq!(remaining, expected_total);
}
