use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use plotters::coord::Shift;
use plotters::prelude::*;

use diesel::sql_query;
use diesel::sql_types::{Float, Integer, Text};

use crate::progress::StageProgress;
use crate::schema::{algorithms, implementations, libraries, results, spectra};

const BUCKET_BOUNDARIES: &[i32] = &[5, 9, 17, 33, 65, 129, 257, 513];

const COLORS: [RGBColor; 5] = [
    RGBColor(82, 154, 220),  // pastel blue
    RGBColor(238, 134, 62),  // pastel orange
    RGBColor(82, 182, 96),   // pastel green
    RGBColor(210, 78, 78),   // pastel red
    RGBColor(146, 112, 184), // pastel lavender
];

fn series_label(algo: &str, lib: &str) -> String {
    format!("{algo} ({lib})")
}

fn algorithm_uses_match_count_parity(algorithm: &str) -> bool {
    !algorithm.starts_with("EntropySimilarity")
}

#[derive(Clone, Debug)]
struct AlgorithmReference {
    implementation_id: i32,
    label: String,
}

#[derive(Clone, Debug)]
struct ResultRow {
    implementation_id: i32,
    is_reference: bool,
    score: f32,
    median_time_us: f32,
    algo_name: String,
    lib_name: String,
    left_id: i32,
    right_id: i32,
    experiment_id: i32,
}

#[derive(Clone, Debug)]
struct LineSeriesData {
    label: String,
    color: RGBColor,
    values: Vec<f64>,
    std_devs: Vec<f64>,
}

#[derive(Clone, Debug)]
struct FacetChart {
    title: String,
    series: Vec<LineSeriesData>,
}

#[derive(Clone, Debug)]
struct FacetedLineChart {
    title: String,
    y_label: String,
    bucket_labels: Vec<String>,
    facets: Vec<FacetChart>,
    log_y: bool,
}

fn bucket_index(max_peaks: i32) -> Option<usize> {
    if max_peaks < BUCKET_BOUNDARIES[0] {
        return None;
    }
    (0..BUCKET_BOUNDARIES.len())
        .rev()
        .find(|&i| max_peaks >= BUCKET_BOUNDARIES[i])
}

fn bucket_labels() -> Vec<String> {
    BUCKET_BOUNDARIES
        .windows(2)
        .map(|w| format!("{}\u{2013}{}", w[0], w[1] - 1))
        .chain(std::iter::once(format!(
            "{}+",
            BUCKET_BOUNDARIES
                .last()
                .expect("bucket boundaries not empty")
        )))
        .collect()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance =
        values.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

fn build_color_map(data: &[ResultRow]) -> HashMap<String, RGBColor> {
    let mut labels: Vec<String> = data
        .iter()
        .map(|r| series_label(&r.algo_name, &r.lib_name))
        .collect();
    labels.sort();
    labels.dedup();
    labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| (label, COLORS[i % COLORS.len()]))
        .collect()
}

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(p) = progress.as_deref_mut() {
        p.set_substep(message);
    } else {
        eprintln!("{message}");
    }
}

fn load_spectra_peaks(conn: &mut SqliteConnection) -> HashMap<i32, i32> {
    spectra::table
        .order(spectra::id.asc())
        .select((spectra::id, spectra::num_peaks))
        .load::<(i32, i32)>(conn)
        .expect("failed to load spectra peaks")
        .into_iter()
        .collect()
}

