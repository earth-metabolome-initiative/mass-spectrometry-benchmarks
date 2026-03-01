use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::peaks::Peaks;
use crate::schema::*;

// --- Algorithms ---

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = algorithms)]
pub struct Algorithm {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub approximates_algorithm_id: Option<i32>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = algorithms)]
pub struct NewAlgorithm<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub approximates_algorithm_id: Option<i32>,
}

// --- Libraries ---

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = libraries)]
pub struct Library {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub git_commit: Option<String>,
    pub git_url: Option<String>,
    pub language: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = libraries)]
pub struct NewLibrary<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub git_commit: Option<&'a str>,
    pub git_url: Option<&'a str>,
    pub language: &'a str,
}

// --- Implementations ---

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = implementations)]
pub struct Implementation {
    pub id: i32,
    pub algorithm_id: i32,
    pub library_id: i32,
    pub is_reference: bool,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = implementations)]
pub struct NewImplementation {
    pub algorithm_id: i32,
    pub library_id: i32,
    pub is_reference: bool,
}

// --- Experiments ---

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = experiments)]
pub struct Experiment {
    pub id: i32,
    pub params: String,
}

impl Experiment {
    pub fn parse_params(&self) -> ExperimentParams {
        serde_json::from_str(&self.params).expect("invalid experiment params JSON")
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = experiments)]
pub struct NewExperiment {
    pub params: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentParams {
    pub tolerance: f32,
    pub mz_power: f32,
    pub intensity_power: f32,
    pub n_warmup: u32,
    pub n_reps: u32,
}

// --- Spectra ---

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = spectra)]
pub struct SpectrumRow {
    pub id: i32,
    pub name: String,
    pub raw_name: String,
    pub source_file: String,
    pub spectrum_hash: String,
    pub precursor_mz: f32,
    pub num_peaks: i32,
    pub peaks: Peaks,
}

impl SpectrumRow {
    pub fn to_generic_spectrum(&self) -> mass_spectrometry::prelude::GenericSpectrum<f32, f32> {
        self.peaks.to_generic_spectrum(self.precursor_mz)
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = spectra)]
pub struct NewSpectrum {
    pub name: String,
    pub raw_name: String,
    pub source_file: String,
    pub spectrum_hash: String,
    pub precursor_mz: f32,
    pub num_peaks: i32,
    pub peaks: Peaks,
}

// --- Results ---

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = results)]
pub struct Result {
    pub id: i32,
    pub left_id: i32,
    pub right_id: i32,
    pub experiment_id: i32,
    pub implementation_id: i32,
    pub score: f32,
    pub matches: i32,
    pub median_time_us: f32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = results)]
pub struct NewResult {
    pub left_id: i32,
    pub right_id: i32,
    pub experiment_id: i32,
    pub implementation_id: i32,
    pub score: f32,
    pub matches: i32,
    pub median_time_us: f32,
}
