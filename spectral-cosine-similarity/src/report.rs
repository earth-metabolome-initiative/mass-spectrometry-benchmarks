use std::collections::HashMap;
use std::fs;
use std::path::Path;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use plotters::prelude::*;

use diesel::sql_query;
use diesel::sql_types::{Float, Integer};

use crate::schema::{algorithms, implementations, libraries, results, spectra};

const REFERENCE_ALGO: &str = "CosineHungarian";
const REFERENCE_LIB: &str = "matchms";
const BUCKET_BOUNDARIES: &[i32] = &[5, 9, 17, 33, 65, 129, 257, 513];
const BAR_CORNER_RADIUS: f64 = 3.0;

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

struct ChartSeries {
    label: String,
    color: RGBColor,
    values: Vec<f64>,
    std_devs: Vec<f64>,
}

struct GroupedBarChart {
    title: String,
    y_label: String,
    bucket_labels: Vec<String>,
    series: Vec<ChartSeries>,
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
            BUCKET_BOUNDARIES.last().unwrap()
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

struct ResultRow {
    score: f32,
    median_time_us: f32,
    algo_name: String,
    lib_name: String,
    left_id: i32,
    right_id: i32,
    experiment_id: i32,
}

/// Build a stable label -> color mapping from all result data.
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

/// Load peak counts for spectra: spectrum_id -> num_peaks
fn load_spectra_peaks(conn: &mut SqliteConnection) -> HashMap<i32, i32> {
    spectra::table
        .select((spectra::id, spectra::num_peaks))
        .load::<(i32, i32)>(conn)
        .expect("failed to load spectra peaks")
        .into_iter()
        .collect()
}

/// Load result data joined with implementations -> algorithms + libraries
fn load_result_data(conn: &mut SqliteConnection) -> Vec<ResultRow> {
    results::table
        .inner_join(
            implementations::table
                .inner_join(algorithms::table)
                .inner_join(libraries::table),
        )
        .select((
            results::score,
            results::median_time_us,
            algorithms::name,
            libraries::name,
            results::left_id,
            results::right_id,
            results::experiment_id,
        ))
        .load::<(f32, f32, String, String, i32, i32, i32)>(conn)
        .expect("failed to load result data")
        .into_iter()
        .map(
            |(score, median_time_us, algo_name, lib_name, left_id, right_id, experiment_id)| {
                ResultRow {
                    score,
                    median_time_us,
                    algo_name,
                    lib_name,
                    left_id,
                    right_id,
                    experiment_id,
                }
            },
        )
        .collect()
}

fn build_timing_chart(
    data: &[ResultRow],
    spectra_peaks: &HashMap<i32, i32>,
    color_map: &HashMap<String, RGBColor>,
) -> GroupedBarChart {
    let labels = bucket_labels();
    let n_buckets = labels.len();

    let mut grouped: HashMap<(String, String), Vec<Vec<f64>>> = HashMap::new();

    for row in data {
        if let (Some(&lp), Some(&rp)) = (
            spectra_peaks.get(&row.left_id),
            spectra_peaks.get(&row.right_id),
        ) && let Some(bi) = bucket_index(lp.max(rp))
        {
            grouped
                .entry((row.algo_name.clone(), row.lib_name.clone()))
                .or_insert_with(|| vec![Vec::new(); n_buckets])[bi]
                .push(row.median_time_us as f64);
        }
    }

    let mut keys: Vec<_> = grouped.keys().cloned().collect();
    keys.sort();

    let series = keys
        .into_iter()
        .map(|key| {
            let buckets = &grouped[&key];
            let values: Vec<f64> = buckets.iter().map(|v| mean(v)).collect();
            let std_devs: Vec<f64> = buckets.iter().map(|v| std_dev(v)).collect();
            let label = series_label(&key.0, &key.1);
            let color = color_map.get(&label).copied().unwrap_or(COLORS[0]);
            ChartSeries {
                label,
                color,
                values,
                std_devs,
            }
        })
        .collect();

    GroupedBarChart {
        title: "Mean Timing by Peak Count".to_string(),
        y_label: "Mean time (\u{b5}s)".to_string(),
        bucket_labels: labels,
        series,
        log_y: false,
    }
}

fn build_mse_chart(
    data: &[ResultRow],
    spectra_peaks: &HashMap<i32, i32>,
    color_map: &HashMap<String, RGBColor>,
) -> GroupedBarChart {
    let labels = bucket_labels();
    let n_buckets = labels.len();

    // Reference scores keyed by (left_id, right_id, experiment_id)
    let ref_scores: HashMap<(i32, i32, i32), f32> = data
        .iter()
        .filter(|r| r.algo_name == REFERENCE_ALGO && r.lib_name == REFERENCE_LIB)
        .map(|r| ((r.left_id, r.right_id, r.experiment_id), r.score))
        .collect();

    let mut grouped: HashMap<(String, String), Vec<Vec<f64>>> = HashMap::new();

    for row in data {
        // Skip the reference implementation itself
        if row.algo_name == REFERENCE_ALGO && row.lib_name == REFERENCE_LIB {
            continue;
        }
        if let (Some(&ref_score), Some(&lp), Some(&rp)) = (
            ref_scores.get(&(row.left_id, row.right_id, row.experiment_id)),
            spectra_peaks.get(&row.left_id),
            spectra_peaks.get(&row.right_id),
        ) && let Some(bi) = bucket_index(lp.max(rp))
        {
            let diff = row.score as f64 - ref_score as f64;
            grouped
                .entry((row.algo_name.clone(), row.lib_name.clone()))
                .or_insert_with(|| vec![Vec::new(); n_buckets])[bi]
                .push(diff * diff);
        }
    }

    let mut keys: Vec<_> = grouped.keys().cloned().collect();
    keys.sort();

    let series = keys
        .into_iter()
        .map(|key| {
            let buckets = &grouped[&key];
            let values: Vec<f64> = buckets.iter().map(|sq_errors| mean(sq_errors)).collect();
            let std_devs: Vec<f64> = buckets.iter().map(|v| std_dev(v)).collect();
            let label = series_label(&key.0, &key.1);
            let color = color_map.get(&label).copied().unwrap_or(COLORS[0]);
            ChartSeries {
                label,
                color,
                values,
                std_devs,
            }
        })
        .collect();

    GroupedBarChart {
        title: "MSE vs Reference by Peak Count".to_string(),
        y_label: "MSE".to_string(),
        bucket_labels: labels,
        series,
        log_y: false,
    }
}

/// Draw bars and error whiskers on any chart context.
macro_rules! draw_chart_content {
    ($ctx:expr, $chart:expr, $y_floor:expr) => {{
        let n_series = $chart.series.len();
        let group_width = 0.8;
        let bar_width = group_width / n_series as f64;

        for (si, series) in $chart.series.iter().enumerate() {
            let color = series.color;

            let bars: Vec<_> = series
                .values
                .iter()
                .enumerate()
                .filter(|(_, v)| **v > $y_floor)
                .map(|(bi, &val)| {
                    let x0 = bi as f64 - group_width / 2.0 + si as f64 * bar_width;
                    let x1 = x0 + bar_width * 0.9;
                    Rectangle::new([(x0, $y_floor), (x1, val)], color.filled())
                })
                .collect();

            $ctx.draw_series(bars)?
                .label(&series.label)
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 15, y + 5)], color.filled())
                });

            // Error whiskers (symmetric on linear, upper-only on log)
            let whiskers: Vec<PathElement<(f64, f64)>> = series
                .values
                .iter()
                .zip(series.std_devs.iter())
                .enumerate()
                .filter(|(_, (v, _))| **v > $y_floor)
                .flat_map(|(bi, (&val, &sd))| {
                    if sd <= 0.0 {
                        return vec![];
                    }
                    let x_center =
                        bi as f64 - group_width / 2.0 + si as f64 * bar_width + bar_width * 0.45;
                    let cap_half = bar_width * 0.2;
                    let upper = val + sd;
                    let lower = (val - sd).max($y_floor);

                    vec![
                        PathElement::new(vec![(x_center, lower), (x_center, upper)], BLACK),
                        PathElement::new(
                            vec![(x_center - cap_half, upper), (x_center + cap_half, upper)],
                            BLACK,
                        ),
                        PathElement::new(
                            vec![(x_center - cap_half, lower), (x_center + cap_half, lower)],
                            BLACK,
                        ),
                    ]
                })
                .collect();

            $ctx.draw_series(whiskers)?;
        }

        $ctx.configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
    }};
}

