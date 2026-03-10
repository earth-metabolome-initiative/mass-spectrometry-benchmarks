use std::collections::BTreeMap;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Double, Integer, Text};
use diesel::sqlite::SqliteConnection;

use super::aggregate::bucket_index;
use super::render::{AggregatedSeriesPoint, RMSE_LOG_FLOOR};

#[derive(Debug, QueryableByName)]
struct RawMetricRow {
    #[diesel(sql_type = Text)]
    facet_label: String,
    #[diesel(sql_type = Text)]
    series_label: String,
    #[diesel(sql_type = Text)]
    library_name: String,
    #[diesel(sql_type = Integer)]
    min_peaks: i32,
    #[diesel(sql_type = Double)]
    value: f64,
}

type AggKey = (String, String, String, usize);

struct BucketAccumulator {
    sum: f64,
    sum_sq: f64,
    count: usize,
}

fn aggregate_into_buckets(rows: Vec<RawMetricRow>) -> Vec<AggregatedSeriesPoint> {
    let mut buckets: BTreeMap<AggKey, BucketAccumulator> = BTreeMap::new();

    for row in rows {
        let Some(bidx) = bucket_index(row.min_peaks) else {
            continue;
        };
        if !row.value.is_finite() {
            continue;
        }
        let key = (row.facet_label, row.series_label, row.library_name, bidx);
        let acc = buckets.entry(key).or_insert(BucketAccumulator {
            sum: 0.0,
            sum_sq: 0.0,
            count: 0,
        });
        acc.sum += row.value;
        acc.sum_sq += row.value * row.value;
        acc.count += 1;
    }

    buckets
        .into_iter()
        .filter_map(
            |((facet_label, series_label, library_name, bucket_index), acc)| {
                if acc.count == 0 {
                    return None;
                }
                let mean = acc.sum / acc.count as f64;
                let std_dev = sample_std_dev(acc.count, acc.sum, acc.sum_sq);
                Some(AggregatedSeriesPoint {
                    facet_label,
                    series_label,
                    library_name,
                    bucket_index,
                    value: mean,
                    std_dev,
                    count: acc.count,
                })
            },
        )
        .collect()
}

fn sample_std_dev(sample_count: usize, sum_value: f64, sum_value_sq: f64) -> f64 {
    if sample_count < 2 {
        return 0.0;
    }

    let n = sample_count as f64;
    let variance = (sum_value_sq - (sum_value * sum_value / n)) / (n - 1.0);
    variance.max(0.0).sqrt()
}

const RAW_ROWS_SQL: &str =
    "SELECT vt.canonical_algorithm_name || ' (' || vt.canonical_reference_library_name || ')' AS facet_label,
            vt.algorithm_name || ' (' || vt.library_name || ')' AS series_label,
            vt.library_name AS library_name,
            MIN(sl.num_peaks, sr.num_peaks) AS min_peaks,
            CAST(r.median_time_us AS REAL) AS value
     FROM results r
     JOIN v_implementation_topology vt
       ON vt.implementation_id = r.implementation_id
     JOIN spectra sl ON sl.id = r.left_id
     JOIN spectra sr ON sr.id = r.right_id
     WHERE vt.canonical_reference_implementation_id IS NOT NULL
     ORDER BY facet_label, series_label";

pub(crate) fn load_timing_aggregate_rows(
    conn: &mut SqliteConnection,
) -> Vec<AggregatedSeriesPoint> {
    let rows: Vec<RawMetricRow> = sql_query(RAW_ROWS_SQL)
        .load(conn)
        .expect("failed to load raw timing rows");
    aggregate_into_buckets(rows)
}

