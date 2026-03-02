use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A parsed spectrum from an MGF file.
#[derive(Debug)]
pub struct ParsedSpectrum {
    pub raw_name: String,
    pub precursor_mz: f64,
    pub peaks: Vec<(f64, f64)>, // (mz, intensity), sorted by mz
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParseStats {
    pub ions_blocks: usize,
    pub accepted: usize,
    pub dropped_missing_name: usize,
    pub dropped_missing_precursor_mz: usize,
    pub dropped_too_few_peaks: usize,
}

#[derive(Debug, Default)]
pub struct ParseMgfResult {
    pub spectra: Vec<ParsedSpectrum>,
    pub stats: ParseStats,
}

fn preferred_name(
    name: &Option<String>,
    title: &Option<String>,
    compound_name: &Option<String>,
) -> Option<String> {
    name.clone()
        .or_else(|| title.clone())
        .or_else(|| compound_name.clone())
}

/// Parse an MGF file and return all spectra with a preferred name, precursor_mz, and at least `min_peaks` peaks.
pub fn parse_mgf(path: &Path, min_peaks: usize) -> ParseMgfResult {
    let file = File::open(path).unwrap_or_else(|e| panic!("cannot open {}: {e}", path.display()));
    let reader = BufReader::new(file);

    let mut spectra = Vec::new();
    let mut stats = ParseStats::default();
    let mut name: Option<String> = None;
    let mut title: Option<String> = None;
    let mut compound_name: Option<String> = None;
    let mut precursor_mz: Option<f64> = None;
    let mut peaks: Vec<(f64, f64)> = Vec::new();
    let mut in_ions = false;

    for line in reader.lines() {
        let line = line.expect("failed to read line");
        let line = line.trim();

        if line == "BEGIN IONS" {
            stats.ions_blocks += 1;
            name = None;
            title = None;
            compound_name = None;
            precursor_mz = None;
            peaks.clear();
            in_ions = true;
        } else if line == "END IONS" {
            let raw_name = preferred_name(&name, &title, &compound_name);
            match (raw_name, precursor_mz.take()) {
                (None, _) => {
                    stats.dropped_missing_name += 1;
                }
                (Some(_), None) => {
                    stats.dropped_missing_precursor_mz += 1;
                }
                (Some(_), Some(_)) if peaks.len() < min_peaks => {
                    stats.dropped_too_few_peaks += 1;
                }
                (Some(raw_name), Some(pmz)) => {
                    peaks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                    spectra.push(ParsedSpectrum {
                        raw_name,
                        precursor_mz: pmz,
                        peaks: std::mem::take(&mut peaks),
                    });
                    stats.accepted += 1;
                }
            }
            in_ions = false;
        } else if in_ions {
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_uppercase();
                let val = val.trim();
                match key.as_str() {
                    "NAME" => name = Some(val.to_string()),
                    "TITLE" => title = Some(val.to_string()),
                    "COMPOUND_NAME" => compound_name = Some(val.to_string()),
                    "PEPMASS" | "PRECURSOR_MZ" => {
                        if let Some(first) = val.split_whitespace().next()
                            && let Ok(mz) = first.parse::<f64>()
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
                        (parts[0].parse::<f64>(), parts[1].parse::<f64>())
                {
                    peaks.push((mz, intensity));
                }
            }
        }
    }

    ParseMgfResult { spectra, stats }
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

#[cfg(test)]
mod tests {
    use super::{ParseStats, parse_mgf, sanitize_name};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn sanitize_name_normalizes_and_prefixes_numeric_names() {
        assert_eq!(sanitize_name("\"Foo Bar [M+H]+\""), "foo_bar_m_h");
        assert_eq!(sanitize_name("__Weird__Name__"), "weird_name");
        assert_eq!(sanitize_name("123abc"), "n123abc");
        assert_eq!(sanitize_name(""), "");
    }

    #[test]
    fn parse_mgf_filters_and_sorts_peaks() {
        let mut file = NamedTempFile::new().expect("failed to create temporary mgf file");
        writeln!(
            file,
            "\
BEGIN IONS
NAME=First
PEPMASS=123.4 99
200 5
150 10
170 7
180 8
190 9
END IONS
BEGIN IONS
NAME=MissingPrecursor
100 1
101 2
102 3
103 4
104 5
END IONS
BEGIN IONS
NAME=FewPeaks
PEPMASS=321.0
100 1
101 2
END IONS
BEGIN IONS
NAME=Second
PRECURSOR_MZ=456.7
130 4
110 2
120 3
140 5
150 6
END IONS"
        )
        .expect("failed to write temporary mgf fixture");

        let parsed = parse_mgf(file.path(), 5);
        assert_eq!(parsed.spectra.len(), 2);
        assert_eq!(parsed.spectra[0].raw_name, "First");
        assert_eq!(parsed.spectra[0].precursor_mz, 123.4);
        assert_eq!(
            parsed.spectra[0].peaks,
            vec![
                (150.0, 10.0),
                (170.0, 7.0),
                (180.0, 8.0),
                (190.0, 9.0),
                (200.0, 5.0)
            ]
        );

        assert_eq!(parsed.spectra[1].raw_name, "Second");
        assert_eq!(parsed.spectra[1].precursor_mz, 456.7);
        assert_eq!(
            parsed.spectra[1].peaks,
            vec![
                (110.0, 2.0),
                (120.0, 3.0),
                (130.0, 4.0),
                (140.0, 5.0),
                (150.0, 6.0)
            ]
        );
        assert_eq!(
            parsed.stats,
            ParseStats {
                ions_blocks: 4,
                accepted: 2,
                dropped_missing_name: 0,
                dropped_missing_precursor_mz: 1,
                dropped_too_few_peaks: 1,
            }
        );
    }

    #[test]
    fn parse_mgf_uses_name_then_title_then_compound_name() {
        let mut file = NamedTempFile::new().expect("failed to create temporary mgf file");
        writeln!(
            file,
            "\
BEGIN IONS
TITLE=title_only
PEPMASS=100
10 1
20 2
30 3
40 4
50 5
END IONS
BEGIN IONS
COMPOUND_NAME=compound_only
PEPMASS=200
10 1
20 2
30 3
40 4
50 5
END IONS
BEGIN IONS
TITLE=preferred_title
COMPOUND_NAME=ignored_compound
PEPMASS=300
10 1
20 2
30 3
40 4
50 5
END IONS
BEGIN IONS
NAME=preferred_name
TITLE=ignored_title
COMPOUND_NAME=ignored_compound
PEPMASS=400
10 1
20 2
30 3
40 4
50 5
END IONS"
        )
        .expect("failed to write temporary mgf fixture");

        let parsed = parse_mgf(file.path(), 5);
        let names: Vec<&str> = parsed.spectra.iter().map(|s| s.raw_name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "title_only",
                "compound_only",
                "preferred_title",
                "preferred_name"
            ]
        );
        assert_eq!(parsed.stats.accepted, 4);
    }
}
