use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Double, Integer, Text};
use diesel::sqlite::SqliteConnection;

use crate::progress::StageProgress;

use super::types::algorithm_uses_match_count_parity;

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(progress) = progress.as_deref_mut() {
        progress.set_substep(message);
    } else {
        eprintln!("{message}");
    }
}

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

pub(crate) fn compare_results(
    conn: &mut SqliteConnection,
    mut progress: Option<&mut dyn StageProgress>,
) {
    emit(
        &mut progress,
        "[report] Comparing Rust results against references",
    );

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
        emit(
            &mut progress,
            "[report] No Rust-vs-reference comparisons available yet.",
        );
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

    if progress.is_some() {
        emit(
            &mut progress,
            &format!(
                "[report] Compared {} pair(s), mismatch_count={mismatch_count}, rmse={rmse:.6e}",
                rows.len()
            ),
        );
    } else {
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
}
