use std::collections::{BTreeMap, HashMap};

#[cfg(test)]
use super::style::series_label;
use super::types::{
    AggregatedSeriesPoint, BUCKET_BOUNDARIES, FacetChart, FacetedLineChart, LIBRARY_COLORS,
    LineSeriesData, MarkerShape, SeriesStyle,
};
#[cfg(test)]
use super::types::{AlgorithmReference, RMSE_LOG_FLOOR, ResultRow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetricKind {
    Timing,
    Rmse,
}

impl MetricKind {
    fn chart_title(self) -> &'static str {
        match self {
            Self::Timing => "Timing by Peak Count",
            Self::Rmse => "RMSE vs Reference by Peak Count",
        }
    }

    fn y_label(self) -> &'static str {
        match self {
            Self::Timing => "Mean time (µs)",
            Self::Rmse => "RMSE",
        }
    }

    fn use_log_scale(self) -> bool {
        true
    }
}

#[cfg(test)]
pub(crate) fn bucket_index(max_peaks: i32) -> Option<usize> {
    if max_peaks < BUCKET_BOUNDARIES[0] {
        return None;
    }

    (0..BUCKET_BOUNDARIES.len())
        .rev()
        .find(|&i| max_peaks >= BUCKET_BOUNDARIES[i])
}

pub(crate) fn bucket_labels() -> Vec<String> {
    BUCKET_BOUNDARIES
        .windows(2)
        .map(|w| format!("{}–{}", w[0], w[1] - 1))
        .chain(std::iter::once(format!(
            "{}+",
            BUCKET_BOUNDARIES
                .last()
                .expect("bucket boundaries are not empty")
        )))
        .collect()
}

#[cfg(test)]
pub(crate) fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

#[cfg(test)]
pub(crate) fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance =
        values.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

#[cfg(test)]
fn build_reference_score_index(data: &[ResultRow]) -> HashMap<i32, HashMap<(i32, i32, i32), f32>> {
    let mut reference_scores: HashMap<i32, HashMap<(i32, i32, i32), f32>> = HashMap::new();
    for row in data.iter().filter(|r| r.is_reference) {
        reference_scores
            .entry(row.implementation_id)
            .or_default()
            .insert((row.left_id, row.right_id, row.experiment_id), row.score);
    }
    reference_scores
}

#[cfg(test)]
fn collect_bucketed_series<F>(
    data: &[ResultRow],
    spectra_peaks: &HashMap<i32, i32>,
    references: &HashMap<String, AlgorithmReference>,
    n_buckets: usize,
    mut value_for_row: F,
) -> HashMap<(String, String), Vec<Vec<f64>>>
where
    F: FnMut(&ResultRow, &AlgorithmReference) -> Option<f64>,
{
    let mut grouped: HashMap<(String, String), Vec<Vec<f64>>> = HashMap::new();

    for row in data {
        let Some(reference) = references.get(&row.algo_name) else {
            continue;
        };

        let Some(value) = value_for_row(row, reference) else {
            continue;
        };

        if !value.is_finite() {
            continue;
        }

        if let (Some(&lp), Some(&rp)) = (
            spectra_peaks.get(&row.left_id),
            spectra_peaks.get(&row.right_id),
        ) && let Some(bucket_idx) = bucket_index(lp.max(rp))
        {
            grouped
                .entry((
                    reference.label.clone(),
                    series_label(&row.algo_name, &row.lib_name),
                ))
                .or_insert_with(|| vec![Vec::new(); n_buckets])[bucket_idx]
                .push(value);
        }
    }

    grouped
}