fn render_grouped_bar_chart(
    chart: &GroupedBarChart,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let n_buckets = chart.bucket_labels.len();
    let n_series = chart.series.len();

    if n_series == 0 || n_buckets == 0 {
        return Ok(());
    }

    let all_upper: Vec<f64> = chart
        .series
        .iter()
        .flat_map(|s| s.values.iter().zip(s.std_devs.iter()))
        .filter(|(v, _)| **v > 0.0)
        .map(|(v, s)| v + s)
        .collect();

    if all_upper.is_empty() {
        return Ok(());
    }

    let root = SVGBackend::new(path, (900, 500)).into_drawing_area();
    root.fill(&WHITE)?;

    let bucket_labels = chart.bucket_labels.clone();
    let x_range = -0.5f64..(n_buckets as f64 - 0.5);

    let x_label_fmt = move |x: &f64| {
        let idx = x.round() as i64;
        if idx >= 0 && (idx as usize) < bucket_labels.len() && (*x - idx as f64).abs() < 0.1 {
            bucket_labels[idx as usize].clone()
        } else {
            String::new()
        }
    };

    if chart.log_y {
        let min_positive = chart
            .series
            .iter()
            .flat_map(|s| &s.values)
            .copied()
            .filter(|v| *v > 0.0)
            .fold(f64::MAX, f64::min);

        let max_with_err = all_upper.iter().copied().fold(0.0f64, f64::max);

        let y_min = 10f64.powi(min_positive.log10().floor() as i32 - 1);
        let y_max = 10f64.powi(max_with_err.log10().ceil() as i32 + 1);

        let mut ctx = ChartBuilder::on(&root)
            .caption(&chart.title, ("sans-serif", 22))
            .margin(15)
            .margin_right(30)
            .x_label_area_size(40)
            .y_label_area_size(80)
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

        draw_chart_content!(ctx, chart, y_min);
    } else {
        let y_max = all_upper.iter().copied().fold(0.0f64, f64::max) * 1.1;
        let y_max = if y_max > 0.0 { y_max } else { 1.0 };

        let mut ctx = ChartBuilder::on(&root)
            .caption(&chart.title, ("sans-serif", 22))
            .margin(15)
            .margin_right(30)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(x_range, 0f64..y_max)?;

        ctx.configure_mesh()
            .disable_x_mesh()
            .x_labels(n_buckets)
            .x_label_formatter(&x_label_fmt)
            .x_desc("Number of peaks")
            .y_desc(&chart.y_label)
            .y_label_formatter(&|v| format!("{:.1e}", v))
            .draw()?;

        draw_chart_content!(ctx, chart, 0.0f64);
    }

    root.present()?;
    round_bars(path);

    Ok(())
}

