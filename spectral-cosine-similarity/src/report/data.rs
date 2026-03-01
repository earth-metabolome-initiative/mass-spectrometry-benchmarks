#[cfg(test)]
use std::collections::HashMap;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Double, Integer, Text};
use diesel::sqlite::SqliteConnection;

#[cfg(test)]
use super::style::series_label;
use super::types::{AggregatedSeriesPoint, RMSE_LOG_FLOOR};
#[cfg(test)]
use super::types::{AlgorithmReference, ResultRow};

#[derive(Debug, QueryableByName)]
struct AggregatedMetricRow {
    #[diesel(sql_type = Text)]
    facet_label: String,
    #[diesel(sql_type = Text)]
    series_label: String,
    #[diesel(sql_type = Text)]
    library_name: String,
    #[diesel(sql_type = Integer)]
    bucket_index: i32,
    #[diesel(sql_type = BigInt)]
    sample_count: i64,
    #[diesel(sql_type = Double)]
    sum_value: f64,
    #[diesel(sql_type = Double)]
    sum_value_sq: f64,
}

fn bucketed_rows_cte_sql() -> &'static str {
    "WITH bucketed AS (
         SELECT r.implementation_id,
                r.left_id,
                r.right_id,
                r.experiment_id,
                r.score,
                r.median_time_us,
                vt.algorithm_name,
                vt.library_name,
                vt.canonical_algorithm_name,
                vt.canonical_reference_implementation_id,
                vt.canonical_reference_library_name,
                CASE
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 2048 THEN 9
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 1024 THEN 8
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 513 THEN 7
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 257 THEN 6
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 129 THEN 5
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 65 THEN 4
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 33 THEN 3
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 17 THEN 2
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 9 THEN 1
                    WHEN CASE WHEN sl.num_peaks > sr.num_peaks THEN sl.num_peaks ELSE sr.num_peaks END >= 5 THEN 0
                    ELSE -1
                END AS bucket_index
         FROM results r
         JOIN v_implementation_topology vt
           ON vt.implementation_id = r.implementation_id
         JOIN spectra sl
           ON sl.id = r.left_id
         JOIN spectra sr
           ON sr.id = r.right_id
     )"
}

fn sample_std_dev(sample_count: usize, sum_value: f64, sum_value_sq: f64) -> f64 {
    if sample_count < 2 {
        return 0.0;
    }

    let n = sample_count as f64;
    let variance = (sum_value_sq - (sum_value * sum_value / n)) / (n - 1.0);
    variance.max(0.0).sqrt()
}

pub(crate) fn load_timing_aggregate_rows(
    conn: &mut SqliteConnection,
) -> Vec<AggregatedSeriesPoint> {
    let query = format!(
        "{}
         SELECT b.canonical_algorithm_name || ' (' || b.canonical_reference_library_name || ')' AS facet_label,
                b.algorithm_name || ' (' || b.library_name || ')' AS series_label,
                b.library_name AS library_name,
                b.bucket_index AS bucket_index,
                COUNT(*) AS sample_count,
                SUM(CAST(b.median_time_us AS REAL)) AS sum_value,
                SUM(CAST(b.median_time_us AS REAL) * CAST(b.median_time_us AS REAL)) AS sum_value_sq
         FROM bucketed b
         WHERE b.bucket_index >= 0
           AND b.canonical_reference_implementation_id IS NOT NULL
         GROUP BY facet_label, series_label, library_name, bucket_index
         ORDER BY facet_label, series_label, bucket_index",
        bucketed_rows_cte_sql()
    );

    let rows: Vec<AggregatedMetricRow> = sql_query(query)
        .load(conn)
        .expect("failed to load SQL-aggregated timing rows");

    rows.into_iter()
        .filter_map(|row| {
            let bucket_index = usize::try_from(row.bucket_index).ok()?;
            let count = usize::try_from(row.sample_count).ok()?;
            if count == 0 {
                return None;
            }
            Some(AggregatedSeriesPoint {
                facet_label: row.facet_label,
                series_label: row.series_label,
                library_name: row.library_name,
                bucket_index,
                value: row.sum_value / count as f64,
                std_dev: sample_std_dev(count, row.sum_value, row.sum_value_sq),
                count,
            })
        })
        .collect()
}

