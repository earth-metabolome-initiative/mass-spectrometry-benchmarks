use std::collections::BTreeMap;
use std::collections::HashMap;

use statrs::distribution::{ContinuousCDF, StudentsT};

use super::render::{
    AggregatedSeriesPoint, BUCKET_BOUNDARIES, FacetChart, FacetedLineChart, LIBRARY_COLORS,
    LineSeriesData, MarkerShape, SeriesStyle,
};

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

pub(crate) fn bucket_index(pair_peaks: i32) -> Option<usize> {
    if pair_peaks < BUCKET_BOUNDARIES[0] {
        return None;
    }

    (0..BUCKET_BOUNDARIES.len())
        .rev()
        .find(|&i| pair_peaks >= BUCKET_BOUNDARIES[i])
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
        x_label: "Min number of peaks".to_string(),
        y_label: metric.y_label().to_string(),
        bucket_labels: labels,
        facets,
        log_y: metric.use_log_scale(),
        square_facets: false,
    }
}

// --- Correlation aggregation ---

use super::data::CorrelationRow;

const N_TANIMOTO_BINS: usize = 10;

pub(crate) fn tanimoto_bin_labels() -> Vec<String> {
    (0..N_TANIMOTO_BINS)
        .map(|i| {
            let lo = i as f64 * 0.1;
            let hi = lo + 0.1;
            format!("{lo:.1}-{hi:.1}")
        })
        .collect()
}

fn tanimoto_bin_index(score: f64) -> usize {
    let idx = (score * N_TANIMOTO_BINS as f64).floor() as usize;
    idx.min(N_TANIMOTO_BINS - 1)
}

#[derive(Debug, Clone)]
pub(crate) struct CorrelationStats {
    pub(crate) fingerprint_algorithm: String,
    pub(crate) series_label: String,
    pub(crate) pearson_r: f64,
    pub(crate) pearson_p: f64,
    pub(crate) spearman_rho: f64,
    pub(crate) spearman_p: f64,
    pub(crate) n_pairs: usize,
}

fn correlation_p_value(r: f64, n: usize) -> f64 {
    if n < 3 || !r.is_finite() {
        return 1.0;
    }
    let r_clamped = r.clamp(-1.0 + 1e-15, 1.0 - 1e-15);
    let df = (n - 2) as f64;
    let t = r_clamped * (df / (1.0 - r_clamped * r_clamped)).sqrt();
    let dist = StudentsT::new(0.0, 1.0, df).expect("valid Student's t distribution");
    2.0 * (1.0 - dist.cdf(t.abs()))
}

