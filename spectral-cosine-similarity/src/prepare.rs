use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::mgf_parser::{parse_mgf, sanitize_name};
use crate::models::*;
use crate::peaks::Peaks;
use crate::progress::StageProgress;
use crate::schema::*;

const MIN_PEAKS: usize = 5;
const HASH_DECIMALS: usize = 6;

fn canonicalize_component(value: f32) -> String {
    if !value.is_finite() {
        return value.to_string().to_ascii_lowercase();
    }

    let scale = 10f64.powi(HASH_DECIMALS as i32);
    let mut quantized = ((value as f64) * scale).round() / scale;
    if quantized == -0.0 {
        quantized = 0.0;
    }
    format!("{quantized:.6}")
}

fn compute_spectrum_hash(precursor_mz: f32, peaks: &[(f32, f32)]) -> String {
    let mut sorted = peaks.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));

    let mut payload = String::new();
    payload.push_str("pmz=");
    payload.push_str(&canonicalize_component(precursor_mz));
    payload.push_str(";n=");
    payload.push_str(&sorted.len().to_string());
    payload.push_str(";peaks=");

    for (idx, (mz, intensity)) in sorted.iter().enumerate() {
        if idx > 0 {
            payload.push('|');
        }
        payload.push_str(&canonicalize_component(*mz));
        payload.push(':');
        payload.push_str(&canonicalize_component(*intensity));
    }

    let digest = Sha256::digest(payload.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// MGF files to process, in order.
fn mgf_sources() -> Vec<(&'static str, &'static str)> {
    let mut sources = vec![("fixtures/pesticides.mgf", "pesticides.mgf")];
    if Path::new("fixtures/GNPS-LIBRARY.mgf").exists() {
        sources.push(("fixtures/GNPS-LIBRARY.mgf", "GNPS-LIBRARY.mgf"));
    }
    sources
}

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(p) = progress.as_deref_mut() {
        p.set_substep(message);
    } else {
        eprintln!("{message}");
    }
}

/// Parse MGF files and insert spectra into the database.
pub fn run(conn: &mut SqliteConnection, max_spectra: Option<usize>) {
    run_with_progress(conn, max_spectra, None);
}

/// Parse MGF files and insert spectra into the database with optional progress updates.
pub fn run_with_progress(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    progress: Option<&mut dyn StageProgress>,
) {
    let raw_sources = mgf_sources();
    let sources: Vec<(&Path, &str)> = raw_sources
        .iter()
        .map(|(path, source_label)| (Path::new(*path), *source_label))
        .collect();
    run_with_sources_with_progress(conn, max_spectra, &sources, progress);
}

/// Parse explicit MGF sources and insert spectra into the database.
pub fn run_with_sources(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    sources: &[(&Path, &str)],
) {
    run_with_sources_with_progress(conn, max_spectra, sources, None);
}

