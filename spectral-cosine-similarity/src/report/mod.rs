mod aggregate;
mod compare;
mod data;
mod markdown;
mod render;
mod style;
mod types;

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel::sqlite::SqliteConnection;

use crate::progress::StageProgress;

use aggregate::{MetricKind, build_metric_chart_from_aggregates, omit_empty_buckets};
use data::{load_rmse_aggregate_rows, load_timing_aggregate_rows, series_pairs_from_aggregates};
use markdown::{RunScopeMetadata, write_markdown_tables};
use render::render_faceted_line_chart;
use style::build_series_style_map_from_pairs;
use types::FacetedLineChart;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactNames {
    pub timing_svg: String,
    pub rmse_svg: String,
    pub markdown: String,
}

impl Default for ArtifactNames {
    fn default() -> Self {
        Self {
            timing_svg: "timing.svg".to_string(),
            rmse_svg: "rmse.svg".to_string(),
            markdown: "tables.md".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReportConfig {
    pub output_dir: PathBuf,
    pub artifact_names: ArtifactNames,
    pub include_comparison: bool,
    pub prune_empty_buckets: bool,
    pub requested_max_spectra: Option<usize>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("output"),
            artifact_names: ArtifactNames::default(),
            include_comparison: true,
            prune_empty_buckets: true,
            requested_max_spectra: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReportArtifacts {
    pub timing_svg: Option<PathBuf>,
    pub rmse_svg: Option<PathBuf>,
    pub markdown: PathBuf,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(progress) = progress.as_deref_mut() {
        progress.set_message(message);
    } else {
        eprintln!("{message}");
    }
}

fn remove_file_if_exists(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => panic!("failed to remove stale chart {}: {err}", path.display()),
    }
}

fn spectra_used_in_results_count(conn: &mut SqliteConnection) -> i64 {
    sql_query(
        "SELECT COUNT(*) AS n
         FROM (
             SELECT left_id AS spectrum_id FROM results
             UNION
             SELECT right_id AS spectrum_id FROM results
         )",
    )
    .get_result::<CountRow>(conn)
    .expect("failed to count spectra represented in results")
    .n
}

fn total_spectra_count(conn: &mut SqliteConnection) -> i64 {
    sql_query("SELECT COUNT(*) AS n FROM spectra")
        .get_result::<CountRow>(conn)
        .expect("failed to count spectra")
        .n
}

fn render_chart_artifact(
    chart: &FacetedLineChart,
    output_path: &Path,
    progress: &mut Option<&mut dyn StageProgress>,
    render_message: &str,
) -> Option<PathBuf> {
    if chart.facets.is_empty() || chart.bucket_labels.is_empty() {
        remove_file_if_exists(output_path);
        return None;
    }

    emit(progress, render_message);
    render_faceted_line_chart(chart, output_path.to_string_lossy().as_ref())
        .unwrap_or_else(|err| panic!("failed to render chart {}: {err}", output_path.display()));
    emit(
        progress,
        &format!("[report] Written {}", output_path.display()),
    );
    Some(output_path.to_path_buf())
}

pub fn generate(
    conn: &mut SqliteConnection,
    cfg: &ReportConfig,
    mut progress: Option<&mut dyn StageProgress>,
) -> ReportArtifacts {
    emit(&mut progress, "[report] Generating charts");
    fs::create_dir_all(&cfg.output_dir).expect("failed to create output directory");
    let total_spectra = total_spectra_count(conn);
    let spectra_used = spectra_used_in_results_count(conn);

    emit(&mut progress, "[report] Loading aggregated result data");
    let timing_rows = load_timing_aggregate_rows(conn);
    let rmse_rows = load_rmse_aggregate_rows(conn);
    let mut series_pairs = series_pairs_from_aggregates(&timing_rows);
    series_pairs.extend(series_pairs_from_aggregates(&rmse_rows));
    series_pairs.sort();
    series_pairs.dedup();
    let style_map = build_series_style_map_from_pairs(&series_pairs);

    let mut timing_chart =
        build_metric_chart_from_aggregates(MetricKind::Timing, &timing_rows, &style_map);
    let mut rmse_chart =
        build_metric_chart_from_aggregates(MetricKind::Rmse, &rmse_rows, &style_map);

    if cfg.prune_empty_buckets {
        timing_chart = omit_empty_buckets(timing_chart);
        rmse_chart = omit_empty_buckets(rmse_chart);
    }

    let run_scope = format!(" (Spectra used: {spectra_used})");
    timing_chart.title.push_str(&run_scope);
    rmse_chart.title.push_str(&run_scope);

    let timing_path = cfg.output_dir.join(&cfg.artifact_names.timing_svg);
    let timing_svg = render_chart_artifact(
        &timing_chart,
        &timing_path,
        &mut progress,
        "[report] Rendering timing chart",
    );

    let rmse_path = cfg.output_dir.join(&cfg.artifact_names.rmse_svg);
    let rmse_svg = render_chart_artifact(
        &rmse_chart,
        &rmse_path,
        &mut progress,
        "[report] Rendering RMSE chart",
    );

    let markdown_path = cfg.output_dir.join(&cfg.artifact_names.markdown);
    let run_scope = RunScopeMetadata {
        requested_max_spectra: cfg.requested_max_spectra,
        total_spectra_in_db: total_spectra,
        spectra_used_in_results: spectra_used,
    };
    write_markdown_tables(&markdown_path, &[&timing_chart, &rmse_chart], &run_scope);
    emit(
        &mut progress,
        &format!("[report] Written {}", markdown_path.display()),
    );

    if cfg.include_comparison {
        compare::compare_results(conn, progress);
    }

    ReportArtifacts {
        timing_svg,
        rmse_svg,
        markdown: markdown_path,
    }
}

#[cfg(test)]
mod tests {
    use diesel::Connection;

    use super::aggregate::{bucket_index, bucket_labels, mean, std_dev};
    use super::render::{YMode, build_series_points, build_series_whiskers, select_y_mode};
    use super::types::{
        BUCKET_BOUNDARIES, FacetChart, LIBRARY_COLORS, LineSeriesData, MarkerShape,
    };
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    #[test]
    fn bucket_boundaries_and_labels_are_stable() {
        assert_eq!(bucket_index(4), None);
        assert_eq!(bucket_index(5), Some(0));
        assert_eq!(bucket_index(8), Some(0));
        assert_eq!(bucket_index(9), Some(1));
        assert_eq!(bucket_index(513), Some(7));
        assert_eq!(bucket_index(1023), Some(7));
        assert_eq!(bucket_index(1024), Some(8));
        assert_eq!(bucket_index(2047), Some(8));
        assert_eq!(bucket_index(2048), Some(BUCKET_BOUNDARIES.len() - 1));

        let labels = bucket_labels();
        assert_eq!(labels.first().expect("missing first label"), "5–8");
        assert_eq!(labels.last().expect("missing last label"), "2048+");
        assert_eq!(labels.len(), BUCKET_BOUNDARIES.len());
    }

    #[test]
    fn mean_and_std_dev_handle_edge_cases() {
        assert_eq!(mean(&[]), 0.0);
        assert_eq!(std_dev(&[]), 0.0);
        assert_eq!(std_dev(&[42.0]), 0.0);

        let m = mean(&[2.0, 4.0, 6.0]);
        let sd = std_dev(&[2.0, 4.0, 6.0]);
        assert!((m - 4.0).abs() < 1e-9);
        assert!((sd - 2.0).abs() < 1e-9);
    }

    #[test]
    fn series_points_log_filters_non_positive_values() {
        let points = build_series_points(&[-1.0, 0.0, 1.0, 2.0], true);
        assert_eq!(points, vec![(2.0, 1.0), (3.0, 2.0)]);
    }

    #[test]
    fn series_points_linear_keeps_zero_and_negative_values() {
        let points = build_series_points(&[-1.0, 0.0, 1.0, 2.0], false);
        assert_eq!(
            points,
            vec![(0.0, -1.0), (1.0, 0.0), (2.0, 1.0), (3.0, 2.0)]
        );
    }

    #[test]
    fn whisker_generation_is_mode_aware() {
        let series = LineSeriesData {
            label: "s".to_string(),
            color: LIBRARY_COLORS[0],
            marker: MarkerShape::Circle,
            values: vec![0.0, 1.0],
            std_devs: vec![0.1, 0.1],
            counts: vec![1, 1],
        };

        let linear = build_series_whiskers(&series, false, 0.0);
        let log = build_series_whiskers(&series, true, 1e-6);
        assert_eq!(linear.len(), 6);
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn y_mode_selection_matches_log_and_linear_fallback_behavior() {
        let log_chart = FacetedLineChart {
            title: "t".to_string(),
            y_label: "y".to_string(),
            bucket_labels: vec!["A".to_string()],
            facets: Vec::new(),
            log_y: true,
        };
        let facet_log = FacetChart {
            title: "f".to_string(),
            series: vec![LineSeriesData {
                label: "s".to_string(),
                color: LIBRARY_COLORS[0],
                marker: MarkerShape::Circle,
                values: vec![1.0],
                std_devs: vec![0.0],
                counts: vec![1],
            }],
        };
        assert!(matches!(
            select_y_mode(&log_chart, &facet_log),
            Some(YMode::Log { .. })
        ));

        let facet_linear_fallback = FacetChart {
            title: "f".to_string(),
            series: vec![LineSeriesData {
                label: "s".to_string(),
                color: LIBRARY_COLORS[0],
                marker: MarkerShape::Circle,
                values: vec![0.0],
                std_devs: vec![0.2],
                counts: vec![1],
            }],
        };
        assert!(matches!(
            select_y_mode(&log_chart, &facet_linear_fallback),
            Some(YMode::Linear { .. })
        ));
    }

    #[test]
    fn omit_empty_buckets_drops_columns_with_no_observations() {
        let chart = FacetedLineChart {
            title: "test".to_string(),
            y_label: "y".to_string(),
            bucket_labels: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            facets: vec![FacetChart {
                title: "f".to_string(),
                series: vec![
                    LineSeriesData {
                        label: "s1".to_string(),
                        color: LIBRARY_COLORS[0],
                        marker: MarkerShape::Circle,
                        values: vec![0.0, 2.0, 0.0],
                        std_devs: vec![0.0, 0.2, 0.0],
                        counts: vec![0, 3, 0],
                    },
                    LineSeriesData {
                        label: "s2".to_string(),
                        color: LIBRARY_COLORS[1],
                        marker: MarkerShape::Square,
                        values: vec![0.0, 1.0, 0.0],
                        std_devs: vec![0.0, 0.1, 0.0],
                        counts: vec![0, 2, 0],
                    },
                ],
            }],
            log_y: true,
        };

        let pruned = omit_empty_buckets(chart);
        assert_eq!(pruned.bucket_labels, vec!["B"]);
        assert_eq!(pruned.facets.len(), 1);
        assert_eq!(pruned.facets[0].series.len(), 2);
        assert_eq!(pruned.facets[0].series[0].counts, vec![3]);
        assert_eq!(pruned.facets[0].series[1].counts, vec![2]);
    }

    #[test]
    fn generate_removes_stale_svgs_when_no_chart_data() {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("failed to open in-memory sqlite db");
        db::initialize(&mut conn);

        let output_dir = TempDir::new().expect("failed to create temp output dir");
        let timing_path = output_dir.path().join("timing.svg");
        let mse_path = output_dir.path().join("rmse.svg");
        fs::write(&timing_path, "stale timing").expect("failed to create stale timing chart");
        fs::write(&mse_path, "stale rmse").expect("failed to create stale rmse chart");

        let config = ReportConfig {
            output_dir: output_dir.path().to_path_buf(),
            include_comparison: false,
            ..ReportConfig::default()
        };
        generate(&mut conn, &config, None);

        assert!(
            !timing_path.exists(),
            "stale timing chart should be removed when timing chart has no data"
        );
        assert!(
            !mse_path.exists(),
            "stale RMSE chart should be removed when RMSE chart has no data"
        );
        let markdown_path = output_dir.path().join("tables.md");
        assert!(
            markdown_path.exists(),
            "markdown report should always be written"
        );
        let markdown = fs::read_to_string(&markdown_path).expect("failed to read markdown report");
        assert!(markdown.contains("# Benchmark Tables"));
        assert!(markdown.contains("## Run Scope"));
        assert!(markdown.contains("Requested max spectra: `unbounded`"));
        assert!(markdown.contains("Total spectra in DB: `0`"));
        assert!(markdown.contains("Spectra used in results: `0`"));
        assert!(markdown.contains("## Timing by Peak Count"));
        assert!(markdown.contains("## RMSE vs Reference by Peak Count"));
        assert!(markdown.contains("Spectra used: 0"));
        assert!(markdown.contains("_No data available._"));
    }
}