pub(crate) fn load_rmse_aggregate_rows(conn: &mut SqliteConnection) -> Vec<AggregatedSeriesPoint> {
    let sq_floor = RMSE_LOG_FLOOR * RMSE_LOG_FLOOR;
    let query = format!(
        "{},
         paired AS (
             SELECT b.canonical_algorithm_name || ' (' || b.canonical_reference_library_name || ')' AS facet_label,
                    b.algorithm_name || ' (' || b.library_name || ')' AS series_label,
                    b.library_name AS library_name,
                    b.bucket_index AS bucket_index,
                    CASE
                        WHEN ((CAST(b.score AS REAL) - CAST(ref.score AS REAL))
                              * (CAST(b.score AS REAL) - CAST(ref.score AS REAL))) < {sq_floor}
                        THEN {sq_floor}
                        ELSE ((CAST(b.score AS REAL) - CAST(ref.score AS REAL))
                              * (CAST(b.score AS REAL) - CAST(ref.score AS REAL)))
                    END AS sq_error
             FROM bucketed b
             JOIN results ref
               ON ref.implementation_id = b.canonical_reference_implementation_id
              AND ref.left_id = b.left_id
              AND ref.right_id = b.right_id
              AND ref.experiment_id = b.experiment_id
             WHERE b.bucket_index >= 0
               AND b.canonical_reference_implementation_id IS NOT NULL
               AND b.implementation_id <> b.canonical_reference_implementation_id
         )
         SELECT p.facet_label AS facet_label,
                p.series_label AS series_label,
                p.library_name AS library_name,
                p.bucket_index AS bucket_index,
                COUNT(*) AS sample_count,
                SUM(p.sq_error) AS sum_value,
                SUM(p.sq_error * p.sq_error) AS sum_value_sq
         FROM paired p
         GROUP BY p.facet_label, p.series_label, p.library_name, p.bucket_index
         ORDER BY p.facet_label, p.series_label, p.bucket_index",
        bucketed_rows_cte_sql(),
        sq_floor = sq_floor
    );

    let rows: Vec<AggregatedMetricRow> = sql_query(query)
        .load(conn)
        .expect("failed to load SQL-aggregated RMSE rows");

    rows.into_iter()
        .filter_map(|row| {
            let bucket_index = usize::try_from(row.bucket_index).ok()?;
            let count = usize::try_from(row.sample_count).ok()?;
            if count == 0 {
                return None;
            }

            let mean_sq = (row.sum_value / count as f64).max(sq_floor);
            let rmse = mean_sq.sqrt();
            let std_sq = sample_std_dev(count, row.sum_value, row.sum_value_sq);
            let upper = (mean_sq + std_sq).max(sq_floor).sqrt();
            let lower = (mean_sq - std_sq).max(sq_floor).sqrt();

            Some(AggregatedSeriesPoint {
                facet_label: row.facet_label,
                series_label: row.series_label,
                library_name: row.library_name,
                bucket_index,
                value: rmse,
                std_dev: (upper - lower) / 2.0,
                count,
            })
        })
        .collect()
}

pub(crate) fn series_pairs_from_aggregates(
    rows: &[AggregatedSeriesPoint],
) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = rows
        .iter()
        .map(|row| (row.series_label.clone(), row.library_name.clone()))
        .collect();
    pairs.sort();
    pairs.dedup();
    pairs
}

#[cfg(test)]
pub(crate) fn algorithm_references(
    data: &[ResultRow],
    canonical_algorithm_map: &HashMap<String, String>,
) -> HashMap<String, AlgorithmReference> {
    let mut canonical_refs: HashMap<String, AlgorithmReference> = HashMap::new();

    for row in data.iter().filter(|r| r.is_reference) {
        let canonical_name = canonical_algorithm_map
            .get(&row.algo_name)
            .map(String::as_str)
            .unwrap_or(&row.algo_name);

        // Only canonical algorithms define canonical references.
        if row.algo_name != canonical_name {
            continue;
        }

        let label = series_label(&row.algo_name, &row.lib_name);
        match canonical_refs.entry(canonical_name.to_string()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(AlgorithmReference {
                    implementation_id: row.implementation_id,
                    label,
                });
            }
            std::collections::hash_map::Entry::Occupied(entry) => {
                if entry.get().implementation_id != row.implementation_id {
                    panic!(
                        "canonical algorithm '{}' has multiple reference implementations in results",
                        canonical_name
                    );
                }
            }
        }
    }

    let mut refs: HashMap<String, AlgorithmReference> = HashMap::new();
    let mut algorithms: Vec<String> = data.iter().map(|r| r.algo_name.clone()).collect();
    algorithms.sort();
    algorithms.dedup();

    for algorithm_name in algorithms {
        let canonical_name = canonical_algorithm_map
            .get(&algorithm_name)
            .map(String::as_str)
            .unwrap_or(&algorithm_name);

        if let Some(reference) = canonical_refs.get(canonical_name) {
            refs.insert(algorithm_name, reference.clone());
        }
    }

    refs
}
