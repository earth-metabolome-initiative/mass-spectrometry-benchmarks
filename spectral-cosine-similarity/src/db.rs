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
const MS_ENTROPY_LIB_NAME: &str = "ms_entropy";

const N_WARMUP: u32 = 3;
const N_REPS: u32 = 5;

/// Parameter sets: (tolerance, mz_power, intensity_power)
const PARAM_SETS: [(f64, f64, f64); 1] = [
    (0.01, 0.0, 1.0), // single benchmark default
];

pub fn db_path() -> &'static str {
    DB_PATH
}

fn apply_sqlite_pragmas(conn: &mut SqliteConnection) {
    sql_query("PRAGMA journal_mode = WAL")
        .execute(conn)
        .expect("failed to enable WAL journal mode");
    sql_query("PRAGMA synchronous = NORMAL")
        .execute(conn)
        .expect("failed to set PRAGMA synchronous");
    sql_query("PRAGMA busy_timeout = 5000")
        .execute(conn)
        .expect("failed to set PRAGMA busy_timeout");
    sql_query("PRAGMA foreign_keys = ON")
        .execute(conn)
        .expect("failed to enable PRAGMA foreign_keys");
    sql_query("PRAGMA ignore_check_constraints = OFF")
        .execute(conn)
        .expect("failed to set PRAGMA ignore_check_constraints");
}

pub fn establish_connection() -> SqliteConnection {
    let path = DB_PATH;
    let mut conn = SqliteConnection::establish(path)
        .unwrap_or_else(|e| panic!("Error connecting to {path}: {e}"));
    apply_sqlite_pragmas(&mut conn);
    conn
}

pub fn establish_connection_at(path: &Path) -> SqliteConnection {
    let path_str = path.to_string_lossy();
    let mut conn = SqliteConnection::establish(path_str.as_ref())
        .unwrap_or_else(|e| panic!("Error connecting to {}: {e}", path.display()));
    apply_sqlite_pragmas(&mut conn);
    conn
}

pub fn initialize(conn: &mut SqliteConnection) {
    apply_sqlite_pragmas(conn);

    for statement in SCHEMA_SQL.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty()
            && let Err(e) = sql_query(trimmed).execute(conn)
        {
            panic!("Failed to execute schema statement: {e}\n{trimmed}");
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
        Some("Exact precursor-shift-aware modified cosine similarity"),
    );
    let modified_greedy_cosine_id = ensure_algorithm(
        conn,
        "ModifiedGreedyCosine",
        Some("Greedy precursor-shift-aware modified cosine similarity"),
    );
    let entropy_weighted_id = ensure_algorithm(
        conn,
        "EntropySimilarityWeighted",
        Some("Weighted spectral entropy similarity"),
    );
    let entropy_unweighted_id = ensure_algorithm(
        conn,
        "EntropySimilarityUnweighted",
        Some("Unweighted spectral entropy similarity"),
    );
    let linear_cosine_id = ensure_algorithm(
        conn,
        "LinearCosine",
        Some("Linear-time cosine similarity for well-separated spectra"),
    );
    let modified_linear_cosine_id = ensure_algorithm(
        conn,
        "ModifiedLinearCosine",
        Some("Linear-time modified cosine similarity for well-separated spectra"),
    );
    set_algorithm_approximation(conn, cosine_hungarian_id, None);
    set_algorithm_approximation(conn, cosine_greedy_id, Some(cosine_hungarian_id));
    set_algorithm_approximation(conn, linear_cosine_id, Some(cosine_hungarian_id));
    set_algorithm_approximation(conn, modified_cosine_id, None);
    set_algorithm_approximation(conn, modified_greedy_cosine_id, Some(modified_cosine_id));
    set_algorithm_approximation(conn, modified_linear_cosine_id, Some(modified_cosine_id));
    set_algorithm_approximation(conn, entropy_weighted_id, None);
    set_algorithm_approximation(conn, entropy_unweighted_id, None);

    // Seed libraries
    let rust_version = rust_lib_version();
    let rust_git_commit = rust_lib_git_commit();
    let rust_lib_id = ensure_library(
        conn,
        RUST_LIB_NAME,
        &rust_version,
        rust_git_commit.as_deref(),
        Some(RUST_LIB_GIT_URL),
        "rust",
    );
    let matchms_lib_id = ensure_library(
        conn,
        MATCHMS_LIB_NAME,
        &python_package_version("matchms"),
        None,
        Some("https://github.com/matchms/matchms"),
        "python",
    );
    let ms_entropy_lib_id = ensure_library(
        conn,
        MS_ENTROPY_LIB_NAME,
        &python_package_version("ms_entropy"),
        None,
        Some("https://github.com/YuanyueLi/MSEntropy"),
        "python",
    );

    // Seed implementations (same algorithm can have multiple implementations)
    ensure_implementation(conn, cosine_hungarian_id, rust_lib_id, false);
    ensure_implementation(conn, cosine_hungarian_id, matchms_lib_id, true);
    ensure_implementation(conn, cosine_greedy_id, rust_lib_id, false);
    ensure_implementation(conn, cosine_greedy_id, matchms_lib_id, true);
    ensure_implementation(conn, modified_cosine_id, rust_lib_id, true);
    ensure_implementation(conn, modified_greedy_cosine_id, rust_lib_id, false);
    ensure_implementation(conn, modified_greedy_cosine_id, matchms_lib_id, false);
    ensure_implementation(conn, entropy_weighted_id, rust_lib_id, false);
    ensure_implementation(conn, entropy_weighted_id, ms_entropy_lib_id, true);
    ensure_implementation(conn, entropy_unweighted_id, rust_lib_id, false);
    ensure_implementation(conn, entropy_unweighted_id, ms_entropy_lib_id, true);
    ensure_implementation(conn, linear_cosine_id, rust_lib_id, false);
    ensure_implementation(conn, modified_linear_cosine_id, rust_lib_id, false);

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
        .values(&NewAlgorithm {
            name,
            description,
            approximates_algorithm_id: None,
        })
        .returning(algorithms::id)
        .get_result::<i32>(conn)
        .expect("failed to insert algorithm")
}