fn load_result_data(conn: &mut SqliteConnection) -> Vec<ResultRow> {
    results::table
        .inner_join(
            implementations::table
                .inner_join(algorithms::table)
                .inner_join(libraries::table),
        )
        .order((
            algorithms::name.asc(),
            libraries::name.asc(),
            results::experiment_id.asc(),
            results::left_id.asc(),
            results::right_id.asc(),
        ))
        .select((
            implementations::id,
            implementations::is_reference,
            results::score,
            results::median_time_us,
            algorithms::name,
            libraries::name,
            results::left_id,
            results::right_id,
            results::experiment_id,
        ))
        .load::<(i32, bool, f32, f32, String, String, i32, i32, i32)>(conn)
        .expect("failed to load result data")
        .into_iter()
        .map(
            |(
                implementation_id,
                is_reference,
                score,
                median_time_us,
                algo_name,
                lib_name,
                left_id,
                right_id,
                experiment_id,
            )| ResultRow {
                implementation_id,
                is_reference,
                score,
                median_time_us,
                algo_name,
                lib_name,
                left_id,
                right_id,
                experiment_id,
            },
        )
        .collect()
}

fn algorithm_references(data: &[ResultRow]) -> HashMap<String, AlgorithmReference> {
    let mut refs: HashMap<String, AlgorithmReference> = HashMap::new();

    for row in data.iter().filter(|r| r.is_reference) {
        let label = series_label(&row.algo_name, &row.lib_name);
        match refs.entry(row.algo_name.clone()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(AlgorithmReference {
                    implementation_id: row.implementation_id,
                    label,
                });
            }
            std::collections::hash_map::Entry::Occupied(entry) => {
                if entry.get().implementation_id != row.implementation_id {
                    panic!(
                        "algorithm '{}' has multiple reference implementations in results",
                        row.algo_name
                    );
                }
            }
        }
    }

    refs
}

fn grouped_to_facets(
    grouped: HashMap<(String, String), Vec<Vec<f64>>>,
    color_map: &HashMap<String, RGBColor>,
    bucket_labels: Vec<String>,
    title: &str,
    y_label: &str,
    log_y: bool,
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
                .map(|(label, buckets)| LineSeriesData {
                    color: color_map.get(&label).copied().unwrap_or(COLORS[0]),
                    label,
                    values: buckets.iter().map(|b| mean(b)).collect(),
                    std_devs: buckets.iter().map(|b| std_dev(b)).collect(),
                })
                .collect();

            FacetChart {
                title: format!("Reference: {facet_label}"),
                series,
            }
        })
        .collect();

    FacetedLineChart {
        title: title.to_string(),
        y_label: y_label.to_string(),
        bucket_labels,
        facets,
        log_y,
    }
}

fn build_timing_chart(
    data: &[ResultRow],
    spectra_peaks: &HashMap<i32, i32>,
    color_map: &HashMap<String, RGBColor>,
    references: &HashMap<String, AlgorithmReference>,
) -> FacetedLineChart {
    let labels = bucket_labels();
    let n_buckets = labels.len();

    let mut grouped: HashMap<(String, String), Vec<Vec<f64>>> = HashMap::new();

    for row in data {
        let Some(reference) = references.get(&row.algo_name) else {
            continue;
        };

        if let (Some(&lp), Some(&rp)) = (
            spectra_peaks.get(&row.left_id),
            spectra_peaks.get(&row.right_id),
        ) && let Some(bi) = bucket_index(lp.max(rp))
        {
            grouped
                .entry((
                    reference.label.clone(),
                    series_label(&row.algo_name, &row.lib_name),
                ))
                .or_insert_with(|| vec![Vec::new(); n_buckets])[bi]
                .push(row.median_time_us as f64);
        }
    }

    grouped_to_facets(
        grouped,
        color_map,
        labels,
        "Timing by Peak Count",
        "Mean time (µs)",
        true,
    )
}