pub(crate) fn build_correlation_chart(
    rows: &[CorrelationRow],
    style_map: &HashMap<String, SeriesStyle>,
) -> (FacetedLineChart, Vec<CorrelationStats>) {
    let labels = tanimoto_bin_labels();

    // Group by (fingerprint_algorithm, series_label, library_name)
    type Key = (String, String, String);
    let mut groups: BTreeMap<Key, Vec<(f64, f64)>> = BTreeMap::new();

    for row in rows {
        if !row.spectral_score.is_finite() || !row.tanimoto_score.is_finite() {
            continue;
        }
        let key = (
            row.fingerprint_algorithm.clone(),
            row.series_label.clone(),
            row.library_name.clone(),
        );
        groups
            .entry(key)
            .or_default()
            .push((row.tanimoto_score, row.spectral_score));
    }

    // Build per-facet (fingerprint algorithm) per-series binned data
    type BinAcc = Vec<Vec<f64>>; // bins[bin_idx] = list of spectral scores
    // facet_name -> series_label -> bins
    let mut by_facet: BTreeMap<String, BTreeMap<String, BinAcc>> = BTreeMap::new();
    let mut stats_list: Vec<CorrelationStats> = Vec::new();

    for ((fp_algo, series_label, _library_name), pairs) in &groups {
        let n = pairs.len();

        let pearson_r = pearson(pairs);
        let spearman_rho = spearman(pairs);

        let pearson_p = correlation_p_value(pearson_r, n);
        let spearman_p = correlation_p_value(spearman_rho, n);

        stats_list.push(CorrelationStats {
            fingerprint_algorithm: fp_algo.clone(),
            series_label: series_label.clone(),
            pearson_r,
            pearson_p,
            spearman_rho,
            spearman_p,
            n_pairs: n,
        });

        // Bin by Tanimoto score
        let mut bins: Vec<Vec<f64>> = vec![Vec::new(); N_TANIMOTO_BINS];
        for &(tanimoto, spectral) in pairs {
            let idx = tanimoto_bin_index(tanimoto);
            bins[idx].push(spectral);
        }

        by_facet
            .entry(fp_algo.clone())
            .or_default()
            .insert(series_label.clone(), bins);
    }

    let facets: Vec<FacetChart> = by_facet
        .into_iter()
        .map(|(fp_algo, by_series)| {
            let series: Vec<LineSeriesData> = by_series
                .into_iter()
                .map(|(label, bins)| {
                    let style = style_map.get(&label).copied().unwrap_or(SeriesStyle {
                        color: LIBRARY_COLORS[0],
                        marker: MarkerShape::Circle,
                    });

                    let values: Vec<f64> = bins
                        .iter()
                        .map(|bin| {
                            if bin.is_empty() {
                                0.0
                            } else {
                                bin.iter().sum::<f64>() / bin.len() as f64
                            }
                        })
                        .collect();
                    let std_devs: Vec<f64> = bins
                        .iter()
                        .map(|bin| {
                            if bin.len() < 2 {
                                return 0.0;
                            }
                            let m = bin.iter().sum::<f64>() / bin.len() as f64;
                            let var = bin.iter().map(|v| (v - m) * (v - m)).sum::<f64>()
                                / (bin.len() - 1) as f64;
                            var.max(0.0).sqrt()
                        })
                        .collect();
                    let counts: Vec<usize> = bins.iter().map(|bin| bin.len()).collect();

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
                title: fp_algo,
                series,
            }
        })
        .collect();

    let chart = FacetedLineChart {
        title: "Spectral Similarity vs Structural Similarity (Tanimoto)".to_string(),
        x_label: "Tanimoto similarity".to_string(),
        y_label: "Mean spectral similarity".to_string(),
        bucket_labels: labels,
        facets,
        log_y: false,
        square_facets: true,
    };

    (chart, stats_list)
}

fn pearson(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let sum_x: f64 = pairs.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = pairs.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = pairs.iter().map(|(x, y)| x * y).sum();
    let sum_x2: f64 = pairs.iter().map(|(x, _)| x * x).sum();
    let sum_y2: f64 = pairs.iter().map(|(_, y)| y * y).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denom = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
    if denom < 1e-15 {
        return 0.0;
    }
    numerator / denom
}

fn spearman(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len();
    if n < 2 {
        return 0.0;
    }
    let ranks_x = rank(&pairs.iter().map(|(x, _)| *x).collect::<Vec<_>>());
    let ranks_y = rank(&pairs.iter().map(|(_, y)| *y).collect::<Vec<_>>());
    let rank_pairs: Vec<(f64, f64)> = ranks_x.into_iter().zip(ranks_y).collect();
    pearson(&rank_pairs)
}

fn rank(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.total_cmp(&b.1));

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && indexed[j].1 == indexed[i].1 {
            j += 1;
        }
        let avg_rank = (i + j + 1) as f64 / 2.0;
        for item in &indexed[i..j] {
            ranks[item.0] = avg_rank;
        }
        i = j;
    }
    ranks
}

pub(crate) fn omit_empty_buckets(chart: FacetedLineChart) -> FacetedLineChart {
    let FacetedLineChart {
        title,
        x_label,
        y_label,
        bucket_labels,
        facets,
        log_y,
        square_facets,
    } = chart;

    if bucket_labels.is_empty() || facets.is_empty() {
        return FacetedLineChart {
            title,
            x_label,
            y_label,
            bucket_labels,
            facets,
            log_y,
            square_facets,
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
            x_label,
            y_label,
            bucket_labels,
            facets,
            log_y,
            square_facets,
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
        x_label,
        y_label,
        bucket_labels: filtered_labels,
        facets: filtered_facets,
        log_y,
        square_facets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_p_value_perfect_correlation() {
        // r=1.0 with many points should give p ≈ 0
        let p = correlation_p_value(0.999, 100);
        assert!(p < 1e-10, "p={p}, expected near 0 for strong correlation");
    }

    #[test]
    fn correlation_p_value_no_correlation() {
        // r≈0 should give p ≈ 1
        let p = correlation_p_value(0.001, 100);
        assert!(p > 0.9, "p={p}, expected near 1 for no correlation");
    }

    #[test]
    fn correlation_p_value_small_sample() {
        // n < 3 should return 1.0
        assert_eq!(correlation_p_value(0.5, 2), 1.0);
        assert_eq!(correlation_p_value(0.5, 0), 1.0);
    }

    #[test]
    fn pearson_perfect_positive() {
        let pairs: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, i as f64 * 2.0 + 1.0)).collect();
        let r = pearson(&pairs);
        assert!((r - 1.0).abs() < 1e-10, "r={r}, expected 1.0");
    }

    #[test]
    fn spearman_monotone_relationship() {
        // Perfect monotone nonlinear relationship
        let pairs: Vec<(f64, f64)> = (1..=10).map(|i| (i as f64, (i as f64).powi(3))).collect();
        let rho = spearman(&pairs);
        assert!((rho - 1.0).abs() < 1e-10, "rho={rho}, expected 1.0");
    }
}
