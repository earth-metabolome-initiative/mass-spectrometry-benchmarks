use plotters::prelude::RGBColor;

pub(crate) const BUCKET_BOUNDARIES: &[i32] = &[5, 9, 17, 33, 65, 129, 257, 513, 1024, 2048];
pub(crate) const MSE_LOG_FLOOR: f64 = 1e-16;

pub(crate) const LIBRARY_COLORS: [RGBColor; 10] = [
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
];

#[derive(Clone, Debug)]
pub(crate) struct AlgorithmReference {
    pub(crate) implementation_id: i32,
    pub(crate) label: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ResultRow {
    pub(crate) implementation_id: i32,
    pub(crate) is_reference: bool,
    pub(crate) score: f32,
    pub(crate) median_time_us: f32,
    pub(crate) algo_name: String,
    pub(crate) lib_name: String,
    pub(crate) left_id: i32,
    pub(crate) right_id: i32,
    pub(crate) experiment_id: i32,
}

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

pub(crate) fn algorithm_uses_match_count_parity(algorithm: &str) -> bool {
    !algorithm.starts_with("EntropySimilarity")
}
