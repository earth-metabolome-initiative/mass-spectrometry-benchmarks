use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A parsed spectrum from an MGF file.
#[derive(Debug)]
pub struct ParsedSpectrum {
    pub raw_name: String,
    pub precursor_mz: f32,
    pub peaks: Vec<(f32, f32)>, // (mz, intensity), sorted by mz
}

/// Parse an MGF file and return all spectra with a name, precursor_mz, and at least `min_peaks` peaks.
pub fn parse_mgf(path: &Path, min_peaks: usize) -> Vec<ParsedSpectrum> {
    let file = File::open(path).unwrap_or_else(|e| panic!("cannot open {}: {e}", path.display()));
    let reader = BufReader::new(file);

    let mut spectra = Vec::new();
    let mut raw_name: Option<String> = None;
    let mut precursor_mz: Option<f32> = None;
    let mut peaks: Vec<(f32, f32)> = Vec::new();
    let mut in_ions = false;

    for line in reader.lines() {
        let line = line.expect("failed to read line");
        let line = line.trim();

        if line == "BEGIN IONS" {
            raw_name = None;
            precursor_mz = None;
            peaks.clear();
            in_ions = true;
        } else if line == "END IONS" {
            if let (Some(name), Some(pmz)) = (raw_name.take(), precursor_mz.take())
                && peaks.len() >= min_peaks
            {
                peaks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                spectra.push(ParsedSpectrum {
                    raw_name: name,
                    precursor_mz: pmz,
                    peaks: std::mem::take(&mut peaks),
                });
            }
            in_ions = false;
        } else if in_ions {
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_uppercase();
                let val = val.trim();
                match key.as_str() {
                    "NAME" => raw_name = Some(val.to_string()),
                    "PEPMASS" | "PRECURSOR_MZ" => {
                        if let Some(first) = val.split_whitespace().next()
                            && let Ok(mz) = first.parse::<f32>()
                        {
                            precursor_mz = Some(mz);
                        }
                    }
                    _ => {}
                }
            } else if line.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2
                    && let (Ok(mz), Ok(intensity)) =
                        (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                {
                    peaks.push((mz, intensity));
                }
            }
        }
    }

    spectra
}

/// Sanitize a raw spectrum name to a valid snake_case identifier.
/// Preserves charge state / ionization mode since different adducts produce
/// genuinely different spectra.
pub fn sanitize_name(raw_name: &str) -> String {
    let mut name = raw_name.to_string();

    // Strip quotes
    name = name.trim_matches('"').trim_matches('\'').to_string();

    // Convert to snake_case: replace non-alphanumeric with underscore
    let name: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    // Collapse underscores and strip leading/trailing
    let mut result = String::new();
    let mut prev_underscore = true; // start true to strip leading
    for c in name.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    let result = result.trim_end_matches('_').to_string();

    // Rust identifiers cannot start with a digit
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        format!("n{result}")
    } else {
        result
    }
}
