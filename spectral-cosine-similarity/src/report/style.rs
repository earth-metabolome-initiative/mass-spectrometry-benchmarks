use std::collections::HashMap;

use super::types::{LIBRARY_COLORS, MarkerShape, SeriesStyle};

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

pub(crate) fn build_series_style_map_from_pairs(
    series_pairs: &[(String, String)],
) -> HashMap<String, SeriesStyle> {
    let mut libraries: Vec<String> = series_pairs.iter().map(|(_, lib)| lib.clone()).collect();
    libraries.sort();
    libraries.dedup();

    let library_markers: HashMap<String, MarkerShape> = libraries
        .into_iter()
        .enumerate()
        .map(|(idx, lib)| (lib, marker_for_library_index(idx)))
        .collect();

    let mut styles: HashMap<String, SeriesStyle> = HashMap::new();
    for (index, (label, library_name)) in series_pairs.iter().enumerate() {
        let marker = *library_markers
            .get(library_name)
            .expect("marker must exist for each library");
        styles.insert(
            label.clone(),
            SeriesStyle {
                color: color_for_series_index(index),
                marker,
            },
        );
    }

    styles
}
