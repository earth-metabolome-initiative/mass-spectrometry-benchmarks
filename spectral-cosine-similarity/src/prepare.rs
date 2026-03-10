use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::process::Command;

use crate::schema::*;

pub fn spectra_count(conn: &mut SqliteConnection) -> i64 {
    spectra::table
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .expect("failed to count spectra")
}

/// Load spectra from Mass Spec Gym via Python subprocess.
pub fn run(conn: &mut SqliteConnection) {
    let existing = spectra_count(conn);
    if existing > 0 {
        eprintln!("[prepare] {existing} spectra already in DB, skipping load");
        return;
    }

    let db_path = crate::db::db_path();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("python_load_massspecgym.py");

    eprintln!("[prepare] Loading spectra from Mass Spec Gym via Python...");
    let status = Command::new("uv")
        .args(["run", "python3", &script.to_string_lossy(), db_path])
        .status()
        .unwrap_or_else(|err| panic!("[prepare] failed to launch `uv run python3`: {err}"));

    if !status.success() {
        panic!(
            "[prepare] python_load_massspecgym.py exited with {status}. \
             Run `uv sync` in spectral-cosine-similarity/ and ensure \
             `huggingface_hub` and `skfp` import successfully."
        );
    }

    let new_count = spectra_count(conn);
    eprintln!("[prepare] DB now contains {new_count} spectra");
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_query;

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
                peaks TEXT NOT NULL,
                molecule_id INTEGER NOT NULL
            )",
        )
        .execute(&mut conn)
        .expect("failed to create test schema");

        conn
    }

    #[test]
    fn spectra_count_returns_zero_for_empty_table() {
        let mut conn = setup_in_memory_connection();
        assert_eq!(spectra_count(&mut conn), 0);
    }
}
