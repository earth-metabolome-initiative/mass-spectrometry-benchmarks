use plotters::prelude::RGBColor;

pub(crate) const BUCKET_BOUNDARIES: &[i32] = &[5, 9, 17, 33, 65, 129, 257, 513, 1024, 2048];
pub(crate) const RMSE_LOG_FLOOR: f64 = 1e-16;

pub(crate) const LIBRARY_COLORS: [RGBColor; 11] = [
    RGBColor(82, 154, 220),  // pastel blue
    RGBColor(238, 134, 62),  // pastel orange
    RGBColor(82, 182, 96),   // pastel green
    RGBColor(210, 78, 78),   // pastel red
    RGBColor(146, 112, 184), // pastel lavender
    RGBColor(93, 185, 175),  // pastel teal
    RGBColor(227, 116, 133), // pastel rose
    RGBColor(169, 148, 97),  // pastel khaki
    RGBColor(119, 130, 210), // pastel indigo
    RGBColor(145, 185, 102), // pastel lime
    RGBColor(214, 169, 0),   // goldenrod (high contrast fallback replacement)
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MarkerShape {
    Circle,
    Square,
    Triangle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SeriesStyle {
    pub(crate) color: RGBColor,
    pub(crate) marker: MarkerShape,
}

#[derive(Clone, Debug)]
pub(crate) struct LineSeriesData {
    pub(crate) label: String,
    pub(crate) color: RGBColor,
    pub(crate) marker: MarkerShape,
    pub(crate) values: Vec<f64>,
    pub(crate) std_devs: Vec<f64>,
    pub(crate) counts: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct FacetChart {
    pub(crate) title: String,
    pub(crate) series: Vec<LineSeriesData>,
}

#[derive(Clone, Debug)]
pub(crate) struct FacetedLineChart {
    pub(crate) title: String,
    pub(crate) y_label: String,
    pub(crate) bucket_labels: Vec<String>,
    pub(crate) facets: Vec<FacetChart>,
    pub(crate) log_y: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct AggregatedSeriesPoint {
    pub(crate) facet_label: String,
    pub(crate) series_label: String,
    pub(crate) library_name: String,
    pub(crate) bucket_index: usize,
    pub(crate) value: f64,
    pub(crate) std_dev: f64,
    pub(crate) count: usize,
}

pub(crate) fn algorithm_uses_match_count_parity(algorithm: &str) -> bool {
    !algorithm.starts_with("EntropySimilarity")
}
