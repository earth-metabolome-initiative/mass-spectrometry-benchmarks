use std::collections::HashMap;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use indicatif::{ProgressBar, ProgressStyle};

use crate::models::{FingerprintAlgorithm, TanimotoResult};
use crate::schema::*;

const FLUSH_BATCH: usize = 10_000;

fn tanimoto_binary(a: &[u8], b: &[u8]) -> f64 {
    let mut and_bits: u32 = 0;
    let mut or_bits: u32 = 0;
    for (&x, &y) in a.iter().zip(b.iter()) {
        and_bits += (x & y).count_ones();
        or_bits += (x | y).count_ones();
    }
    if or_bits == 0 {
        return 0.0;
    }
    f64::from(and_bits) / f64::from(or_bits)
}

/// Compute Tanimoto fingerprint similarity for all selected pairs in the DB,
/// once per fingerprint algorithm.
pub fn run(conn: &mut SqliteConnection) {
    let fp_algorithms: Vec<FingerprintAlgorithm> = fingerprint_algorithms::table
        .order(fingerprint_algorithms::id)
        .load(conn)
        .expect("failed to load fingerprint_algorithms");

    if fp_algorithms.is_empty() {
        eprintln!("[fingerprint] No fingerprint algorithms found, skipping");
        return;
    }

    let pairs: Vec<(i32, i32)> = selected_pairs::table
        .select((selected_pairs::left_id, selected_pairs::right_id))
        .order((selected_pairs::left_id, selected_pairs::right_id))
        .load(conn)
        .expect("failed to load selected_pairs");

    if pairs.is_empty() {
        eprintln!("[fingerprint] No selected pairs found");
        return;
    }

    // Collect all spectrum IDs involved in pairs.
    let mut spec_ids: Vec<i32> = pairs.iter().flat_map(|&(l, r)| [l, r]).collect();
    spec_ids.sort_unstable();
    spec_ids.dedup();

    // Map spectrum_id -> molecule_id (shared across all algorithms).
    let spec_to_mol: HashMap<i32, i32> = spectra::table
        .filter(spectra::id.eq_any(&spec_ids))
        .select((spectra::id, spectra::molecule_id))
        .load::<(i32, i32)>(conn)
        .expect("failed to load spectrum molecule_ids")
        .into_iter()
        .collect();

    // Collect all molecule IDs we need.
    let mut mol_ids: Vec<i32> = spec_to_mol.values().copied().collect();
    mol_ids.sort_unstable();
    mol_ids.dedup();

    for algo in &fp_algorithms {
        // Check if tanimoto rows already exist for this algorithm.
        let existing: i64 = tanimoto_results::table
            .filter(tanimoto_results::fingerprint_algorithm_id.eq(algo.id))
            .select(diesel::dsl::count_star())
            .first(conn)
            .expect("failed to count tanimoto_results");
        if existing > 0 {
            eprintln!(
                "[fingerprint] {} tanimoto results already exist for '{}', skipping",
                existing, algo.name
            );
            continue;
        }

        // Load fingerprints for this algorithm: molecule_id -> fingerprint bytes.
        let mol_fps: HashMap<i32, Vec<u8>> = fingerprints::table
            .filter(fingerprints::fingerprint_algorithm_id.eq(algo.id))
            .filter(fingerprints::molecule_id.eq_any(&mol_ids))
            .select((fingerprints::molecule_id, fingerprints::fingerprint))
            .load::<(i32, Vec<u8>)>(conn)
            .expect("failed to load fingerprints")
            .into_iter()
            .collect();

        eprintln!(
            "[fingerprint] Computing Tanimoto ({}) for {} pairs...",
            algo.name,
            pairs.len()
        );

        let mut batch: Vec<TanimotoResult> = Vec::with_capacity(pairs.len());
        let mut skipped = 0usize;

        for (left_id, right_id) in &pairs {
            let score = (|| {
                let left_mol = spec_to_mol.get(left_id)?;
                let right_mol = spec_to_mol.get(right_id)?;
                let left_fp = mol_fps.get(left_mol)?;
                let right_fp = mol_fps.get(right_mol)?;
                Some(tanimoto_binary(left_fp, right_fp))
            })();

            let Some(score) = score else {
                skipped += 1;
                continue;
            };

            batch.push(TanimotoResult {
                left_id: *left_id,
                right_id: *right_id,
                fingerprint_algorithm_id: algo.id,
                tanimoto_score: score,
            });
        }

        let inserted = batch.len();
        let pb = ProgressBar::new(inserted as u64);
        pb.set_style(
            ProgressStyle::with_template(&format!(
                "[fingerprint] DB flush ({}): {{bar:40}} {{pos}}/{{len}} ({{eta}})",
                algo.name
            ))
            .expect("invalid progress bar template"),
        );
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            for chunk in batch.chunks(FLUSH_BATCH) {
                diesel::insert_into(tanimoto_results::table)
                    .values(chunk)
                    .execute(conn)
                    .expect("failed to insert tanimoto_results");
                pb.inc(chunk.len() as u64);
            }
            Ok(())
        })
        .expect("failed to flush tanimoto_results transaction");
        pb.finish_and_clear();

        eprintln!(
            "[fingerprint] Done ({}): inserted={inserted}, skipped={skipped}",
            algo.name
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanimoto_identical_fingerprints() {
        let fp = vec![0b1010_1010u8; 32];
        assert!((tanimoto_binary(&fp, &fp) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn tanimoto_disjoint_fingerprints() {
        let a = vec![0b1010_1010u8; 32];
        let b = vec![0b0101_0101u8; 32];
        assert!((tanimoto_binary(&a, &b)).abs() < 1e-15);
    }

    #[test]
    fn tanimoto_empty_fingerprints() {
        let a = vec![0u8; 32];
        let b = vec![0u8; 32];
        assert!((tanimoto_binary(&a, &b)).abs() < 1e-15);
    }

    #[test]
    fn tanimoto_half_overlap() {
        // a = 1111_0000, b = 1100_1100 → AND=1100_0000(2 bits), OR=1111_1100(6 bits)
        let a = vec![0b1111_0000];
        let b = vec![0b1100_1100];
        let expected = 2.0 / 6.0;
        assert!((tanimoto_binary(&a, &b) - expected).abs() < 1e-15);
    }
}
