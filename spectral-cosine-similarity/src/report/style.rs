use std::collections::HashMap;

use super::types::{LIBRARY_COLORS, MarkerShape, ResultRow, SeriesStyle};

pub(crate) fn series_label(algo: &str, lib: &str) -> String {
    format!("{algo} ({lib})")
}

fn marker_for_library_index(index: usize) -> MarkerShape {
    match index % 3 {
        0 => MarkerShape::Circle,
        1 => MarkerShape::Square,
        _ => MarkerShape::Triangle,
    }
}

fn color_for_series_index(index: usize) -> plotters::style::RGBColor {
    if let Some(color) = LIBRARY_COLORS.get(index) {
        return *color;
    }

    // Fallback distinct-ish colors for unexpected extra libraries.
    let i = index as u32;
    plotters::style::RGBColor(
        (40 + ((97 * i) % 180)) as u8,
        (40 + ((57 * i) % 180)) as u8,
        (40 + ((137 * i) % 180)) as u8,
    )
}

pub(crate) fn build_series_style_map(data: &[ResultRow]) -> HashMap<String, SeriesStyle> {
    let mut libraries: Vec<String> = data.iter().map(|r| r.lib_name.clone()).collect();
    libraries.sort();
    libraries.dedup();

    let library_markers: HashMap<String, MarkerShape> = libraries
        .into_iter()
        .enumerate()
        .map(|(idx, lib)| (lib, marker_for_library_index(idx)))
        .collect();

    let mut series_pairs: Vec<(String, String)> = data
        .iter()
        .map(|r| (series_label(&r.algo_name, &r.lib_name), r.lib_name.clone()))
        .collect();
    series_pairs.sort();
    series_pairs.dedup();

    let mut styles: HashMap<String, SeriesStyle> = HashMap::new();
    for (index, (label, library_name)) in series_pairs.into_iter().enumerate() {
        let marker = *library_markers
            .get(&library_name)
            .expect("marker must exist for each library");
        styles.insert(
            label,
            SeriesStyle {
                color: color_for_series_index(index),
                marker,
            },
        );
    }

    styles
}