fn build_mse_chart(
    data: &[ResultRow],
    spectra_peaks: &HashMap<i32, i32>,
    color_map: &HashMap<String, RGBColor>,
    references: &HashMap<String, AlgorithmReference>,
) -> FacetedLineChart {
    let labels = bucket_labels();
    let n_buckets = labels.len();

    let mut reference_scores: HashMap<String, HashMap<(i32, i32, i32), f32>> = HashMap::new();
    for row in data.iter().filter(|r| r.is_reference) {
        reference_scores
            .entry(row.algo_name.clone())
            .or_default()
            .insert((row.left_id, row.right_id, row.experiment_id), row.score);
    }

    let mut grouped: HashMap<(String, String), Vec<Vec<f64>>> = HashMap::new();

    for row in data {
        if row.is_reference {
            continue;
        }

        let Some(reference) = references.get(&row.algo_name) else {
            continue;
        };

        if let (Some(algo_refs), Some(&lp), Some(&rp)) = (
            reference_scores.get(&row.algo_name),
            spectra_peaks.get(&row.left_id),
            spectra_peaks.get(&row.right_id),
        ) && let Some(&ref_score) =
            algo_refs.get(&(row.left_id, row.right_id, row.experiment_id))
            && let Some(bi) = bucket_index(lp.max(rp))
        {
            let diff = row.score as f64 - ref_score as f64;
            grouped
                .entry((
                    reference.label.clone(),
                    series_label(&row.algo_name, &row.lib_name),
                ))
                .or_insert_with(|| vec![Vec::new(); n_buckets])[bi]
                .push(diff * diff);
        }
    }

    grouped_to_facets(
        grouped,
        color_map,
        labels,
        "MSE vs Reference by Peak Count",
        "MSE",
        true,
    )
}

