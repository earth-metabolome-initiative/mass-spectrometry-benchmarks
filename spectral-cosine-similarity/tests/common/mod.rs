use diesel::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use spectral_cosine_similarity::{db, prepare};

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

pub fn pesticide_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("pesticides.mgf")
}

pub fn prepare_small_dataset(conn: &mut SqliteConnection, max_spectra: usize) {
    let fixture = pesticide_fixture_path();
    let sources: [(&Path, &str); 1] = [(fixture.as_path(), "pesticides.mgf")];
    prepare::run_with_sources(conn, max_spectra, &sources);
}
