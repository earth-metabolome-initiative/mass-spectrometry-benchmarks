use std::fs;
use std::path::Path;

use super::types::FacetedLineChart;

fn markdown_table_cell(value: f64, std_dev: f64, count: usize) -> String {
    if count == 0 {
        return "-".to_string();
    }

    if std_dev > 0.0 {
        format!("{value:.3e} ± {std_dev:.2e} (n={count})")
    } else {
        format!("{value:.3e} (n={count})")
    }
}

fn append_chart_markdown(markdown: &mut String, chart: &FacetedLineChart) {
    markdown.push_str(&format!("## {}\n\n", chart.title));
    markdown.push_str(&format!("Y-axis: `{}`\n\n", chart.y_label));

    if chart.facets.is_empty() || chart.bucket_labels.is_empty() {
        markdown.push_str("_No data available._\n\n");
        return;
    }

    for facet in &chart.facets {
        markdown.push_str(&format!("### {}\n\n", facet.title));
        markdown.push_str("| Series |");
        for label in &chart.bucket_labels {
            markdown.push_str(&format!(" {label} |"));
        }
        markdown.push('\n');

        markdown.push_str("| --- |");
        for _ in &chart.bucket_labels {
            markdown.push_str(" --- |");
        }
        markdown.push('\n');

        for series in &facet.series {
            markdown.push_str(&format!("| {} |", series.label.replace('|', "\\|")));
            for idx in 0..chart.bucket_labels.len() {
                let value = series.values.get(idx).copied().unwrap_or(0.0);
                let std_dev = series.std_devs.get(idx).copied().unwrap_or(0.0);
                let count = series.counts.get(idx).copied().unwrap_or(0);
                markdown.push_str(&format!(
                    " {} |",
                    markdown_table_cell(value, std_dev, count)
                ));
            }
            markdown.push('\n');
        }
        markdown.push('\n');
    }
}

pub(crate) fn write_markdown_tables(output_path: &Path, charts: &[&FacetedLineChart]) {
    let mut markdown = String::from("# Benchmark Tables\n\n");
    for chart in charts {
        append_chart_markdown(&mut markdown, chart);
    }

    fs::write(output_path, markdown).unwrap_or_else(|err| {
        panic!(
            "failed to write markdown report {}: {err}",
            output_path.display()
        )
    });
}