#[cfg(test)]
fn grouped_to_facets(
    grouped: HashMap<(String, String), Vec<Vec<f64>>>,
    style_map: &HashMap<String, SeriesStyle>,
    bucket_labels: Vec<String>,
    metric: MetricKind,
) -> FacetedLineChart {
    let mut by_facet: BTreeMap<String, BTreeMap<String, Vec<Vec<f64>>>> = BTreeMap::new();

    for ((facet_label, series_label), buckets) in grouped {
        by_facet
            .entry(facet_label)
            .or_default()
            .insert(series_label, buckets);
    }

    let facets = by_facet
        .into_iter()
        .map(|(facet_label, series_map)| {
            let series = series_map
                .into_iter()
                .map(|(label, buckets)| {
                    let style = style_map.get(&label).copied().unwrap_or(SeriesStyle {
                        color: LIBRARY_COLORS[0],
                        marker: MarkerShape::Circle,
                    });

                    let (values, std_devs): (Vec<f64>, Vec<f64>) = buckets
                        .iter()
                        .map(|bucket| match metric {
                            MetricKind::Timing => (mean(bucket), std_dev(bucket)),
                            MetricKind::Rmse => {
                                if bucket.is_empty() {
                                    return (0.0, 0.0);
                                }
                                let sq_floor = RMSE_LOG_FLOOR * RMSE_LOG_FLOOR;
                                let mean_sq = mean(bucket).max(sq_floor);
                                let rmse = mean_sq.sqrt();
                                let std_sq = std_dev(bucket);
                                let upper = (mean_sq + std_sq).max(sq_floor).sqrt();
                                let lower = (mean_sq - std_sq).max(sq_floor).sqrt();
                                (rmse, (upper - lower) / 2.0)
                            }
                        })
                        .unzip();

                    LineSeriesData {
                        label,
                        color: style.color,
                        marker: style.marker,
                        values,
                        std_devs,
                        counts: buckets.iter().map(Vec::len).collect(),
                    }
                })
                .collect();

            FacetChart {
                title: format!("Reference: {facet_label}"),
                series,
            }
        })
        .collect();

    FacetedLineChart {
        title: metric.chart_title().to_string(),
        y_label: metric.y_label().to_string(),
        bucket_labels,
        facets,
        log_y: metric.use_log_scale(),
    }
}

#[cfg(test)]
pub(crate) fn build_metric_chart(
    metric: MetricKind,
    data: &[ResultRow],
    spectra_peaks: &HashMap<i32, i32>,
    style_map: &HashMap<String, SeriesStyle>,
    references: &HashMap<String, AlgorithmReference>,
) -> FacetedLineChart {
    let labels = bucket_labels();
    let n_buckets = labels.len();

    let reference_scores = match metric {
        MetricKind::Rmse => Some(build_reference_score_index(data)),
        MetricKind::Timing => None,
    };

    let grouped = collect_bucketed_series(
        data,
        spectra_peaks,
        references,
        n_buckets,
        |row, reference| match metric {
            MetricKind::Timing => Some(row.median_time_us as f64),
            MetricKind::Rmse => {
                if row.implementation_id == reference.implementation_id {
                    return None;
                }

                let algo_refs = reference_scores
                    .as_ref()
                    .and_then(|index| index.get(&reference.implementation_id))?;
                let ref_score = *algo_refs.get(&(row.left_id, row.right_id, row.experiment_id))?;
                let diff = row.score as f64 - ref_score as f64;
                let squared_error = diff * diff;
                Some(squared_error.max(RMSE_LOG_FLOOR * RMSE_LOG_FLOOR))
            }
        },
    );

    grouped_to_facets(grouped, style_map, labels, metric)
}