fn set_algorithm_approximation(
    conn: &mut SqliteConnection,
    algorithm_id: i32,
    approximates_algorithm_id: Option<i32>,
) {
    if approximates_algorithm_id == Some(algorithm_id) {
        panic!("algorithm {algorithm_id} cannot approximate itself");
    }
    diesel::update(algorithms::table.filter(algorithms::id.eq(algorithm_id)))
        .set(algorithms::approximates_algorithm_id.eq(approximates_algorithm_id))
        .execute(conn)
        .expect("failed to set algorithm approximation relationship");
}

fn ensure_library(
    conn: &mut SqliteConnection,
    name: &str,
    version: &str,
    git_commit: Option<&str>,
    git_url: Option<&str>,
    language: &str,
) -> i32 {
    if let Some(lib) = libraries::table
        .filter(libraries::name.eq(name))
        .filter(libraries::version.eq(version))
        .first::<Library>(conn)
        .optional()
        .expect("query failed")
    {
        return lib.id;
    }

    diesel::insert_into(libraries::table)
        .values(&NewLibrary {
            name,
            version,
            git_commit,
            git_url,
            language,
        })
        .returning(libraries::id)
        .get_result::<i32>(conn)
        .expect("failed to insert library")
}

fn ensure_implementation(
    conn: &mut SqliteConnection,
    algorithm_id: i32,
    library_id: i32,
    is_reference: bool,
) -> i32 {
    if is_reference {
        diesel::update(
            implementations::table.filter(implementations::algorithm_id.eq(algorithm_id)),
        )
        .set(implementations::is_reference.eq(false))
        .execute(conn)
        .expect("failed to clear previous reference implementation");
    }

    if let Some(imp) = implementations::table
        .filter(implementations::algorithm_id.eq(algorithm_id))
        .filter(implementations::library_id.eq(library_id))
        .first::<Implementation>(conn)
        .optional()
        .expect("query failed")
    {
        if imp.is_reference != is_reference {
            diesel::update(implementations::table.filter(implementations::id.eq(imp.id)))
                .set(implementations::is_reference.eq(is_reference))
                .execute(conn)
                .expect("failed to update implementation reference marker");
        }
        return imp.id;
    }

    diesel::insert_into(implementations::table)
        .values(&NewImplementation {
            algorithm_id,
            library_id,
            is_reference,
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

fn python_package_version(package: &str) -> String {
    std::process::Command::new("uv")
        .args([
            "run",
            "python3",
            "-c",
            &format!("import {package}; print({package}.__version__)"),
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
        .order(experiments::id.asc())
        .load::<Experiment>(conn)
        .expect("failed to load experiments")
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::dsl::count_star;
    use std::collections::BTreeMap;

    fn setup_in_memory_connection() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:")
            .expect("failed to open in-memory sqlite connection");
        apply_sqlite_pragmas(&mut conn);
        conn
    }

    fn seed_two_test_spectra(conn: &mut SqliteConnection) -> (i32, i32) {
        sql_query(
            "INSERT INTO spectra
                (name, raw_name, source_file, spectrum_hash, precursor_mz, num_peaks, peaks)
             VALUES
                ('spec_left', 'spec_left', 'test.mgf', 'test_hash_left', 100.0, 2, '[[50.0, 0.5], [60.0, 0.5]]'),
                ('spec_right', 'spec_right', 'test.mgf', 'test_hash_right', 200.0, 2, '[[70.0, 0.5], [80.0, 0.5]]')",
        )
        .execute(conn)
        .expect("failed to seed test spectra");

        let left_id = spectra::table
            .filter(spectra::spectrum_hash.eq("test_hash_left"))
            .select(spectra::id)
            .first::<i32>(conn)
            .expect("failed to load left test spectrum id");
        let right_id = spectra::table
            .filter(spectra::spectrum_hash.eq("test_hash_right"))
            .select(spectra::id)
            .first::<i32>(conn)
            .expect("failed to load right test spectrum id");
        (left_id, right_id)
    }

    fn first_experiment_id(conn: &mut SqliteConnection) -> i32 {
        experiments::table
            .order(experiments::id.asc())
            .select(experiments::id)
            .first::<i32>(conn)
            .expect("failed to load first experiment id")
    }

    fn first_implementation_id(conn: &mut SqliteConnection) -> i32 {
        implementations::table
            .order(implementations::id.asc())
            .select(implementations::id)
            .first::<i32>(conn)
            .expect("failed to load first implementation id")
    }

    fn first_two_implementation_ids(conn: &mut SqliteConnection) -> (i32, i32) {
        let ids: Vec<i32> = implementations::table
            .order(implementations::id.asc())
            .select(implementations::id)
            .limit(2)
            .load(conn)
            .expect("failed to load implementation ids");
        assert_eq!(ids.len(), 2, "expected at least two implementations");
        (ids[0], ids[1])
    }

    #[test]
    fn results_reject_invalid_foreign_keys_when_pragmas_enabled() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let bad_insert = diesel::insert_into(results::table)
            .values(&NewResult {
                left_id: 999_999,
                right_id: 999_999,
                experiment_id: first_experiment_id(&mut conn),
                implementation_id: first_implementation_id(&mut conn),
                score: 0.5,
                matches: 0,
                median_time_us: 1.0,
            })
            .execute(&mut conn);

        assert!(
            bad_insert.is_err(),
            "foreign key violation should be rejected"
        );
    }

    #[test]
    fn results_enforce_pair_ordering_check() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);
        let (left_id, right_id) = seed_two_test_spectra(&mut conn);

        let bad_insert = diesel::insert_into(results::table)
            .values(&NewResult {
                left_id: right_id,
                right_id: left_id,
                experiment_id: first_experiment_id(&mut conn),
                implementation_id: first_implementation_id(&mut conn),
                score: 0.5,
                matches: 0,
                median_time_us: 1.0,
            })
            .execute(&mut conn);

        assert!(bad_insert.is_err(), "left_id > right_id must be rejected");
    }

    #[test]
    fn results_enforce_score_bounds() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);
        let (left_id, right_id) = seed_two_test_spectra(&mut conn);
        let experiment_id = first_experiment_id(&mut conn);
        let (implementation_id_a, implementation_id_b) = first_two_implementation_ids(&mut conn);

        let above_one = diesel::insert_into(results::table)
            .values(&NewResult {
                left_id,
                right_id,
                experiment_id,
                implementation_id: implementation_id_a,
                score: 1.1,
                matches: 0,
                median_time_us: 1.0,
            })
            .execute(&mut conn);
        assert!(above_one.is_err(), "score > 1.000001 must be rejected");

        let below_zero = diesel::insert_into(results::table)
            .values(&NewResult {
                left_id,
                right_id,
                experiment_id,
                implementation_id: implementation_id_b,
                score: -0.0001,
                matches: 0,
                median_time_us: 1.0,
            })
            .execute(&mut conn);
        assert!(below_zero.is_err(), "score < 0 must be rejected");
    }

    #[test]
    fn results_allow_entropy_matches_sentinel_and_reject_lower_values() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);
        let (left_id, right_id) = seed_two_test_spectra(&mut conn);
        let experiment_id = first_experiment_id(&mut conn);
        let (implementation_id_a, implementation_id_b) = first_two_implementation_ids(&mut conn);

        let sentinel = diesel::insert_into(results::table)
            .values(&NewResult {
                left_id,
                right_id,
                experiment_id,
                implementation_id: implementation_id_a,
                score: 0.5,
                matches: -1,
                median_time_us: 1.0,
            })
            .execute(&mut conn);
        assert!(sentinel.is_ok(), "matches = -1 should be allowed");

        let invalid = diesel::insert_into(results::table)
            .values(&NewResult {
                left_id,
                right_id,
                experiment_id,
                implementation_id: implementation_id_b,
                score: 0.5,
                matches: -2,
                median_time_us: 1.0,
            })
            .execute(&mut conn);
        assert!(invalid.is_err(), "matches < -1 must be rejected");
    }

    #[test]
    fn spectra_constraints_reject_absurd_rows() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let bad_num_peaks = sql_query(
            "INSERT INTO spectra
                (name, raw_name, source_file, spectrum_hash, precursor_mz, num_peaks, peaks)
             VALUES
                ('bad_num_peaks', 'bad_num_peaks', 'test.mgf', 'bad_hash_num_peaks', 100.0, 0, '[[50.0, 1.0]]')",
        )
        .execute(&mut conn);
        assert!(bad_num_peaks.is_err(), "num_peaks <= 0 must be rejected");

        let bad_precursor = sql_query(
            "INSERT INTO spectra
                (name, raw_name, source_file, spectrum_hash, precursor_mz, num_peaks, peaks)
             VALUES
                ('bad_precursor', 'bad_precursor', 'test.mgf', 'bad_hash_precursor', 0.0, 2, '[[50.0, 1.0]]')",
        )
        .execute(&mut conn);
        assert!(bad_precursor.is_err(), "precursor_mz <= 0 must be rejected");

        let bad_peaks_json = sql_query(
            "INSERT INTO spectra
                (name, raw_name, source_file, spectrum_hash, precursor_mz, num_peaks, peaks)
             VALUES
                ('bad_json', 'bad_json', 'test.mgf', 'bad_hash_json', 150.0, 2, 'not-json')",
        )
        .execute(&mut conn);
        assert!(
            bad_peaks_json.is_err(),
            "invalid peaks JSON must be rejected"
        );

        let blank_name = sql_query(
            "INSERT INTO spectra
                (name, raw_name, source_file, spectrum_hash, precursor_mz, num_peaks, peaks)
             VALUES
                ('   ', 'blank', 'test.mgf', 'bad_hash_blank_name', 150.0, 2, '[[50.0, 1.0]]')",
        )
        .execute(&mut conn);
        assert!(blank_name.is_err(), "blank spectrum name must be rejected");
    }

    #[test]
    fn libraries_and_experiments_reject_invalid_values() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let bad_language = sql_query(
            "INSERT INTO libraries (name, version, git_commit, git_url, language)
             VALUES ('invalid-lib', '0.0.1', NULL, NULL, 'go')",
        )
        .execute(&mut conn);
        assert!(
            bad_language.is_err(),
            "unsupported library language must be rejected"
        );

        let bad_experiment =
            sql_query("INSERT INTO experiments (params) VALUES ('not-json')").execute(&mut conn);
        assert!(
            bad_experiment.is_err(),
            "invalid experiment JSON must be rejected"
        );
    }

    #[test]
    fn algorithms_reject_self_approximation() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let algorithm_id = algorithms::table
            .order(algorithms::id.asc())
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .expect("failed to load an algorithm id");

        let update = sql_query(format!(
            "UPDATE algorithms SET approximates_algorithm_id = {algorithm_id} WHERE id = {algorithm_id}"
        ))
        .execute(&mut conn);
        assert!(
            update.is_err(),
            "self-approximation should be rejected by CHECK"
        );
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

        assert_eq!(algorithm_count, 8);
        assert_eq!(implementation_count, 13);
        assert_eq!(experiment_count, PARAM_SETS.len() as i64);
    }

    #[test]
    fn resolves_seeded_implementation_ids() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let rust_hungarian =
            get_implementation_id(&mut conn, "CosineHungarian", "mass-spectrometry-traits");
        let matchms_hungarian = get_implementation_id(&mut conn, "CosineHungarian", "matchms");
        let rust_greedy =
            get_implementation_id(&mut conn, "CosineGreedy", "mass-spectrometry-traits");
        let matchms_greedy = get_implementation_id(&mut conn, "CosineGreedy", "matchms");
        let rust_modified =
            get_implementation_id(&mut conn, "ModifiedCosine", "mass-spectrometry-traits");
        let rust_modified_greedy = get_implementation_id(
            &mut conn,
            "ModifiedGreedyCosine",
            "mass-spectrometry-traits",
        );
        let matchms_modified_greedy =
            get_implementation_id(&mut conn, "ModifiedGreedyCosine", "matchms");
        let rust_entropy_weighted = get_implementation_id(
            &mut conn,
            "EntropySimilarityWeighted",
            "mass-spectrometry-traits",
        );
        let py_entropy_weighted =
            get_implementation_id(&mut conn, "EntropySimilarityWeighted", "ms_entropy");
        let rust_entropy_unweighted = get_implementation_id(
            &mut conn,
            "EntropySimilarityUnweighted",
            "mass-spectrometry-traits",
        );
        let py_entropy_unweighted =
            get_implementation_id(&mut conn, "EntropySimilarityUnweighted", "ms_entropy");

        assert_ne!(rust_hungarian, matchms_hungarian);
        assert_ne!(rust_greedy, matchms_greedy);
        assert_ne!(rust_hungarian, rust_greedy);
        assert_ne!(matchms_hungarian, matchms_greedy);
        assert_ne!(rust_hungarian, matchms_greedy);
        assert_ne!(matchms_hungarian, rust_greedy);
        assert_ne!(rust_modified_greedy, matchms_modified_greedy);
        assert_ne!(rust_modified, rust_modified_greedy);
        assert_ne!(rust_hungarian, rust_modified);
        assert_ne!(matchms_hungarian, matchms_modified_greedy);
        assert_ne!(matchms_greedy, matchms_modified_greedy);
        assert_ne!(rust_entropy_weighted, py_entropy_weighted);
        assert_ne!(rust_entropy_unweighted, py_entropy_unweighted);
        assert_ne!(rust_entropy_weighted, rust_entropy_unweighted);
        assert_ne!(py_entropy_weighted, py_entropy_unweighted);
    }

    #[test]
    fn loads_experiments_in_id_order() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let loaded = load_experiments(&mut conn);
        assert!(
            loaded.windows(2).all(|w| w[0].id < w[1].id),
            "experiments should be returned in ascending id order"
        );
    }

    #[test]
    fn seeds_reference_counts_are_valid() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let rows: Vec<(i32, bool)> = implementations::table
            .select((implementations::algorithm_id, implementations::is_reference))
            .load(&mut conn)
            .expect("failed to load implementation reference flags");

        let mut refs_by_algorithm: BTreeMap<i32, usize> = BTreeMap::new();
        for (algorithm_id, is_reference) in rows {
            if is_reference {
                *refs_by_algorithm.entry(algorithm_id).or_insert(0) += 1;
            } else {
                refs_by_algorithm.entry(algorithm_id).or_insert(0);
            }
        }

        assert_eq!(refs_by_algorithm.len(), 8);
        assert!(
            refs_by_algorithm.values().all(|&n| n <= 1),
            "expected at most one reference implementation per algorithm, got {refs_by_algorithm:?}"
        );

        let canonical_algorithm_ids: Vec<i32> = algorithms::table
            .filter(algorithms::approximates_algorithm_id.is_null())
            .order(algorithms::id.asc())
            .select(algorithms::id)
            .load(&mut conn)
            .expect("failed to load canonical algorithms");

        for algorithm_id in canonical_algorithm_ids {
            let count = refs_by_algorithm.get(&algorithm_id).copied().unwrap_or(0);
            assert_eq!(
                count, 1,
                "canonical algorithm id {algorithm_id} must have exactly one reference implementation"
            );
        }

        let modified_greedy_cosine_id = algorithms::table
            .filter(algorithms::name.eq("ModifiedGreedyCosine"))
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .expect("failed to load ModifiedGreedyCosine id");
        assert_eq!(
            refs_by_algorithm
                .get(&modified_greedy_cosine_id)
                .copied()
                .unwrap_or(0),
            0
        );
    }

    #[test]
    fn seeds_cosine_greedy_as_approximation_of_cosine_hungarian() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let cosine_hungarian_id = algorithms::table
            .filter(algorithms::name.eq("CosineHungarian"))
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .expect("failed to load CosineHungarian id");

        let cosine_greedy_approx = algorithms::table
            .filter(algorithms::name.eq("CosineGreedy"))
            .select(algorithms::approximates_algorithm_id)
            .first::<Option<i32>>(&mut conn)
            .expect("failed to load CosineGreedy approximation target");

        assert_eq!(cosine_greedy_approx, Some(cosine_hungarian_id));

        let cosine_hungarian_approx = algorithms::table
            .filter(algorithms::name.eq("CosineHungarian"))
            .select(algorithms::approximates_algorithm_id)
            .first::<Option<i32>>(&mut conn)
            .expect("failed to load CosineHungarian approximation target");

        assert_eq!(cosine_hungarian_approx, None);
    }

    #[test]
    fn does_not_seed_modified_cosine_approx_algorithm() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let modified_cosine_approx = algorithms::table
            .filter(algorithms::name.eq("ModifiedCosineApprox"))
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .optional()
            .expect("failed to query ModifiedCosineApprox algorithm");
        assert_eq!(modified_cosine_approx, None);

        let rust_modified_ref = implementations::table
            .inner_join(algorithms::table)
            .inner_join(libraries::table)
            .filter(algorithms::name.eq("ModifiedCosine"))
            .filter(libraries::name.eq("mass-spectrometry-traits"))
            .select(implementations::is_reference)
            .first::<bool>(&mut conn)
            .expect("failed to load ModifiedCosine rust reference flag");
        assert!(rust_modified_ref);
    }

    #[test]
    fn seeds_modified_greedy_cosine_as_approximation_of_modified_cosine() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let modified_cosine_id = algorithms::table
            .filter(algorithms::name.eq("ModifiedCosine"))
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .expect("failed to load ModifiedCosine id");

        let modified_greedy_approx = algorithms::table
            .filter(algorithms::name.eq("ModifiedGreedyCosine"))
            .select(algorithms::approximates_algorithm_id)
            .first::<Option<i32>>(&mut conn)
            .expect("failed to load ModifiedGreedyCosine approximation target");

        assert_eq!(modified_greedy_approx, Some(modified_cosine_id));

        let rust_modified_greedy_ref = implementations::table
            .inner_join(algorithms::table)
            .inner_join(libraries::table)
            .filter(algorithms::name.eq("ModifiedGreedyCosine"))
            .filter(libraries::name.eq("mass-spectrometry-traits"))
            .select(implementations::is_reference)
            .first::<bool>(&mut conn)
            .expect("failed to load ModifiedGreedyCosine rust reference flag");
        assert!(!rust_modified_greedy_ref);

        let matchms_modified_greedy_ref = implementations::table
            .inner_join(algorithms::table)
            .inner_join(libraries::table)
            .filter(algorithms::name.eq("ModifiedGreedyCosine"))
            .filter(libraries::name.eq("matchms"))
            .select(implementations::is_reference)
            .first::<bool>(&mut conn)
            .expect("failed to load ModifiedGreedyCosine matchms reference flag");
        assert!(!matchms_modified_greedy_ref);
    }
}