fn render_facet_chart<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    chart: &FacetedLineChart,
    facet: &FacetChart,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB::ErrorType: 'static,
{
    let n_buckets = chart.bucket_labels.len();
    if n_buckets == 0 || facet.series.is_empty() {
        return Ok(());
    }

    let all_upper: Vec<f64> = facet
        .series
        .iter()
        .flat_map(|s| s.values.iter().zip(s.std_devs.iter()))
        .filter_map(|(v, sd)| {
            let upper = v + sd;
            if upper > 0.0 { Some(upper) } else { None }
        })
        .collect();
    if all_upper.is_empty() {
        return Ok(());
    }

    let labels = chart.bucket_labels.clone();
    let x_label_fmt = move |x: &f64| {
        let idx = x.round() as i64;
        if idx >= 0 && (idx as usize) < labels.len() && (*x - idx as f64).abs() < 0.1 {
            labels[idx as usize].clone()
        } else {
            String::new()
        }
    };

    let x_range = -0.5f64..(n_buckets as f64 - 0.5);

    let min_positive = facet
        .series
        .iter()
        .flat_map(|s| s.values.iter().copied())
        .filter(|v| *v > 0.0)
        .min_by(|a, b| a.total_cmp(b));

    let use_log = chart.log_y && min_positive.is_some();

    if use_log {
        let min_positive = min_positive.expect("checked above");
        let max_with_err = all_upper.iter().copied().fold(0.0f64, f64::max);

        let y_min = 10f64.powi(min_positive.log10().floor() as i32 - 1);
        let y_max = 10f64.powi(max_with_err.log10().ceil() as i32 + 1);

        let mut ctx = ChartBuilder::on(area)
            .caption(&facet.title, ("sans-serif", 20))
            .margin(12)
            .margin_right(24)
            .x_label_area_size(36)
            .y_label_area_size(72)
            .build_cartesian_2d(x_range, (y_min..y_max).log_scale())?;

        ctx.configure_mesh()
            .disable_x_mesh()
            .bold_line_style(BLACK.mix(0.35))
            .light_line_style(BLACK.mix(0.15))
            .x_labels(n_buckets)
            .x_label_formatter(&x_label_fmt)
            .x_desc("Number of peaks")
            .y_desc(&chart.y_label)
            .y_label_formatter(&|v| format!("{:.0e}", v))
            .draw()?;

        for series in &facet.series {
            let color = series.color;
            let points: Vec<(f64, f64)> = series
                .values
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| if v > 0.0 { Some((i as f64, v)) } else { None })
                .collect();
            if points.is_empty() {
                continue;
            }

            ctx.draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))?
                .label(series.label.clone())
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 16, y)], color.stroke_width(2))
                });

            ctx.draw_series(
                points
                    .iter()
                    .map(|(x, y)| Circle::new((*x, *y), 3, color.filled())),
            )?;

            let whiskers: Vec<PathElement<(f64, f64)>> = series
                .values
                .iter()
                .zip(series.std_devs.iter())
                .enumerate()
                .filter_map(|(idx, (&v, &sd))| {
                    if v <= 0.0 || sd <= 0.0 {
                        return None;
                    }
                    let lower = (v - sd).max(y_min);
                    let upper = v + sd;
                    let x = idx as f64;
                    let cap_half = 0.15;
                    Some(vec![
                        PathElement::new(vec![(x, lower), (x, upper)], BLACK),
                        PathElement::new(vec![(x - cap_half, upper), (x + cap_half, upper)], BLACK),
                        PathElement::new(vec![(x - cap_half, lower), (x + cap_half, lower)], BLACK),
                    ])
                })
                .flatten()
                .collect();
            ctx.draw_series(whiskers)?;
        }

        ctx.configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
    } else {
        let y_max = all_upper.iter().copied().fold(0.0f64, f64::max).max(1.0) * 1.1;

        let mut ctx = ChartBuilder::on(area)
            .caption(&facet.title, ("sans-serif", 20))
            .margin(12)
            .margin_right(24)
            .x_label_area_size(36)
            .y_label_area_size(72)
            .build_cartesian_2d(x_range, 0f64..y_max)?;

        ctx.configure_mesh()
            .disable_x_mesh()
            .x_labels(n_buckets)
            .x_label_formatter(&x_label_fmt)
            .x_desc("Number of peaks")
            .y_desc(&chart.y_label)
            .y_label_formatter(&|v| format!("{:.1e}", v))
            .draw()?;

        for series in &facet.series {
            let color = series.color;
            let points: Vec<(f64, f64)> = series
                .values
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64, v))
                .collect();

            ctx.draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))?
                .label(series.label.clone())
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 16, y)], color.stroke_width(2))
                });

            ctx.draw_series(
                points
                    .iter()
                    .map(|(x, y)| Circle::new((*x, *y), 3, color.filled())),
            )?;

            let whiskers: Vec<PathElement<(f64, f64)>> = series
                .values
                .iter()
                .zip(series.std_devs.iter())
                .enumerate()
                .filter_map(|(idx, (&v, &sd))| {
                    if sd <= 0.0 {
                        return None;
                    }
                    let lower = (v - sd).max(0.0);
                    let upper = v + sd;
                    let x = idx as f64;
                    let cap_half = 0.15;
                    Some(vec![
                        PathElement::new(vec![(x, lower), (x, upper)], BLACK),
                        PathElement::new(vec![(x - cap_half, upper), (x + cap_half, upper)], BLACK),
                        PathElement::new(vec![(x - cap_half, lower), (x + cap_half, lower)], BLACK),
                    ])
                })
                .flatten()
                .collect();
            ctx.draw_series(whiskers)?;
        }

        ctx.configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
    }

    Ok(())
}