/// Parse explicit MGF sources and insert spectra into the database with optional progress updates.
pub fn run_with_sources_with_progress(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    sources: &[(&Path, &str)],
    mut progress: Option<&mut dyn StageProgress>,
) {
    // Collect hashes already in the DB to avoid duplicates.
    let existing_hashes: std::collections::HashSet<String> = spectra::table
        .order(spectra::id.asc())
        .select(spectra::spectrum_hash)
        .load::<String>(conn)
        .expect("failed to load existing spectrum hashes")
        .into_iter()
        .collect();

    let total_existing = existing_hashes.len();
    let mut remaining_budget = max_spectra.map(|max| max.saturating_sub(total_existing));

    if let Some(0) = remaining_budget {
        let max = max_spectra.expect("remaining_budget is Some only when max_spectra is set");
        emit(
            &mut progress,
            &format!(
                "[prepare] {total_existing} spectra already in DB (max {max}), nothing to load"
            ),
        );
        return;
    }

    emit(
        &mut progress,
        &format!(
            "[prepare] {total_existing} spectra already in DB, scanning {} source(s) for missing spectra...",
            sources.len()
        ),
    );

    let mut seen_hashes = existing_hashes;
    let mut inserted_total = 0usize;
    let mut processed_sources = 0usize;

    for (mgf_path, source_label) in sources {
        if !mgf_path.exists() {
            emit(
                &mut progress,
                &format!("[prepare] Skipping {} (not found)", mgf_path.display()),
            );
            continue;
        }
        processed_sources += 1;
        emit(
            &mut progress,
            &format!("[prepare] Parsing {}", mgf_path.display()),
        );

        let parsed = parse_mgf(mgf_path, MIN_PEAKS);
        let parsed_count = parsed.len();
        let mut skipped_hash_duplicates = 0usize;
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

            let spectrum_hash = compute_spectrum_hash(spec.precursor_mz, &spec.peaks);
            if seen_hashes.contains(&spectrum_hash) {
                skipped_hash_duplicates += 1;
                continue;
            }
            seen_hashes.insert(spectrum_hash.clone());

            let mut name = sanitize_name(&spec.raw_name);
            if name.is_empty() {
                name = format!("spectrum_{}", &spectrum_hash[..12]);
            }

            let peaks = Peaks(spec.peaks);
            let num_peaks = peaks.0.len() as i32;

            batch.push(NewSpectrum {
                name,
                raw_name: spec.raw_name,
                source_file: source_label.to_string(),
                spectrum_hash,
                precursor_mz: spec.precursor_mz,
                num_peaks,
                peaks,
            });

            if let Some(ref mut remaining) = remaining_budget {
                *remaining = remaining.saturating_sub(1);
            }
        }

        for chunk in batch.chunks(500) {
            for row in chunk {
                let inserted = diesel::insert_into(spectra::table)
                    .values(row)
                    .on_conflict(spectra::spectrum_hash)
                    .do_nothing()
                    .execute(conn)
                    .expect("failed to insert spectrum row");
                inserted_source += inserted;
            }
        }

        inserted_total += inserted_source;
        emit(
            &mut progress,
            &format!(
                "[prepare] {source_label}: parsed={parsed_count}, hash_duplicates={skipped_hash_duplicates}, inserted={inserted_source}, stopped_due_to_max={stopped_due_to_max}"
            ),
        );

        if stopped_due_to_max {
            break;
        }
    }

    if processed_sources == 0 {
        emit(&mut progress, "[prepare] No source files were available");
    }

    emit(
        &mut progress,
        &format!("[prepare] Inserted {inserted_total} new spectra"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_query;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    fn setup_in_memory_connection() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:")
            .expect("failed to open in-memory sqlite connection");

        sql_query(
            "CREATE TABLE spectra (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                raw_name TEXT NOT NULL,
                source_file TEXT NOT NULL,
                spectrum_hash TEXT NOT NULL UNIQUE,
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
        let fixture = pesticide_fixture_path();
        let mut unique = Vec::new();
        let mut seen = HashSet::new();

        for spec in parse_mgf(fixture.as_path(), MIN_PEAKS) {
            let hash = compute_spectrum_hash(spec.precursor_mz, &spec.peaks);
            if !seen.insert(hash.clone()) {
                continue;
            }
            unique.push((hash, spec));
        }

        unique
    }

    fn pesticide_fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("pesticides.mgf")
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
        let (seed_hash, seed_spec) = unique_specs.remove(0);
        let seed_name = sanitize_name(&seed_spec.raw_name);
        let seed_name = if seed_name.is_empty() {
            format!("spectrum_{}", &seed_hash[..12])
        } else {
            seed_name
        };

        diesel::insert_into(spectra::table)
            .values(NewSpectrum {
                name: seed_name,
                raw_name: seed_spec.raw_name,
                source_file: "pesticides.mgf".to_string(),
                spectrum_hash: seed_hash,
                precursor_mz: seed_spec.precursor_mz,
                num_peaks: seed_spec.peaks.len() as i32,
                peaks: Peaks(seed_spec.peaks),
            })
            .execute(&mut conn)
            .expect("failed to seed first spectrum");

        let fixture = pesticide_fixture_path();
        let sources: [(&Path, &str); 1] = [(fixture.as_path(), "pesticides.mgf")];

        run_with_sources(&mut conn, Some(total_unique), &sources);

        let count_after_backfill = spectra::table
            .select(diesel::dsl::count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count rows after backfill");
        assert_eq!(count_after_backfill, total_unique as i64);

        run_with_sources(&mut conn, Some(total_unique), &sources);

        let count_after_second_run = spectra::table
            .select(diesel::dsl::count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count rows after second run");
        assert_eq!(count_after_second_run, total_unique as i64);
    }

    #[test]
    fn spectrum_hash_is_order_invariant_and_quantized() {
        let pmz_base = 500.123_44_f32;
        let pmz_nearly_equal = pmz_base + 1e-8_f32;
        let mz_base = 200.123_46_f32;
        let mz_nearly_equal = mz_base + 1e-8_f32;

        assert_eq!(
            canonicalize_component(pmz_base),
            canonicalize_component(pmz_nearly_equal)
        );
        assert_ne!(
            canonicalize_component(500.123_4),
            canonicalize_component(500.123_5)
        );

        let h1 = compute_spectrum_hash(
            pmz_base,
            &[(mz_base, 10.000_000_4), (100.000_000_4, 1.000_000_4)],
        );
        let h2 = compute_spectrum_hash(
            pmz_nearly_equal,
            &[
                (100.000_000_4, 1.000_000_4),
                (mz_nearly_equal, 10.000_000_4),
            ],
        );
        let h3 = compute_spectrum_hash(
            pmz_base,
            &[(mz_base, 10.000_000_4), (100.000_000_4, 1.100_000_4)],
        );

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
