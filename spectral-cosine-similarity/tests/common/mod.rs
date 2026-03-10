use diesel::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

use spectral_cosine_similarity::db;
use spectral_cosine_similarity::models::{
    Fingerprint, NewFingerprintAlgorithm, NewMolecule, NewSpectrum,
};
use spectral_cosine_similarity::peaks::Peaks;
use spectral_cosine_similarity::schema::{
    fingerprint_algorithms, fingerprints, molecules, spectra,
};

pub struct TestDb {
    _temp_dir: TempDir,
    #[allow(dead_code)]
    pub db_path: PathBuf,
    pub conn: SqliteConnection,
}

impl TestDb {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("failed to create temporary test directory");
        let db_path = temp_dir.path().join("benchmark.db");
        let mut conn = db::establish_connection_at(&db_path);
        db::initialize(&mut conn);

        Self {
            _temp_dir: temp_dir,
            db_path,
            conn,
        }
    }
}

/// Insert hard-coded test molecules and spectra directly into the DB.
/// This avoids needing the Python massspecgym dependency for unit tests.
pub fn prepare_small_dataset(conn: &mut SqliteConnection) {
    let test_molecules = [
        NewMolecule {
            smiles: "CCO".to_string(),
            inchikey: "LFQSCWFLJHTTHZ-UHFFFAOYSA-N".to_string(),
        },
        NewMolecule {
            smiles: "CC(=O)O".to_string(),
            inchikey: "QTBSBXVTEAMEQO-UHFFFAOYSA-N".to_string(),
        },
        NewMolecule {
            smiles: "c1ccccc1".to_string(),
            inchikey: "IMNFDUFMRHMDMM-UHFFFAOYSA-N".to_string(),
        },
        NewMolecule {
            smiles: "CC(C)CC1=CC=C(C=C1)C(C)C(=O)O".to_string(),
            inchikey: "HEFNNWSXXWATRW-UHFFFAOYSA-N".to_string(),
        },
        NewMolecule {
            smiles: "CC12CCC3C(C1CCC2O)CCC4=CC(=O)CCC34C".to_string(),
            inchikey: "VOXZDWNPVJITMN-UHFFFAOYSA-N".to_string(),
        },
    ];

    for mol in &test_molecules {
        diesel::insert_into(molecules::table)
            .values(mol)
            .on_conflict(molecules::inchikey)
            .do_nothing()
            .execute(conn)
            .expect("failed to insert test molecule");
    }

    let mol_ids: Vec<i32> = molecules::table
        .order(molecules::id.asc())
        .select(molecules::id)
        .load(conn)
        .expect("failed to load molecule ids");

    // Insert fingerprint algorithms
    let fp_algo_configs = [
        ("ecfp", r#"{"fp_size":2048,"radius":2}"#),
        (
            "fcfp",
            r#"{"fp_size":2048,"radius":2,"pharmacophoric":true}"#,
        ),
        ("maccs", r#"{"fp_size":166}"#),
        ("rdkit", r#"{"fp_size":2048}"#),
        ("atompair", r#"{"fp_size":2048}"#),
        ("map", r#"{"fp_size":2048}"#),
    ];
    for (name, params) in &fp_algo_configs {
        diesel::insert_into(fingerprint_algorithms::table)
            .values(NewFingerprintAlgorithm { name, params })
            .on_conflict(fingerprint_algorithms::name)
            .do_nothing()
            .execute(conn)
            .expect("failed to insert fingerprint algorithm");
    }
    let fp_algo_ids: Vec<i32> = fingerprint_algorithms::table
        .order(fingerprint_algorithms::id.asc())
        .select(fingerprint_algorithms::id)
        .load(conn)
        .expect("failed to load fingerprint algorithm ids");

    // Insert dummy fingerprints for each molecule × algorithm
    for &mol_id in &mol_ids {
        for (idx, &algo_id) in fp_algo_ids.iter().enumerate() {
            // MACCS = 21 bytes (166 bits), all others = 256 bytes (2048 bits)
            let fp_len = if fp_algo_configs[idx].0 == "maccs" {
                21
            } else {
                256
            };
            diesel::insert_or_ignore_into(fingerprints::table)
                .values(Fingerprint {
                    molecule_id: mol_id,
                    fingerprint_algorithm_id: algo_id,
                    fingerprint: vec![0xAAu8; fp_len],
                })
                .execute(conn)
                .expect("failed to insert fingerprint");
        }
    }

    let test_spectra = [
        NewSpectrum {
            name: "ethanol_spectrum".to_string(),
            raw_name: "ethanol_spectrum".to_string(),
            source_file: "test".to_string(),
            spectrum_hash: "test_hash_0000".to_string(),
            precursor_mz: 47.0,
            num_peaks: 5,
            peaks: Peaks(vec![
                (10.0, 1.0),
                (20.0, 2.0),
                (30.0, 3.0),
                (40.0, 4.0),
                (50.0, 5.0),
            ]),
            molecule_id: mol_ids[0],
        },
        NewSpectrum {
            name: "acetic_acid_spectrum".to_string(),
            raw_name: "acetic_acid_spectrum".to_string(),
            source_file: "test".to_string(),
            spectrum_hash: "test_hash_0001".to_string(),
            precursor_mz: 61.0,
            num_peaks: 5,
            peaks: Peaks(vec![
                (15.0, 1.5),
                (25.0, 2.5),
                (35.0, 3.5),
                (45.0, 4.5),
                (55.0, 5.5),
            ]),
            molecule_id: mol_ids[1],
        },
        NewSpectrum {
            name: "benzene_spectrum".to_string(),
            raw_name: "benzene_spectrum".to_string(),
            source_file: "test".to_string(),
            spectrum_hash: "test_hash_0002".to_string(),
            precursor_mz: 79.0,
            num_peaks: 5,
            peaks: Peaks(vec![
                (18.0, 1.8),
                (28.0, 2.8),
                (38.0, 3.8),
                (48.0, 4.8),
                (58.0, 5.8),
            ]),
            molecule_id: mol_ids[2],
        },
        NewSpectrum {
            name: "ibuprofen_spectrum".to_string(),
            raw_name: "ibuprofen_spectrum".to_string(),
            source_file: "test".to_string(),
            spectrum_hash: "test_hash_0003".to_string(),
            precursor_mz: 207.0,
            num_peaks: 5,
            peaks: Peaks(vec![
                (22.0, 1.2),
                (32.0, 2.2),
                (42.0, 3.2),
                (52.0, 4.2),
                (62.0, 5.2),
            ]),
            molecule_id: mol_ids[3],
        },
        NewSpectrum {
            name: "testosterone_spectrum".to_string(),
            raw_name: "testosterone_spectrum".to_string(),
            source_file: "test".to_string(),
            spectrum_hash: "test_hash_0004".to_string(),
            precursor_mz: 289.0,
            num_peaks: 5,
            peaks: Peaks(vec![
                (12.0, 1.1),
                (23.0, 2.1),
                (34.0, 3.1),
                (45.5, 4.1),
                (56.0, 5.1),
            ]),
            molecule_id: mol_ids[4],
        },
    ];

    for spectrum in &test_spectra {
        diesel::insert_into(spectra::table)
            .values(spectrum)
            .on_conflict(spectra::spectrum_hash)
            .do_nothing()
            .execute(conn)
            .expect("failed to insert test spectrum");
    }
}