/// Extract a numeric attribute value from an SVG element string.
fn svg_attr(line: &str, name: &str) -> Option<f64> {
    let pattern = format!(" {name}=\"");
    let start = line.find(&pattern)? + pattern.len();
    let end = start + line[start..].find('"')?;
    line[start..end].parse().ok()
}

/// Extract a string attribute value from an SVG element string.
fn svg_str_attr<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let pattern = format!(" {name}=\"");
    let start = line.find(&pattern)? + pattern.len();
    let end = start + line[start..].find('"')?;
    Some(&line[start..end])
}

/// Post-process SVG: round top corners of data bars (via `<path>`),
/// round all corners of legend swatches and legend box (via `rx`/`ry`).
fn round_bars(path: &str) {
    let content = fs::read_to_string(path).expect("failed to read SVG for post-processing");

    let bar_colors: Vec<String> = COLORS
        .iter()
        .map(|c| format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2))
        .collect();

    let result: String = content
        .lines()
        .map(|line| {
            if !line.contains("<rect ") {
                return line.to_string();
            }

            // Legend background (white fill, opacity < 1) and border (no fill, black stroke)
            let is_legend = (line.contains("fill=\"#FFFFFF\"") && line.contains("opacity=\"0.8\""))
                || (line.contains("fill=\"none\"") && line.contains("stroke=\"#000000\""));
            if is_legend {
                return line.replacen("<rect ", &format!("<rect rx=\"{BAR_CORNER_RADIUS}\" ry=\"{BAR_CORNER_RADIUS}\" "), 1);
            }

            let is_bar = bar_colors
                .iter()
                .any(|c| line.contains(&format!("fill=\"{c}\"")));

            if !is_bar {
                return line.to_string();
            }

            let (Some(x), Some(y), Some(w), Some(h)) =
                (svg_attr(line, "x"), svg_attr(line, "y"), svg_attr(line, "width"), svg_attr(line, "height"))
            else {
                return line.to_string();
            };

            // Legend swatches (small rects, width <= 15) — round all corners
            if h <= 15.0 && w <= 15.0 {
                let r = 2.0_f64.min(h / 2.0).min(w / 2.0);
                if r > 0.0 {
                    return line.replacen("<rect ", &format!("<rect rx=\"{r}\" ry=\"{r}\" "), 1);
                }
                return line.to_string();
            }

            // Data bars — round only the top two corners using a <path>
            let r = BAR_CORNER_RADIUS.min(h / 2.0).min(w / 2.0);
            if r <= 0.0 {
                return line.to_string();
            }

            let fill = svg_str_attr(line, "fill").unwrap_or("#000000");
            let opacity = svg_str_attr(line, "opacity").unwrap_or("1");

            format!(
                "<path d=\"M {bx},{by} L {bx},{ty} A {r},{r} 0 0 1 {tlx},{top} L {trx},{top} A {r},{r} 0 0 1 {rx},{ty} L {rx},{by} Z\" opacity=\"{opacity}\" fill=\"{fill}\" stroke=\"none\"/>",
                bx = x,
                by = y + h,
                ty = y + r,
                top = y,
                tlx = x + r,
                trx = x + w - r,
                rx = x + w,
                r = r,
                opacity = opacity,
                fill = fill,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(path, result).expect("failed to write post-processed SVG");
}

#[derive(QueryableByName)]
#[allow(dead_code)]
struct ComparisonRow {
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
    matchms_score: f32,
    #[diesel(sql_type = Integer)]
    matchms_matches: i32,
}

fn compare_results(conn: &mut SqliteConnection) {
    let rows: Vec<ComparisonRow> = sql_query(
        "SELECT r.left_id, r.right_id, r.experiment_id,
                r.score AS rust_score, r.matches AS rust_matches,
                m.score AS matchms_score, m.matches AS matchms_matches
         FROM results r
         JOIN results m ON r.left_id = m.left_id
                       AND r.right_id = m.right_id
                       AND r.experiment_id = m.experiment_id
         JOIN implementations ri ON r.implementation_id = ri.id
         JOIN implementations mi ON m.implementation_id = mi.id
         JOIN libraries rl ON ri.library_id = rl.id
         JOIN libraries ml ON mi.library_id = ml.id
         WHERE ri.algorithm_id = mi.algorithm_id
           AND rl.name = 'mass-spectrometry-traits'
           AND ml.name = 'matchms'",
    )
    .load(conn)
    .expect("failed to compare results");

    if rows.is_empty() {
        eprintln!("[report] No cross-implementation pairs to compare yet.");
        return;
    }

    let mut max_score_diff: f32 = 0.0;
    let mut max_match_diff: i32 = 0;
    let mut sum_sq_score: f64 = 0.0;
    let mut mismatch_count = 0usize;

    for r in &rows {
        let sd = (r.rust_score - r.matchms_score).abs();
        let md = (r.rust_matches - r.matchms_matches).abs();
        if sd > max_score_diff {
            max_score_diff = sd;
        }
        if md > max_match_diff {
            max_match_diff = md;
        }
        sum_sq_score += (sd as f64) * (sd as f64);
        if sd > 1e-6 || md > 0 {
            mismatch_count += 1;
        }
    }

    let rmse = (sum_sq_score / rows.len() as f64).sqrt();

    eprintln!("[report] Cross-implementation comparison (Rust vs matchms):");
    eprintln!("[report]   Pairs compared: {}", rows.len());
    eprintln!("[report]   Mismatches (score>1e-6 or matches differ): {mismatch_count}");
    eprintln!("[report]   Max score diff: {max_score_diff:.6e}");
    eprintln!("[report]   Max match diff: {max_match_diff}");
    eprintln!("[report]   RMSE (score): {rmse:.6e}");
}

pub fn run(conn: &mut SqliteConnection) {
    run_to_dir(conn, Path::new("output"));
}

pub fn run_to_dir(conn: &mut SqliteConnection, output_dir: &Path) {
    eprintln!("[report] Generating charts...");
    fs::create_dir_all(output_dir).expect("failed to create output directory");

    let spectra_peaks = load_spectra_peaks(conn);
    let data = load_result_data(conn);

    let color_map = build_color_map(&data);

    let timing_chart = build_timing_chart(&data, &spectra_peaks, &color_map);
    if !timing_chart.series.is_empty() {
        let timing_path = output_dir.join("timing_by_peaks.svg");
        render_grouped_bar_chart(&timing_chart, timing_path.to_string_lossy().as_ref())
            .expect("failed to render timing chart");
        eprintln!("[report] Written {}", timing_path.display());
    }

    let mse_chart = build_mse_chart(&data, &spectra_peaks, &color_map);
    if !mse_chart.series.is_empty() {
        let mse_path = output_dir.join("mse_score_by_peaks.svg");
        render_grouped_bar_chart(&mse_chart, mse_path.to_string_lossy().as_ref())
            .expect("failed to render MSE chart");
        eprintln!("[report] Written {}", mse_path.display());
    }

    compare_results(conn);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_row(algo: &str, lib: &str, score: f32, median_time_us: f32) -> ResultRow {
        ResultRow {
            score,
            median_time_us,
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
            sample_row("CosineHungarian", "matchms", 0.1, 1.0),
            sample_row("CosineGreedy", "matchms", 0.2, 2.0),
            sample_row("CosineHungarian", "mass-spectrometry-traits", 0.3, 3.0),
        ];
        let data_b = vec![
            sample_row("CosineHungarian", "mass-spectrometry-traits", 0.3, 3.0),
            sample_row("CosineHungarian", "matchms", 0.1, 1.0),
            sample_row("CosineGreedy", "matchms", 0.2, 2.0),
        ];

        assert_eq!(build_color_map(&data_a), build_color_map(&data_b));
    }

    #[test]
    fn svg_attribute_extractors_parse_expected_values() {
        let line = r##"<rect x="1.5" y="2.5" width="10" fill="#FFFFFF" opacity="0.8"/>"##;
        assert_eq!(svg_attr(line, "x"), Some(1.5));
        assert_eq!(svg_attr(line, "width"), Some(10.0));
        assert_eq!(svg_attr(line, "missing"), None);
        assert_eq!(svg_str_attr(line, "fill"), Some("#FFFFFF"));
        assert_eq!(svg_str_attr(line, "stroke"), None);
    }

    #[test]
    fn round_bars_rewrites_data_bars_and_rounds_legend_rectangles() {
        let mut file = NamedTempFile::new().expect("failed to create temporary svg file");
        writeln!(
            file,
            "\
<svg>
<rect x=\"1\" y=\"2\" width=\"10\" height=\"20\" fill=\"#529ADC\" opacity=\"1\"/>
<rect x=\"2\" y=\"3\" width=\"14\" height=\"14\" fill=\"#529ADC\" opacity=\"1\"/>
<rect x=\"0\" y=\"0\" width=\"100\" height=\"50\" fill=\"#FFFFFF\" opacity=\"0.8\"/>
<rect x=\"0\" y=\"0\" width=\"100\" height=\"50\" fill=\"none\" stroke=\"#000000\"/>
<rect x=\"5\" y=\"6\" width=\"7\" height=\"8\" fill=\"#000000\" opacity=\"1\"/>
</svg>"
        )
        .expect("failed to write temporary svg");

        let path = file.path().to_string_lossy().to_string();
        round_bars(&path);

        let content = fs::read_to_string(file.path()).expect("failed to read processed svg");
        assert!(content.contains("<path d=\"M 1,22"));
        assert!(content.contains("<rect rx=\"2\" ry=\"2\" x=\"2\" y=\"3\""));
        assert!(content.contains("<rect rx=\"3\" ry=\"3\" x=\"0\" y=\"0\" width=\"100\" height=\"50\" fill=\"#FFFFFF\" opacity=\"0.8\"/>"));
        assert!(content.contains("<rect rx=\"3\" ry=\"3\" x=\"0\" y=\"0\" width=\"100\" height=\"50\" fill=\"none\" stroke=\"#000000\"/>"));
        assert!(content.contains(
            "<rect x=\"5\" y=\"6\" width=\"7\" height=\"8\" fill=\"#000000\" opacity=\"1\"/>"
        ));
    }
}
