use plotters::coord::Shift;
use plotters::coord::ranged1d::Ranged;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;

use super::types::{FacetChart, FacetedLineChart, LineSeriesData, MarkerShape};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum YMode {
    Linear { y_max: f64 },
    Log { y_min: f64, y_max: f64 },
}

pub(crate) fn select_y_mode(chart: &FacetedLineChart, facet: &FacetChart) -> Option<YMode> {
    let max_with_err = facet
        .series
        .iter()
        .flat_map(|s| s.values.iter().zip(s.std_devs.iter()))
        .filter_map(|(v, sd)| {
            let upper = v + sd;
            if upper > 0.0 { Some(upper) } else { None }
        })
        .max_by(|a, b| a.total_cmp(b))?;

    let min_positive = facet
        .series
        .iter()
        .flat_map(|s| s.values.iter().copied())
        .filter(|v| *v > 0.0)
        .min_by(|a, b| a.total_cmp(b));

    if chart.log_y
        && let Some(min_positive) = min_positive
    {
        return Some(YMode::Log {
            y_min: 10f64.powi(min_positive.log10().floor() as i32 - 1),
            y_max: 10f64.powi(max_with_err.log10().ceil() as i32 + 1),
        });
    }

    Some(YMode::Linear {
        y_max: max_with_err.max(1.0) * 1.1,
    })
}

pub(crate) fn build_series_points(values: &[f64], positive_only: bool) -> Vec<(f64, f64)> {
    values
        .iter()
        .enumerate()
        .filter_map(|(idx, &value)| {
            if positive_only && value <= 0.0 {
                None
            } else {
                Some((idx as f64, value))
            }
        })
        .collect()
}

fn draw_series_line_and_markers<'a, DB, Y>(
    ctx: &mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, Y>>,
    series: &LineSeriesData,
    points: &[(f64, f64)],
) -> Result<(), Box<dyn std::error::Error>>
where
    DB: DrawingBackend + 'a,
    DB::ErrorType: 'static,
    Y: Ranged<ValueType = f64>,
{
    let color = series.color;
    match series.marker {
        MarkerShape::Circle => {
            ctx.draw_series(LineSeries::new(
                points.iter().copied(),
                color.stroke_width(2),
            ))?
            .label(series.label.clone())
            .legend(move |(x, y)| {
                EmptyElement::at((x, y))
                    + PathElement::new(vec![(0, 0), (16, 0)], color.stroke_width(2))
                    + Circle::new((8, 0), 3, color.filled())
            });
            ctx.draw_series(points.iter().map(|(x, y)| {
                EmptyElement::at((*x, *y)) + Circle::new((0, 0), 3, color.filled())
            }))?;
        }
        MarkerShape::Square => {
            ctx.draw_series(LineSeries::new(
                points.iter().copied(),
                color.stroke_width(2),
            ))?
            .label(series.label.clone())
            .legend(move |(x, y)| {
                EmptyElement::at((x, y))
                    + PathElement::new(vec![(0, 0), (16, 0)], color.stroke_width(2))
                    + Rectangle::new([(5, -3), (11, 3)], color.filled())
            });
            ctx.draw_series(points.iter().map(|(x, y)| {
                EmptyElement::at((*x, *y)) + Rectangle::new([(-3, -3), (3, 3)], color.filled())
            }))?;
        }
        MarkerShape::Triangle => {
            ctx.draw_series(LineSeries::new(
                points.iter().copied(),
                color.stroke_width(2),
            ))?
            .label(series.label.clone())
            .legend(move |(x, y)| {
                EmptyElement::at((x, y))
                    + PathElement::new(vec![(0, 0), (16, 0)], color.stroke_width(2))
                    + Polygon::new(vec![(8, -4), (4, 3), (12, 3)], color.filled())
            });
            ctx.draw_series(points.iter().map(|(x, y)| {
                EmptyElement::at((*x, *y))
                    + Polygon::new(vec![(0, -4), (-4, 3), (4, 3)], color.filled())
            }))?;
        }
    }

    Ok(())
}

pub(crate) fn build_series_whiskers(
    series: &LineSeriesData,
    positive_only: bool,
    lower_bound: f64,
) -> Vec<PathElement<(f64, f64)>> {
    let color = series.color;
    series
        .values
        .iter()
        .zip(series.std_devs.iter())
        .enumerate()
        .filter_map(|(idx, (&value, &std_dev))| {
            if std_dev <= 0.0 || (positive_only && value <= 0.0) {
                return None;
            }

            let lower = (value - std_dev).max(lower_bound);
            let upper = value + std_dev;
            let x = idx as f64;
            let cap_half = 0.15;
            let whisker_style = color.stroke_width(1);
            Some(vec![
                PathElement::new(vec![(x, lower), (x, upper)], whisker_style),
                PathElement::new(
                    vec![(x - cap_half, upper), (x + cap_half, upper)],
                    whisker_style,
                ),
                PathElement::new(
                    vec![(x - cap_half, lower), (x + cap_half, lower)],
                    whisker_style,
                ),
            ])
        })
        .flatten()
        .collect()
}

fn draw_series_whiskers<'a, DB, Y>(
    ctx: &mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, Y>>,
    series: &LineSeriesData,
    positive_only: bool,
    lower_bound: f64,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB: DrawingBackend + 'a,
    DB::ErrorType: 'static,
    Y: Ranged<ValueType = f64>,
{
    let whiskers = build_series_whiskers(series, positive_only, lower_bound);
    ctx.draw_series(whiskers)?;
    Ok(())
}

fn configure_series_legend<'a, DB, Y>(
    ctx: &mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, Y>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB: DrawingBackend + 'a,
    DB::ErrorType: 'static,
    Y: Ranged<ValueType = f64>,
{
    ctx.configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .legend_area_size(30)
        .margin(10)
        .background_style(WHITE.mix(0.85))
        .border_style(BLACK.mix(0.5))
        .draw()?;
    Ok(())
}

fn draw_facet_series<'a, DB, Y>(
    ctx: &mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, Y>>,
    series_list: &[LineSeriesData],
    positive_only: bool,
    lower_bound: f64,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB: DrawingBackend + 'a,
    DB::ErrorType: 'static,
    Y: Ranged<ValueType = f64>,
{
    for series in series_list {
        let points = build_series_points(&series.values, positive_only);
        if points.is_empty() {
            continue;
        }

        draw_series_line_and_markers(ctx, series, &points)?;
        draw_series_whiskers(ctx, series, positive_only, lower_bound)?;
    }

    configure_series_legend(ctx)?;
    Ok(())
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

    let Some(y_mode) = select_y_mode(chart, facet) else {
        return Ok(());
    };

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

    match y_mode {
        YMode::Log { y_min, y_max } => {
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
                .y_label_formatter(&|v| format!("{v:.0e}"))
                .draw()?;
            draw_facet_series(&mut ctx, &facet.series, true, y_min)?;
        }
        YMode::Linear { y_max } => {
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
                .y_label_formatter(&|v| format!("{v:.1e}"))
                .draw()?;
            draw_facet_series(&mut ctx, &facet.series, false, 0.0)?;
        }
    }

    Ok(())
}

pub(crate) fn render_faceted_line_chart(
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