pub(crate) fn build_metric_chart_from_aggregates(
    metric: MetricKind,
    rows: &[AggregatedSeriesPoint],
    style_map: &HashMap<String, SeriesStyle>,
) -> FacetedLineChart {
    type BucketStats = Option<(f64, f64, usize)>;
    type SeriesBucketStats = Vec<BucketStats>;
    type FacetSeriesBucketStats = BTreeMap<String, SeriesBucketStats>;

    let labels = bucket_labels();
    let n_buckets = labels.len();

    let mut by_facet: BTreeMap<String, FacetSeriesBucketStats> = BTreeMap::new();

    for row in rows {
        if row.bucket_index >= n_buckets || !row.value.is_finite() || !row.std_dev.is_finite() {
            continue;
        }
        let series_buckets = by_facet
            .entry(row.facet_label.clone())
            .or_default()
            .entry(row.series_label.clone())
            .or_insert_with(|| vec![None; n_buckets]);
        series_buckets[row.bucket_index] = Some((row.value, row.std_dev.max(0.0), row.count));
    }

    let facets = by_facet
        .into_iter()
        .map(|(facet_label, series_map)| {
            let series = series_map
                .into_iter()
                .map(|(label, buckets)| {
                    let style = style_map.get(&label).copied().unwrap_or(SeriesStyle {
                        color: LIBRARY_COLORS[0],
                        marker: MarkerShape::Circle,
                    });

                    let values: Vec<f64> = buckets
                        .iter()
                        .map(|entry| entry.map(|(v, _, _)| v).unwrap_or(0.0))
                        .collect();
                    let std_devs: Vec<f64> = buckets
                        .iter()
                        .map(|entry| entry.map(|(_, sd, _)| sd).unwrap_or(0.0))
                        .collect();
                    let counts: Vec<usize> = buckets
                        .iter()
                        .map(|entry| entry.map(|(_, _, c)| c).unwrap_or(0))
                        .collect();

                    LineSeriesData {
                        label,
                        color: style.color,
                        marker: style.marker,
                        values,
                        std_devs,
                        counts,
                    }
                })
                .collect();

            FacetChart {
                title: format!("Reference: {facet_label}"),
                series,
            }
        })
        .collect();

    FacetedLineChart {
        title: metric.chart_title().to_string(),
        y_label: metric.y_label().to_string(),
        bucket_labels: labels,
        facets,
        log_y: metric.use_log_scale(),
    }
}

pub(crate) fn omit_empty_buckets(chart: FacetedLineChart) -> FacetedLineChart {
    let FacetedLineChart {
        title,
        y_label,
        bucket_labels,
        facets,
        log_y,
    } = chart;

    if bucket_labels.is_empty() || facets.is_empty() {
        return FacetedLineChart {
            title,
            y_label,
            bucket_labels,
            facets,
            log_y,
        };
    }

    let keep_indices: Vec<usize> = (0..bucket_labels.len())
        .filter(|&idx| {
            facets
                .iter()
                .flat_map(|facet| facet.series.iter())
                .any(|series| series.counts.get(idx).copied().unwrap_or(0) > 0)
        })
        .collect();

    if keep_indices.len() == bucket_labels.len() {
        return FacetedLineChart {
            title,
            y_label,
            bucket_labels,
            facets,
            log_y,
        };
    }

    let filtered_labels: Vec<String> = keep_indices
        .iter()
        .map(|&idx| bucket_labels[idx].clone())
        .collect();

    let filtered_facets: Vec<FacetChart> = facets
        .into_iter()
        .map(|facet| {
            let filtered_series = facet
                .series
                .into_iter()
                .map(|series| {
                    let values = keep_indices
                        .iter()
                        .map(|&idx| series.values.get(idx).copied().unwrap_or(0.0))
                        .collect();
                    let std_devs = keep_indices
                        .iter()
                        .map(|&idx| series.std_devs.get(idx).copied().unwrap_or(0.0))
                        .collect();
                    let counts = keep_indices
                        .iter()
                        .map(|&idx| series.counts.get(idx).copied().unwrap_or(0))
                        .collect();

                    LineSeriesData {
                        label: series.label,
                        color: series.color,
                        marker: series.marker,
                        values,
                        std_devs,
                        counts,
                    }
                })
                .collect();

            FacetChart {
                title: facet.title,
                series: filtered_series,
            }
        })
        .collect();

    FacetedLineChart {
        title,
        y_label,
        bucket_labels: filtered_labels,
        facets: filtered_facets,
        log_y,
    }
}
