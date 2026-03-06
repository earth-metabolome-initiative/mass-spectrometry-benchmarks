use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Double, Integer, Text};
use diesel::sqlite::SqliteConnection;

use super::render::algorithm_uses_match_count_parity;
use crate::db;

#[derive(QueryableByName)]
#[allow(dead_code)]
struct ComparisonRow {
    #[diesel(sql_type = Text)]
    algorithm_name: String,
    #[diesel(sql_type = Text)]
    rust_library: String,
    #[diesel(sql_type = Text)]
    reference_library: String,
    #[diesel(sql_type = Integer)]
    left_id: i32,
    #[diesel(sql_type = Integer)]
    right_id: i32,
    #[diesel(sql_type = Integer)]
    experiment_id: i32,
    #[diesel(sql_type = Double)]
    rust_score: f64,
    #[diesel(sql_type = Integer)]
    rust_matches: i32,
    #[diesel(sql_type = Double)]
    reference_score: f64,
    #[diesel(sql_type = Integer)]
    reference_matches: i32,
}

pub(crate) fn compare_results(conn: &mut SqliteConnection) {
    eprintln!("[report] Comparing Rust results against references");

    let rows: Vec<ComparisonRow> = sql_query(
        "SELECT a.name AS algorithm_name,
                rl.name AS rust_library,
                refl.name AS reference_library,
                r.left_id,
                r.right_id,
                r.experiment_id,
                r.score AS rust_score,
                r.matches AS rust_matches,
                ref.score AS reference_score,
                ref.matches AS reference_matches
         FROM results r
         JOIN implementations ri ON r.implementation_id = ri.id
         JOIN algorithms a ON ri.algorithm_id = a.id
         JOIN algorithms canonical ON canonical.id = COALESCE(a.approximates_algorithm_id, a.id)
         JOIN libraries rl ON ri.library_id = rl.id
         JOIN implementations refi ON refi.algorithm_id = canonical.id AND refi.is_reference = 1
         JOIN libraries refl ON refi.library_id = refl.id
         JOIN results ref ON ref.left_id = r.left_id
                        AND ref.right_id = r.right_id
                        AND ref.experiment_id = r.experiment_id
                        AND ref.implementation_id = refi.id
         WHERE rl.language = 'rust'
           AND ri.id != refi.id
         ORDER BY a.name, r.experiment_id, r.left_id, r.right_id",
    )
    .load(conn)
    .expect("failed to compare results");

    if rows.is_empty() {
        eprintln!("[report] No Rust-vs-reference comparisons available yet.");
        return;
    }

    let mut max_score_diff: f64 = 0.0;
    let mut max_match_diff: i32 = 0;
    let mut sum_sq_score: f64 = 0.0;
    let mut mismatch_count = 0usize;

    for row in &rows {
        let score_diff = (row.rust_score - row.reference_score).abs();
        let match_diff = (row.rust_matches - row.reference_matches).abs();

        if score_diff > max_score_diff {
            max_score_diff = score_diff;
        }

        if algorithm_uses_match_count_parity(&row.algorithm_name) && match_diff > max_match_diff {
            max_match_diff = match_diff;
        }

        sum_sq_score += score_diff * score_diff;

        let mismatch = if algorithm_uses_match_count_parity(&row.algorithm_name) {
            score_diff > 1e-6 || match_diff > 0
        } else {
            score_diff > 1e-6
        };
        if mismatch {
            mismatch_count += 1;
        }
    }

    let rmse = (sum_sq_score / rows.len() as f64).sqrt();

    eprintln!("[report] Cross-implementation comparison (Rust vs DB-marked reference):");
    eprintln!("[report]   Pairs compared: {}", rows.len());
    eprintln!(
        "[report]   Mismatches: score>1e-6 for all algorithms; matches must also agree for cosine-family algorithms"
    );
    eprintln!("[report]   Mismatch count: {mismatch_count}");
    eprintln!("[report]   Max score diff: {max_score_diff:.6e}");
    eprintln!("[report]   Max match diff: {max_match_diff}");
    eprintln!("[report]   RMSE (score): {rmse:.6e}");
}

#[derive(QueryableByName)]
struct MergedBaselineRow {
    #[diesel(sql_type = Double)]
    linear_score: f64,
    #[diesel(sql_type = Integer)]
    linear_matches: i32,
    #[diesel(sql_type = Double)]
    merged_score: f64,
    #[diesel(sql_type = Integer)]
    merged_matches: i32,
}

pub(crate) fn compare_merged_baselines(conn: &mut SqliteConnection) {
    const RUST_LIB: &str = "mass-spectrometry-traits";
    const PAIRS: &[(&str, &str)] = &[
        ("LinearCosine", "CosineHungarianMerged"),
        ("ModifiedLinearCosine", "ModifiedCosineMerged"),
    ];

    for &(linear_name, merged_name) in PAIRS {
        let linear_id = db::get_implementation_id(conn, linear_name, RUST_LIB);
        let merged_id = db::get_implementation_id(conn, merged_name, RUST_LIB);

        let rows: Vec<MergedBaselineRow> = sql_query(
            "SELECT l.score AS linear_score,
                    l.matches AS linear_matches,
                    m.score AS merged_score,
                    m.matches AS merged_matches
             FROM results l
             JOIN results m ON m.left_id = l.left_id
                           AND m.right_id = l.right_id
                           AND m.experiment_id = l.experiment_id
                           AND m.implementation_id = ?2
             WHERE l.implementation_id = ?1",
        )
        .bind::<Integer, _>(linear_id)
        .bind::<Integer, _>(merged_id)
        .load(conn)
        .expect("failed to load merged baseline comparison rows");

        if rows.is_empty() {
            eprintln!("[report] {linear_name} vs {merged_name}: no data yet");
            continue;
        }

        let mut max_score_diff: f64 = 0.0;
        let mut max_match_diff: i32 = 0;
        let mut sum_sq: f64 = 0.0;
        let mut mismatch_count = 0usize;

        for row in &rows {
            let sd = (row.linear_score - row.merged_score).abs();
            let md = (row.linear_matches - row.merged_matches).abs();
            if sd > max_score_diff {
                max_score_diff = sd;
            }
            if md > max_match_diff {
                max_match_diff = md;
            }
            sum_sq += sd * sd;
            if sd > 1e-6 || md > 0 {
                mismatch_count += 1;
            }
        }

        let rmse = (sum_sq / rows.len() as f64).sqrt();
        eprintln!("[report] {linear_name} vs {merged_name}:");
        eprintln!("[report]   Pairs compared: {}", rows.len());
        eprintln!("[report]   Mismatch count: {mismatch_count}");
        eprintln!("[report]   Max score diff: {max_score_diff:.6e}");
        eprintln!("[report]   Max match diff: {max_match_diff}");
        eprintln!("[report]   RMSE (score): {rmse:.6e}");
    }
}
