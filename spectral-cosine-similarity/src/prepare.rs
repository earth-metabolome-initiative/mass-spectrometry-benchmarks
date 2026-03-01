use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

use crate::mgf_parser::{parse_mgf, sanitize_name};
use crate::models::*;
use crate::peaks::Peaks;
use crate::schema::*;

const MIN_PEAKS: usize = 5;

/// MGF files to process, in order.
fn mgf_sources() -> Vec<(&'static str, &'static str)> {
    let mut sources = vec![("fixtures/pesticides.mgf", "pesticides.mgf")];
    if Path::new("fixtures/GNPS-LIBRARY.mgf").exists() {
        sources.push(("fixtures/GNPS-LIBRARY.mgf", "GNPS-LIBRARY.mgf"));
    }
    sources
}

/// Parse MGF files and insert spectra into the database.
pub fn run(conn: &mut SqliteConnection, max_spectra: Option<usize>) {
    // Collect names already in the DB to avoid duplicates.
    let existing_names: std::collections::HashSet<String> = spectra::table
        .select(spectra::name)
        .load::<String>(conn)
        .expect("failed to load existing spectrum names")
        .into_iter()
        .collect();

    let total_existing = existing_names.len();

    // If we already have enough spectra, skip loading.
    if let Some(max) = max_spectra
        && total_existing >= max
    {
        eprintln!("[prepare] {total_existing} spectra already in DB (max {max}), nothing to load");
        return;
    }

    // Determine which source files still need loading.
    let loaded_sources: std::collections::HashSet<String> = spectra::table
        .select(spectra::source_file)
        .distinct()
        .load::<String>(conn)
        .expect("failed to load source files")
        .into_iter()
        .collect();

    let sources_to_load: Vec<_> = mgf_sources()
        .into_iter()
        .filter(|(_, label)| !loaded_sources.contains(*label))
        .collect();

    if sources_to_load.is_empty() {
        eprintln!(
            "[prepare] {} spectra already in DB, no new sources to load",
            total_existing
        );
        return;
    }

    eprintln!(
        "[prepare] {} spectra already in DB, loading {} new source(s)...",
        total_existing,
        sources_to_load.len()
    );

    let mut seen_names = existing_names;
    let mut inserted = 0;

    for (path, source_label) in sources_to_load {
        let mgf_path = Path::new(path);
        if !mgf_path.exists() {
            eprintln!("[prepare] Skipping {path} (not found)");
            continue;
        }

        let parsed = parse_mgf(mgf_path, MIN_PEAKS);
        eprintln!(
            "[prepare] Parsed {} spectra from {source_label}",
            parsed.len()
        );

        let mut batch: Vec<NewSpectrum> = Vec::new();

        for spec in &parsed {
            let name = sanitize_name(&spec.raw_name);
            if name.is_empty() || seen_names.contains(&name) {
                continue;
            }
            seen_names.insert(name.clone());

            let peaks = Peaks(spec.peaks.clone());
            let num_peaks = peaks.0.len() as i32;

            batch.push(NewSpectrum {
                name,
                raw_name: spec.raw_name.clone(),
                source_file: source_label.to_string(),
                precursor_mz: spec.precursor_mz,
                num_peaks,
                peaks,
            });

            if let Some(max) = max_spectra
                && total_existing + inserted + batch.len() >= max
            {
                batch.truncate(max - total_existing - inserted);
                break;
            }
        }

        let pb = ProgressBar::new(batch.len() as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "[prepare] Inserting spectra {bar:40} {pos}/{len} [{eta}]",
            )
            .unwrap(),
        );

        for chunk in batch.chunks(500) {
            diesel::insert_into(spectra::table)
                .values(chunk)
                .execute(conn)
                .expect("failed to insert spectra batch");
            pb.inc(chunk.len() as u64);
        }

        inserted += batch.len();
        pb.finish_and_clear();

        if let Some(max) = max_spectra
            && total_existing + inserted >= max
        {
            break;
        }
    }
    eprintln!("[prepare] Inserted {inserted} new spectra");
}