pub(crate) fn load_rmse_aggregate_rows(conn: &mut SqliteConnection) -> Vec<AggregatedSeriesPoint> {
    let sq_floor = RMSE_LOG_FLOOR * RMSE_LOG_FLOOR;

    #[derive(Debug, QueryableByName)]
    struct RawRmseRow {
        #[diesel(sql_type = Text)]
        facet_label: String,
        #[diesel(sql_type = Text)]
        series_label: String,
        #[diesel(sql_type = Text)]
        library_name: String,
        #[diesel(sql_type = Integer)]
        min_peaks: i32,
        #[diesel(sql_type = Double)]
        sq_error: f64,
    }

    let query =
        "SELECT vt.canonical_algorithm_name || ' (' || vt.canonical_reference_library_name || ')' AS facet_label,
                vt.algorithm_name || ' (' || vt.library_name || ')' AS series_label,
                vt.library_name AS library_name,
                MIN(sl.num_peaks, sr.num_peaks) AS min_peaks,
                (CAST(r.score AS REAL) - CAST(ref.score AS REAL))
                    * (CAST(r.score AS REAL) - CAST(ref.score AS REAL)) AS sq_error
         FROM results r
         JOIN v_implementation_topology vt
           ON vt.implementation_id = r.implementation_id
         JOIN spectra sl ON sl.id = r.left_id
         JOIN spectra sr ON sr.id = r.right_id
         JOIN results ref
           ON ref.implementation_id = vt.canonical_reference_implementation_id
          AND ref.left_id = r.left_id
          AND ref.right_id = r.right_id
          AND ref.experiment_id = r.experiment_id
         WHERE vt.canonical_reference_implementation_id IS NOT NULL
           AND r.implementation_id <> vt.canonical_reference_implementation_id
         ORDER BY facet_label, series_label";

    let raw_rows: Vec<RawRmseRow> = sql_query(query)
        .load(conn)
        .expect("failed to load raw RMSE rows");

    let mut buckets: BTreeMap<AggKey, BucketAccumulator> = BTreeMap::new();

    for row in raw_rows {
        let Some(bidx) = bucket_index(row.min_peaks) else {
            continue;
        };
        if !row.sq_error.is_finite() {
            continue;
        }
        let sq = row.sq_error.max(sq_floor);
        let key = (row.facet_label, row.series_label, row.library_name, bidx);
        let acc = buckets.entry(key).or_insert(BucketAccumulator {
            sum: 0.0,
            sum_sq: 0.0,
            count: 0,
        });
        acc.sum += sq;
        acc.sum_sq += sq * sq;
        acc.count += 1;
    }

    buckets
        .into_iter()
        .filter_map(
            |((facet_label, series_label, library_name, bucket_index), acc)| {
                if acc.count == 0 {
                    return None;
                }
                let mean_sq = (acc.sum / acc.count as f64).max(sq_floor);
                let rmse = mean_sq.sqrt();
                let std_sq = sample_std_dev(acc.count, acc.sum, acc.sum_sq);
                let upper = (mean_sq + std_sq).max(sq_floor).sqrt();
                let lower = (mean_sq - std_sq).max(sq_floor).sqrt();
                Some(AggregatedSeriesPoint {
                    facet_label,
                    series_label,
                    library_name,
                    bucket_index,
                    value: rmse,
                    std_dev: (upper - lower) / 2.0,
                    count: acc.count,
                })
            },
        )
        .collect()
}

// --- Correlation data loading ---

#[derive(Debug, QueryableByName)]
pub(crate) struct CorrelationRow {
    #[diesel(sql_type = Text)]
    pub(crate) fingerprint_algorithm: String,
    #[diesel(sql_type = Text)]
    pub(crate) series_label: String,
    #[diesel(sql_type = Text)]
    pub(crate) library_name: String,
    #[diesel(sql_type = Double)]
    pub(crate) spectral_score: f64,
    #[diesel(sql_type = Double)]
    pub(crate) tanimoto_score: f64,
}

pub(crate) fn load_correlation_rows(conn: &mut SqliteConnection) -> Vec<CorrelationRow> {
    let query = "SELECT fa.name AS fingerprint_algorithm,
                        vt.algorithm_name || ' (' || vt.library_name || ')' AS series_label,
                        vt.library_name AS library_name,
                        CAST(r.score AS REAL) AS spectral_score,
                        t.tanimoto_score AS tanimoto_score
                 FROM results r
                 JOIN v_implementation_topology vt
                   ON vt.implementation_id = r.implementation_id
                 JOIN tanimoto_results t
                   ON t.left_id = r.left_id AND t.right_id = r.right_id
                 JOIN fingerprint_algorithms fa
                   ON fa.id = t.fingerprint_algorithm_id
                 WHERE vt.canonical_reference_implementation_id IS NOT NULL
                  AND vt.library_name = 'mass-spectrometry-traits'
                  AND vt.algorithm_name NOT IN ('LinearCosine', 'ModifiedLinearCosine', 'ModifiedCosineMerged')
                 ORDER BY fa.name, series_label";

    sql_query(query)
        .load(conn)
        .expect("failed to load correlation rows")
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
