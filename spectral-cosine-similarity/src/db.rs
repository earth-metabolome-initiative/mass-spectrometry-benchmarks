use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;
use std::path::Path;

use crate::models::*;
use crate::schema::*;

const DB_PATH: &str = "fixtures/benchmark.db";
const SCHEMA_SQL: &str = include_str!("../fixtures/schema.sql");

const RUST_LIB_NAME: &str = "mass-spectrometry-traits";
const RUST_LIB_GIT_URL: &str =
    "https://github.com/earth-metabolome-initiative/mass-spectrometry-traits";

const MATCHMS_LIB_NAME: &str = "matchms";

const N_WARMUP: u32 = 3;
const N_REPS: u32 = 10;

/// Parameter sets: (tolerance, mz_power, intensity_power)
const PARAM_SETS: [(f32, f32, f32); 4] = [
    (0.1, 0.0, 1.0), // matchms defaults
    (0.1, 1.0, 1.0), // current Rust test params
    (0.5, 1.0, 0.5), // stress test
    (2.0, 0.0, 1.0), // wide tolerance
];

pub fn db_path(_max_spectra: Option<usize>) -> &'static str {
    DB_PATH
}

pub fn establish_connection(max_spectra: Option<usize>) -> SqliteConnection {
    let path = db_path(max_spectra);
    SqliteConnection::establish(path).unwrap_or_else(|e| panic!("Error connecting to {path}: {e}"))
}

pub fn establish_connection_at(path: &Path) -> SqliteConnection {
    let path_str = path.to_string_lossy();
    SqliteConnection::establish(path_str.as_ref())
        .unwrap_or_else(|e| panic!("Error connecting to {}: {e}", path.display()))
}

pub fn initialize(conn: &mut SqliteConnection) {
    // Run schema.sql (all CREATE IF NOT EXISTS, so idempotent)
    for statement in SCHEMA_SQL.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sql_query(trimmed)
                .execute(conn)
                .unwrap_or_else(|e| panic!("Failed to execute schema statement: {e}\n{trimmed}"));
        }
    }

    // Seed algorithms (implementation-agnostic)
    let cosine_hungarian_id = ensure_algorithm(
        conn,
        "CosineHungarian",
        Some("Hungarian algorithm-based cosine similarity"),
    );
    let cosine_greedy_id = ensure_algorithm(conn, "CosineGreedy", Some("Greedy cosine similarity"));
    let modified_cosine_id = ensure_algorithm(
        conn,
        "ModifiedCosine",
        Some("Precursor-shift-aware modified cosine similarity"),
    );

    // Seed libraries
    let rust_lib_id = ensure_rust_library(conn);
    let matchms_lib_id = ensure_matchms_library(conn);

    // Seed implementations (same algorithm can have multiple implementations)
    ensure_implementation(conn, cosine_hungarian_id, rust_lib_id);
    ensure_implementation(conn, cosine_hungarian_id, matchms_lib_id);
    ensure_implementation(conn, cosine_greedy_id, matchms_lib_id);
    ensure_implementation(conn, modified_cosine_id, rust_lib_id);
    ensure_implementation(conn, modified_cosine_id, matchms_lib_id);

    // Seed experiments
    for (tolerance, mz_power, intensity_power) in PARAM_SETS {
        let params = ExperimentParams {
            tolerance,
            mz_power,
            intensity_power,
            n_warmup: N_WARMUP,
            n_reps: N_REPS,
        };
        ensure_experiment(conn, &params);
    }
}

fn ensure_algorithm(conn: &mut SqliteConnection, name: &str, description: Option<&str>) -> i32 {
    if let Some(algo) = algorithms::table
        .filter(algorithms::name.eq(name))
        .first::<Algorithm>(conn)
        .optional()
        .expect("query failed")
    {
        return algo.id;
    }

    diesel::insert_into(algorithms::table)
        .values(&NewAlgorithm { name, description })
        .returning(algorithms::id)
        .get_result::<i32>(conn)
        .expect("failed to insert algorithm")
}

