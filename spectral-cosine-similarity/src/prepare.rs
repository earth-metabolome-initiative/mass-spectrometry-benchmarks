use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
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
    let mut remaining_budget = max_spectra.map(|max| max.saturating_sub(total_existing));

    if let Some(0) = remaining_budget {
        let max = max_spectra.expect("remaining_budget is Some only when max_spectra is set");
        eprintln!("[prepare] {total_existing} spectra already in DB (max {max}), nothing to load");
        return;
    }

    let sources = mgf_sources();
    eprintln!(
        "[prepare] {} spectra already in DB, scanning {} source(s) for missing spectra...",
        total_existing,
        sources.len()
    );

    let mut seen_names = existing_names;
    let mut inserted_total = 0usize;
    let mut processed_sources = 0usize;

    for (path, source_label) in sources {
        let mgf_path = Path::new(path);
        if !mgf_path.exists() {
            eprintln!("[prepare] Skipping {path} (not found)");
            continue;
        }
        processed_sources += 1;

        let parsed = parse_mgf(mgf_path, MIN_PEAKS);
        let parsed_count = parsed.len();
        let mut skipped_missing_or_duplicate = 0usize;
        let mut inserted_source = 0usize;
        let mut stopped_due_to_max = false;
        let mut batch: Vec<NewSpectrum> = Vec::new();

        for spec in parsed {
            if let Some(remaining) = remaining_budget
                && remaining == 0
            {
                stopped_due_to_max = true;
                break;
            }

            let name = sanitize_name(&spec.raw_name);
            if name.is_empty() || seen_names.contains(&name) {
                skipped_missing_or_duplicate += 1;
                continue;
            }
            seen_names.insert(name.clone());

            let peaks = Peaks(spec.peaks);
            let num_peaks = peaks.0.len() as i32;

            batch.push(NewSpectrum {
                name,
                raw_name: spec.raw_name,
                source_file: source_label.to_string(),
                precursor_mz: spec.precursor_mz,
                num_peaks,
                peaks,
            });

            inserted_source += 1;
            if let Some(ref mut remaining) = remaining_budget {
                *remaining = remaining.saturating_sub(1);
            }
        }

        for chunk in batch.chunks(500) {
            diesel::insert_into(spectra::table)
                .values(chunk)
                .execute(conn)
                .expect("failed to insert spectra batch");
        }

        inserted_total += inserted_source;
        eprintln!(
            "[prepare] {source_label}: parsed={parsed_count}, skipped={skipped_missing_or_duplicate}, inserted={inserted_source}, stopped_due_to_max={stopped_due_to_max}"
        );

        if stopped_due_to_max {
            break;
        }
    }

    if processed_sources == 0 {
        eprintln!("[prepare] No source files were available");
    }

    eprintln!("[prepare] Inserted {inserted_total} new spectra");
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_query;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn setup_in_memory_connection() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:")
            .expect("failed to open in-memory sqlite connection");

        sql_query(
            "CREATE TABLE spectra (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                raw_name TEXT NOT NULL,
                source_file TEXT NOT NULL,
                precursor_mz REAL NOT NULL,
                num_peaks INTEGER NOT NULL,
                peaks TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .expect("failed to create test schema");

        conn
    }

    fn unique_pesticide_spectra() -> Vec<(String, crate::mgf_parser::ParsedSpectrum)> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixture = manifest_dir.join("fixtures").join("pesticides.mgf");
        let mut unique = Vec::new();
        let mut seen = HashSet::new();

        for spec in parse_mgf(&fixture, MIN_PEAKS) {
            let name = sanitize_name(&spec.raw_name);
            if name.is_empty() || !seen.insert(name.clone()) {
                continue;
            }
            unique.push((name, spec));
        }

        unique
    }

    #[test]
    fn backfills_missing_rows_and_respects_hard_cap() {
        let mut conn = setup_in_memory_connection();
        let mut unique_specs = unique_pesticide_spectra();
        assert!(
            unique_specs.len() > 1,
            "fixture should contain at least two unique spectra"
        );

        let total_unique = unique_specs.len();
        let (seed_name, seed_spec) = unique_specs.remove(0);

        diesel::insert_into(spectra::table)
            .values(NewSpectrum {
                name: seed_name,
                raw_name: seed_spec.raw_name,
                source_file: "pesticides.mgf".to_string(),
                precursor_mz: seed_spec.precursor_mz,
                num_peaks: seed_spec.peaks.len() as i32,
                peaks: Peaks(seed_spec.peaks),
            })
            .execute(&mut conn)
            .expect("failed to seed first spectrum");

        run(&mut conn, Some(total_unique));

        let count_after_backfill = spectra::table
            .select(diesel::dsl::count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count rows after backfill");
        assert_eq!(count_after_backfill, total_unique as i64);

        run(&mut conn, Some(total_unique));

        let count_after_second_run = spectra::table
            .select(diesel::dsl::count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count rows after second run");
        assert_eq!(count_after_second_run, total_unique as i64);
    }
}
