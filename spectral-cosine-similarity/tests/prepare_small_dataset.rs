use diesel::dsl::count_star;
use diesel::prelude::*;
use spectral_cosine_similarity::schema::spectra;

mod common;

#[test]
fn prepare_with_small_cap_is_idempotent() {
    let mut test_db = common::TestDb::new();

    common::prepare_small_dataset(&mut test_db.conn, 3);
    let count_after_first_run = spectra::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count spectra after first prepare run");
    assert_eq!(count_after_first_run, 3);

    common::prepare_small_dataset(&mut test_db.conn, 3);
    let count_after_second_run = spectra::table
        .select(count_star())
        .first::<i64>(&mut test_db.conn)
        .expect("failed to count spectra after second prepare run");
    assert_eq!(count_after_second_run, 3);
}