fn ensure_rust_library(conn: &mut SqliteConnection) -> i32 {
    let version = rust_lib_version();
    let git_commit = rust_lib_git_commit();

    if let Some(lib) = libraries::table
        .filter(libraries::name.eq(RUST_LIB_NAME))
        .filter(libraries::version.eq(&version))
        .first::<Library>(conn)
        .optional()
        .expect("query failed")
    {
        return lib.id;
    }

    diesel::insert_into(libraries::table)
        .values(&NewLibrary {
            name: RUST_LIB_NAME,
            version: &version,
            git_commit: git_commit.as_deref(),
            git_url: Some(RUST_LIB_GIT_URL),
            language: "rust",
        })
        .returning(libraries::id)
        .get_result::<i32>(conn)
        .expect("failed to insert rust library")
}

fn ensure_matchms_library(conn: &mut SqliteConnection) -> i32 {
    let version = matchms_version();

    if let Some(lib) = libraries::table
        .filter(libraries::name.eq(MATCHMS_LIB_NAME))
        .filter(libraries::version.eq(&version))
        .first::<Library>(conn)
        .optional()
        .expect("query failed")
    {
        return lib.id;
    }

    diesel::insert_into(libraries::table)
        .values(&NewLibrary {
            name: MATCHMS_LIB_NAME,
            version: &version,
            git_commit: None,
            git_url: Some("https://github.com/matchms/matchms"),
            language: "python",
        })
        .returning(libraries::id)
        .get_result::<i32>(conn)
        .expect("failed to insert matchms library")
}

fn ensure_implementation(conn: &mut SqliteConnection, algorithm_id: i32, library_id: i32) -> i32 {
    if let Some(imp) = implementations::table
        .filter(implementations::algorithm_id.eq(algorithm_id))
        .filter(implementations::library_id.eq(library_id))
        .first::<Implementation>(conn)
        .optional()
        .expect("query failed")
    {
        return imp.id;
    }

    diesel::insert_into(implementations::table)
        .values(&NewImplementation {
            algorithm_id,
            library_id,
        })
        .returning(implementations::id)
        .get_result::<i32>(conn)
        .expect("failed to insert implementation")
}

fn ensure_experiment(conn: &mut SqliteConnection, params: &ExperimentParams) -> i32 {
    let json = serde_json::to_string(params).expect("failed to serialize params");

    if let Some(exp) = experiments::table
        .filter(experiments::params.eq(&json))
        .first::<Experiment>(conn)
        .optional()
        .expect("query failed")
    {
        return exp.id;
    }

    diesel::insert_into(experiments::table)
        .values(&NewExperiment { params: json })
        .returning(experiments::id)
        .get_result::<i32>(conn)
        .expect("failed to insert experiment")
}

fn rust_lib_version() -> String {
    let lock_path = "Cargo.lock";
    if let Ok(content) = std::fs::read_to_string(lock_path)
        && let Some(version) = extract_mass_spec_version(&content)
    {
        return version;
    }
    "unknown".to_string()
}

fn rust_lib_git_commit() -> Option<String> {
    let lock_path = "Cargo.lock";
    if let Ok(content) = std::fs::read_to_string(lock_path) {
        return extract_mass_spec_git_commit(&content);
    }
    None
}

pub(crate) fn extract_mass_spec_version(lock_content: &str) -> Option<String> {
    let mut in_mass_spec = false;
    for line in lock_content.lines() {
        if line.starts_with("name = \"mass_spectrometry\"") {
            in_mass_spec = true;
        } else if in_mass_spec && line.starts_with("version = ") {
            return Some(
                line.trim_start_matches("version = ")
                    .trim_matches('"')
                    .to_string(),
            );
        } else if in_mass_spec && line.starts_with("[[") {
            break;
        }
    }
    None
}