fn render_faceted_line_chart(
    chart: &FacetedLineChart,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if chart.facets.is_empty() || chart.bucket_labels.is_empty() {
        return Ok(());
    }

    let n_facets = chart.facets.len();
    let cols = ((n_facets as f64).sqrt().ceil() as usize).max(1);
    let rows = n_facets.div_ceil(cols);

    let width = 920u32 * cols as u32;
    let height = 420u32 * rows as u32;

    let root = SVGBackend::new(path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let titled = root.titled(&chart.title, ("sans-serif", 24))?;
    let areas = titled.split_evenly((rows, cols));

    for (index, area) in areas.iter().enumerate() {
        if let Some(facet) = chart.facets.get(index) {
            render_facet_chart(area, chart, facet)?;
        } else {
            area.fill(&WHITE)?;
        }
    }

    titled.present()?;
    Ok(())
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
    #[diesel(sql_type = Float)]
    rust_score: f32,
    #[diesel(sql_type = Integer)]
    rust_matches: i32,
    #[diesel(sql_type = Float)]
    reference_score: f32,
    #[diesel(sql_type = Integer)]
    reference_matches: i32,
}

fn compare_results(conn: &mut SqliteConnection, mut progress: Option<&mut dyn StageProgress>) {
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
         JOIN libraries rl ON ri.library_id = rl.id
         JOIN implementations refi ON refi.algorithm_id = ri.algorithm_id AND refi.is_reference = 1
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

    let mut max_score_diff: f32 = 0.0;
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

        sum_sq_score += (score_diff as f64) * (score_diff as f64);

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

pub fn run(conn: &mut SqliteConnection) {
    run_with_progress(conn, None);
}

pub fn run_with_progress(conn: &mut SqliteConnection, progress: Option<&mut dyn StageProgress>) {
    run_to_dir_with_progress(conn, Path::new("output"), progress);
}

pub fn run_to_dir(conn: &mut SqliteConnection, output_dir: &Path) {
    run_to_dir_with_progress(conn, output_dir, None);
}

pub fn run_to_dir_with_progress(
    conn: &mut SqliteConnection,
    output_dir: &Path,
    mut progress: Option<&mut dyn StageProgress>,
) {
    emit(&mut progress, "[report] Generating charts");
    fs::create_dir_all(output_dir).expect("failed to create output directory");

    emit(&mut progress, "[report] Loading result data");
    let spectra_peaks = load_spectra_peaks(conn);
    let data = load_result_data(conn);
    let references = algorithm_references(&data);
    let color_map = build_color_map(&data);

    let timing_chart = build_timing_chart(&data, &spectra_peaks, &color_map, &references);
    if !timing_chart.facets.is_empty() {
        emit(&mut progress, "[report] Rendering timing chart");
        let timing_path = output_dir.join("timing_by_peaks.svg");
        render_faceted_line_chart(&timing_chart, timing_path.to_string_lossy().as_ref())
            .expect("failed to render timing chart");
        emit(
            &mut progress,
            &format!("[report] Written {}", timing_path.display()),
        );
    }

    let mse_chart = build_mse_chart(&data, &spectra_peaks, &color_map, &references);
    if !mse_chart.facets.is_empty() {
        emit(&mut progress, "[report] Rendering MSE chart");
        let mse_path = output_dir.join("mse_score_by_peaks.svg");
        render_faceted_line_chart(&mse_chart, mse_path.to_string_lossy().as_ref())
            .expect("failed to render MSE chart");
        emit(
            &mut progress,
            &format!("[report] Written {}", mse_path.display()),
        );
    }

    compare_results(conn, progress);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row(implementation_id: i32, is_reference: bool, algo: &str, lib: &str) -> ResultRow {
        ResultRow {
            implementation_id,
            is_reference,
            score: 0.0,
            median_time_us: 0.0,
            algo_name: algo.to_string(),
            lib_name: lib.to_string(),
            left_id: 1,
            right_id: 1,
            experiment_id: 1,
        }
    }

    #[test]
    fn bucket_boundaries_and_labels_are_stable() {
        assert_eq!(bucket_index(4), None);
        assert_eq!(bucket_index(5), Some(0));
        assert_eq!(bucket_index(8), Some(0));
        assert_eq!(bucket_index(9), Some(1));
        assert_eq!(bucket_index(513), Some(BUCKET_BOUNDARIES.len() - 1));

        let labels = bucket_labels();
        assert_eq!(labels.first().expect("missing first label"), "5–8");
        assert_eq!(labels.last().expect("missing last label"), "513+");
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
    fn build_color_map_is_independent_of_input_order() {
        let data_a = vec![
            ResultRow {
                score: 0.1,
                median_time_us: 1.0,
                ..sample_row(1, true, "CosineHungarian", "matchms")
            },
            ResultRow {
                score: 0.2,
                median_time_us: 2.0,
                ..sample_row(2, false, "CosineHungarian", "mass-spectrometry-traits")
            },
            ResultRow {
                score: 0.3,
                median_time_us: 3.0,
                ..sample_row(3, true, "EntropySimilarityWeighted", "ms_entropy")
            },
        ];
        let data_b = vec![
            ResultRow {
                score: 0.3,
                median_time_us: 3.0,
                ..sample_row(3, true, "EntropySimilarityWeighted", "ms_entropy")
            },
            ResultRow {
                score: 0.2,
                median_time_us: 2.0,
                ..sample_row(2, false, "CosineHungarian", "mass-spectrometry-traits")
            },
            ResultRow {
                score: 0.1,
                median_time_us: 1.0,
                ..sample_row(1, true, "CosineHungarian", "matchms")
            },
        ];

        assert_eq!(build_color_map(&data_a), build_color_map(&data_b));
    }

    #[test]
    fn references_are_driven_by_reference_flag() {
        let data = vec![
            ResultRow {
                score: 0.9,
                median_time_us: 10.0,
                ..sample_row(10, true, "CosineHungarian", "matchms")
            },
            ResultRow {
                score: 0.8,
                median_time_us: 8.0,
                ..sample_row(11, false, "CosineHungarian", "mass-spectrometry-traits")
            },
            ResultRow {
                score: 0.7,
                median_time_us: 12.0,
                ..sample_row(20, true, "EntropySimilarityWeighted", "ms_entropy")
            },
            ResultRow {
                score: 0.6,
                median_time_us: 9.0,
                ..sample_row(
                    21,
                    false,
                    "EntropySimilarityWeighted",
                    "mass-spectrometry-traits",
                )
            },
        ];

        let refs = algorithm_references(&data);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs["CosineHungarian"].implementation_id, 10);
        assert_eq!(refs["CosineHungarian"].label, "CosineHungarian (matchms)");
        assert_eq!(refs["EntropySimilarityWeighted"].implementation_id, 20);
        assert_eq!(
            refs["EntropySimilarityWeighted"].label,
            "EntropySimilarityWeighted (ms_entropy)"
        );
    }

    #[test]
    fn mse_chart_is_grouped_by_reference_label() {
        let data = vec![
            ResultRow {
                score: 0.9,
                median_time_us: 10.0,
                right_id: 2,
                ..sample_row(10, true, "CosineHungarian", "matchms")
            },
            ResultRow {
                score: 0.8,
                median_time_us: 8.0,
                right_id: 2,
                ..sample_row(11, false, "CosineHungarian", "mass-spectrometry-traits")
            },
            ResultRow {
                score: 0.7,
                median_time_us: 12.0,
                right_id: 2,
                ..sample_row(20, true, "EntropySimilarityWeighted", "ms_entropy")
            },
            ResultRow {
                score: 0.6,
                median_time_us: 9.0,
                right_id: 2,
                ..sample_row(
                    21,
                    false,
                    "EntropySimilarityWeighted",
                    "mass-spectrometry-traits",
                )
            },
        ];

        let references = algorithm_references(&data);
        let colors = build_color_map(&data);
        let peaks: HashMap<i32, i32> = HashMap::from([(1, 10), (2, 12)]);

        let chart = build_mse_chart(&data, &peaks, &colors, &references);
        assert_eq!(chart.facets.len(), 2);
        assert!(
            chart
                .facets
                .iter()
                .any(|f| f.title == "Reference: CosineHungarian (matchms)")
        );
        assert!(
            chart
                .facets
                .iter()
                .any(|f| f.title == "Reference: EntropySimilarityWeighted (ms_entropy)")
        );
    }
}
