use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;

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

    // Seed libraries
    let rust_lib_id = ensure_rust_library(conn);
    let matchms_lib_id = ensure_matchms_library(conn);

    // Seed implementations (same algorithm can have multiple implementations)
    ensure_implementation(conn, cosine_hungarian_id, rust_lib_id);
    ensure_implementation(conn, cosine_hungarian_id, matchms_lib_id);
    ensure_implementation(conn, cosine_greedy_id, matchms_lib_id);

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
    if let Ok(content) = std::fs::read_to_string(lock_path) {
        let mut in_mass_spec = false;
        for line in content.lines() {
            if line.starts_with("name = \"mass_spectrometry\"") {
                in_mass_spec = true;
            } else if in_mass_spec && line.starts_with("version = ") {
                return line
                    .trim_start_matches("version = ")
                    .trim_matches('"')
                    .to_string();
            } else if in_mass_spec && line.starts_with("[[") {
                break;
            }
        }
    }
    "unknown".to_string()
}

fn rust_lib_git_commit() -> Option<String> {
    let lock_path = "Cargo.lock";
    if let Ok(content) = std::fs::read_to_string(lock_path) {
        let mut in_mass_spec = false;
        for line in content.lines() {
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