pub(crate) fn extract_mass_spec_git_commit(lock_content: &str) -> Option<String> {
    let mut in_mass_spec = false;
    for line in lock_content.lines() {
        if line.starts_with("name = \"mass_spectrometry\"") {
            in_mass_spec = true;
        } else if in_mass_spec && line.starts_with("source = ") {
            let source = line.trim_start_matches("source = ").trim_matches('"');
            if let Some(hash) = source.split('#').nth(1) {
                return Some(hash.to_string());
            }
        } else if in_mass_spec && line.starts_with("[[") {
            break;
        }
    }
    None
}

fn matchms_version() -> String {
    std::process::Command::new("uv")
        .args([
            "run",
            "python3",
            "-c",
            "import matchms; print(matchms.__version__)",
        ])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn get_implementation_id(conn: &mut SqliteConnection, algo_name: &str, lib_name: &str) -> i32 {
    implementations::table
        .inner_join(algorithms::table)
        .inner_join(libraries::table)
        .filter(algorithms::name.eq(algo_name))
        .filter(libraries::name.eq(lib_name))
        .select(implementations::id)
        .first::<i32>(conn)
        .unwrap_or_else(|_| panic!("implementation '{algo_name}' in '{lib_name}' not found"))
}

pub fn load_experiments(conn: &mut SqliteConnection) -> Vec<Experiment> {
    experiments::table
        .load::<Experiment>(conn)
        .expect("failed to load experiments")
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::dsl::count_star;

    fn setup_in_memory_connection() -> SqliteConnection {
        SqliteConnection::establish(":memory:").expect("failed to open in-memory sqlite connection")
    }

    #[test]
    fn extracts_mass_spec_version_and_commit() {
        let lock = r#"
[[package]]
name = "serde"
version = "1.0.0"

[[package]]
name = "mass_spectrometry"
version = "0.9.1"
source = "git+https://example.com/repo#abc123def"
"#;

        assert_eq!(extract_mass_spec_version(lock), Some("0.9.1".to_string()));
        assert_eq!(
            extract_mass_spec_git_commit(lock),
            Some("abc123def".to_string())
        );
    }

    #[test]
    fn initialize_is_idempotent_and_seeds_expected_rows() {
        let mut conn = setup_in_memory_connection();

        initialize(&mut conn);
        initialize(&mut conn);

        let algorithm_count = algorithms::table
            .select(count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count algorithms");
        let implementation_count = implementations::table
            .select(count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count implementations");
        let experiment_count = experiments::table
            .select(count_star())
            .first::<i64>(&mut conn)
            .expect("failed to count experiments");

        assert_eq!(algorithm_count, 3);
        assert_eq!(implementation_count, 5);
        assert_eq!(experiment_count, PARAM_SETS.len() as i64);
    }

    #[test]
    fn resolves_seeded_implementation_ids() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let rust_hungarian =
            get_implementation_id(&mut conn, "CosineHungarian", "mass-spectrometry-traits");
        let matchms_hungarian = get_implementation_id(&mut conn, "CosineHungarian", "matchms");
        let matchms_greedy = get_implementation_id(&mut conn, "CosineGreedy", "matchms");
        let rust_modified =
            get_implementation_id(&mut conn, "ModifiedCosine", "mass-spectrometry-traits");
        let matchms_modified = get_implementation_id(&mut conn, "ModifiedCosine", "matchms");

        assert_ne!(rust_hungarian, matchms_hungarian);
        assert_ne!(matchms_hungarian, matchms_greedy);
        assert_ne!(rust_hungarian, matchms_greedy);
        assert_ne!(rust_modified, matchms_modified);
        assert_ne!(rust_hungarian, rust_modified);
        assert_ne!(matchms_hungarian, matchms_modified);
        assert_ne!(matchms_greedy, matchms_modified);
    }
}
