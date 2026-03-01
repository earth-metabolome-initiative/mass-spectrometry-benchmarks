use diesel::dsl::count_star;
use diesel::prelude::*;

use spectral_cosine_similarity::schema::results;
use spectral_cosine_similarity::{compute, db};

mod common;

#[test]
fn compute_rust_results_resume_without_duplicates() {
    let mut test_db = common::TestDb::new();
    common::prepare_small_dataset(&mut test_db.conn, 3);

    compute::run_with_matchms(&mut test_db.conn, Some(3), |_| {});

    let rust_impl_id = db::get_implementation_id(
        &mut test_db.conn,
        "CosineHungarian",
        "mass-spectrometry-traits",
    );

    let expected_rust_rows = 6_i64 * 4_i64;
    let count_after_first_run = results::table
        .filter(results::implementation_id.eq(rust_impl_id))
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count rust results after first compute run");
    assert_eq!(count_after_first_run, expected_rust_rows);

    compute::run_with_matchms(&mut test_db.conn, Some(3), |_| {});

    let count_after_second_run = results::table
        .filter(results::implementation_id.eq(rust_impl_id))
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count rust results after second compute run");
    assert_eq!(count_after_second_run, expected_rust_rows);
}
