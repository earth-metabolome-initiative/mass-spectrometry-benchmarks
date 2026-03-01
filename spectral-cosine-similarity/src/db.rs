use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Text};
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
const N_REPS: u32 = 10;

/// Parameter sets: (tolerance, mz_power, intensity_power)
const PARAM_SETS: [(f32, f32, f32); 4] = [
    (0.1, 0.0, 1.0), // matchms defaults
    (0.1, 1.0, 1.0), // current Rust test params
    (0.5, 1.0, 0.5), // stress test
    (2.0, 0.0, 1.0), // wide tolerance
];

#[derive(QueryableByName)]
struct TableInfoRow {
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

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
            if let Err(e) = sql_query(trimmed).execute(conn) {
                let err_msg = e.to_string();
                let is_legacy_reference_index_statement = trimmed
                    .contains("idx_implementations_one_reference_per_algorithm")
                    && err_msg.contains("no such column: is_reference");
                let is_legacy_topology_view_statement = trimmed
                    .contains("v_implementation_topology")
                    && (err_msg.contains("no such column: i.is_reference")
                        || err_msg.contains("no such column: a.approximates_algorithm_id"));

                if !is_legacy_reference_index_statement && !is_legacy_topology_view_statement {
                    panic!("Failed to execute schema statement: {e}\n{trimmed}");
                }
            }
        }
    }
    ensure_algorithms_approximation_schema(conn);
    ensure_implementations_reference_schema(conn);
    ensure_spectra_hash_schema(conn);

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
    let modified_cosine_approx_id = ensure_algorithm(
        conn,
        "ModifiedCosineApprox",
        Some("Approximate precursor-shift-aware modified cosine similarity"),
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
    set_algorithm_approximation(conn, cosine_hungarian_id, None);
    set_algorithm_approximation(conn, cosine_greedy_id, Some(cosine_hungarian_id));
    set_algorithm_approximation(conn, modified_cosine_id, None);
    set_algorithm_approximation(conn, modified_greedy_cosine_id, Some(modified_cosine_id));
    set_algorithm_approximation(conn, modified_cosine_approx_id, Some(modified_cosine_id));
    set_algorithm_approximation(conn, entropy_weighted_id, None);
    set_algorithm_approximation(conn, entropy_unweighted_id, None);

    // Seed libraries
    let rust_lib_id = ensure_rust_library(conn);
    let matchms_lib_id = ensure_matchms_library(conn);
    let ms_entropy_lib_id = ensure_ms_entropy_library(conn);

    migrate_legacy_modified_cosine_matchms_implementation(
        conn,
        modified_cosine_id,
        modified_cosine_approx_id,
        matchms_lib_id,
    );

    // Seed implementations (same algorithm can have multiple implementations)
    ensure_implementation(conn, cosine_hungarian_id, rust_lib_id, false);
    ensure_implementation(conn, cosine_hungarian_id, matchms_lib_id, true);
    ensure_implementation(conn, cosine_greedy_id, rust_lib_id, false);
    ensure_implementation(conn, cosine_greedy_id, matchms_lib_id, true);
    ensure_implementation(conn, modified_cosine_id, rust_lib_id, true);
    ensure_implementation(conn, modified_greedy_cosine_id, rust_lib_id, false);
    ensure_implementation(conn, modified_greedy_cosine_id, matchms_lib_id, false);
    ensure_implementation(conn, modified_cosine_approx_id, matchms_lib_id, false);
    ensure_implementation(conn, entropy_weighted_id, rust_lib_id, false);
    ensure_implementation(conn, entropy_weighted_id, ms_entropy_lib_id, true);
    ensure_implementation(conn, entropy_unweighted_id, rust_lib_id, false);
    ensure_implementation(conn, entropy_unweighted_id, ms_entropy_lib_id, true);

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

fn table_has_column(conn: &mut SqliteConnection, table: &str, column: &str) -> bool {
    let pragma = format!("PRAGMA table_info({table})");
    let rows: Vec<TableInfoRow> = sql_query(pragma)
        .load(conn)
        .unwrap_or_else(|e| panic!("failed to inspect table '{table}': {e}"));
    rows.iter().any(|r| r.name == column)
}

fn algorithms_has_column(conn: &mut SqliteConnection, column: &str) -> bool {
    table_has_column(conn, "algorithms", column)
}

fn implementations_has_column(conn: &mut SqliteConnection, column: &str) -> bool {
    table_has_column(conn, "implementations", column)
}

fn ensure_algorithms_approximation_schema(conn: &mut SqliteConnection) {
    if !algorithms_has_column(conn, "approximates_algorithm_id") {
        sql_query(
            "ALTER TABLE algorithms
             ADD COLUMN approximates_algorithm_id INTEGER REFERENCES algorithms(id)",
        )
        .execute(conn)
        .expect("failed to add algorithms.approximates_algorithm_id column");
    }
}

fn ensure_implementations_reference_schema(conn: &mut SqliteConnection) {
    if !implementations_has_column(conn, "is_reference") {
        sql_query(
            "ALTER TABLE implementations
             ADD COLUMN is_reference INTEGER NOT NULL DEFAULT 0 CHECK (is_reference IN (0, 1))",
        )
        .execute(conn)
        .expect("failed to add implementations.is_reference column");
    }

    sql_query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_implementations_one_reference_per_algorithm
         ON implementations(algorithm_id)
         WHERE is_reference = 1",
    )
    .execute(conn)
    .expect("failed to create reference uniqueness index");

    let duplicate_refs: CountRow = sql_query(
        "SELECT COUNT(*) AS n
         FROM (
             SELECT algorithm_id
             FROM implementations
             WHERE is_reference = 1
             GROUP BY algorithm_id
             HAVING COUNT(*) > 1
         )",
    )
    .get_result(conn)
    .expect("failed to validate implementations reference uniqueness");
    if duplicate_refs.n > 0 {
        panic!(
            "found {} algorithms with multiple reference implementations; \
repair the data before continuing",
            duplicate_refs.n
        );
    }
}

fn ensure_spectra_hash_schema(conn: &mut SqliteConnection) {
    if !table_has_column(conn, "spectra", "spectrum_hash") {
        sql_query("ALTER TABLE spectra ADD COLUMN spectrum_hash TEXT")
            .execute(conn)
            .expect("failed to add spectra.spectrum_hash column");
    }

    sql_query(
        "UPDATE spectra
         SET spectrum_hash = printf('__legacy_spectrum_%d', id)
         WHERE spectrum_hash IS NULL OR spectrum_hash = ''",
    )
    .execute(conn)
    .expect("failed to backfill spectra.spectrum_hash for legacy rows");

    sql_query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_spectra_spectrum_hash_unique
         ON spectra(spectrum_hash)",
    )
    .execute(conn)
    .expect("failed to create spectra.spectrum_hash unique index");
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

fn migrate_legacy_modified_cosine_matchms_implementation(
    conn: &mut SqliteConnection,
    modified_cosine_id: i32,
    modified_cosine_approx_id: i32,
    matchms_library_id: i32,
) {
    let legacy_impl = implementations::table
        .filter(implementations::algorithm_id.eq(modified_cosine_id))
        .filter(implementations::library_id.eq(matchms_library_id))
        .first::<Implementation>(conn)
        .optional()
        .expect("failed to query legacy modified cosine implementation");

    let Some(legacy_impl) = legacy_impl else {
        return;
    };

    let approx_impl = implementations::table
        .filter(implementations::algorithm_id.eq(modified_cosine_approx_id))
        .filter(implementations::library_id.eq(matchms_library_id))
        .first::<Implementation>(conn)
        .optional()
        .expect("failed to query modified cosine approximation implementation");

    if let Some(approx_impl) = approx_impl {
        sql_query(
            "DELETE FROM results
             WHERE implementation_id = ?
               AND EXISTS (
                   SELECT 1
                   FROM results r2
                   WHERE r2.left_id = results.left_id
                     AND r2.right_id = results.right_id
                     AND r2.experiment_id = results.experiment_id
                     AND r2.implementation_id = ?
               )",
        )
        .bind::<Integer, _>(legacy_impl.id)
        .bind::<Integer, _>(approx_impl.id)
        .execute(conn)
        .expect("failed to drop duplicate legacy modified cosine results");

        diesel::update(results::table.filter(results::implementation_id.eq(legacy_impl.id)))
            .set(results::implementation_id.eq(approx_impl.id))
            .execute(conn)
            .expect("failed to remap legacy modified cosine results");

        diesel::delete(implementations::table.filter(implementations::id.eq(legacy_impl.id)))
            .execute(conn)
            .expect("failed to delete legacy modified cosine implementation");
    } else {
        diesel::update(implementations::table.filter(implementations::id.eq(legacy_impl.id)))
            .set(implementations::algorithm_id.eq(modified_cosine_approx_id))
            .execute(conn)
            .expect("failed to migrate modified cosine matchms implementation");
    }
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

fn ensure_ms_entropy_library(conn: &mut SqliteConnection) -> i32 {
    let version = ms_entropy_version();

    if let Some(lib) = libraries::table
        .filter(libraries::name.eq(MS_ENTROPY_LIB_NAME))
        .filter(libraries::version.eq(&version))
        .first::<Library>(conn)
        .optional()
        .expect("query failed")
    {
        return lib.id;
    }

    diesel::insert_into(libraries::table)
        .values(&NewLibrary {
            name: MS_ENTROPY_LIB_NAME,
            version: &version,
            git_commit: None,
            git_url: Some("https://github.com/YuanyueLi/MSEntropy"),
            language: "python",
        })
        .returning(libraries::id)
        .get_result::<i32>(conn)
        .expect("failed to insert ms_entropy library")
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

fn ms_entropy_version() -> String {
    std::process::Command::new("uv")
        .args([
            "run",
            "python3",
            "-c",
            "import ms_entropy; print(ms_entropy.__version__)",
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

        assert_eq!(algorithm_count, 7);
        assert_eq!(implementation_count, 12);
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
        let matchms_modified = get_implementation_id(&mut conn, "ModifiedCosineApprox", "matchms");
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
        assert_ne!(rust_modified, matchms_modified);
        assert_ne!(rust_modified_greedy, matchms_modified_greedy);
        assert_ne!(rust_modified, rust_modified_greedy);
        assert_ne!(matchms_modified, matchms_modified_greedy);
        assert_ne!(rust_hungarian, rust_modified);
        assert_ne!(matchms_hungarian, matchms_modified);
        assert_ne!(matchms_greedy, matchms_modified);
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

        assert_eq!(refs_by_algorithm.len(), 7);
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

        let modified_cosine_approx_id = algorithms::table
            .filter(algorithms::name.eq("ModifiedCosineApprox"))
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .expect("failed to load ModifiedCosineApprox id");
        assert_eq!(
            refs_by_algorithm
                .get(&modified_cosine_approx_id)
                .copied()
                .unwrap_or(0),
            0
        );

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
    fn seeds_modified_cosine_approx_as_approximation_of_modified_cosine() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let modified_cosine_id = algorithms::table
            .filter(algorithms::name.eq("ModifiedCosine"))
            .select(algorithms::id)
            .first::<i32>(&mut conn)
            .expect("failed to load ModifiedCosine id");

        let modified_cosine_approx = algorithms::table
            .filter(algorithms::name.eq("ModifiedCosineApprox"))
            .select(algorithms::approximates_algorithm_id)
            .first::<Option<i32>>(&mut conn)
            .expect("failed to load ModifiedCosineApprox approximation target");

        assert_eq!(modified_cosine_approx, Some(modified_cosine_id));

        let rust_modified_ref = implementations::table
            .inner_join(algorithms::table)
            .inner_join(libraries::table)
            .filter(algorithms::name.eq("ModifiedCosine"))
            .filter(libraries::name.eq("mass-spectrometry-traits"))
            .select(implementations::is_reference)
            .first::<bool>(&mut conn)
            .expect("failed to load ModifiedCosine rust reference flag");
        assert!(rust_modified_ref);

        let matchms_modified_ref = implementations::table
            .inner_join(algorithms::table)
            .inner_join(libraries::table)
            .filter(algorithms::name.eq("ModifiedCosineApprox"))
            .filter(libraries::name.eq("matchms"))
            .select(implementations::is_reference)
            .first::<bool>(&mut conn)
            .expect("failed to load ModifiedCosineApprox matchms reference flag");
        assert!(!matchms_modified_ref);
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

    #[test]
    fn upgrades_legacy_implementations_table_with_reference_column() {
        let mut conn = setup_in_memory_connection();
        sql_query(
            "CREATE TABLE implementations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                algorithm_id INTEGER NOT NULL,
                library_id INTEGER NOT NULL,
                UNIQUE(algorithm_id, library_id)
            ) STRICT",
        )
        .execute(&mut conn)
        .expect("failed to create legacy implementations table");

        assert!(!implementations_has_column(&mut conn, "is_reference"));
        ensure_implementations_reference_schema(&mut conn);
        assert!(implementations_has_column(&mut conn, "is_reference"));
    }

    #[test]
    fn upgrades_legacy_algorithms_table_with_approximation_column() {
        let mut conn = setup_in_memory_connection();
        sql_query(
            "CREATE TABLE algorithms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT
            ) STRICT",
        )
        .execute(&mut conn)
        .expect("failed to create legacy algorithms table");

        assert!(!algorithms_has_column(
            &mut conn,
            "approximates_algorithm_id"
        ));
        ensure_algorithms_approximation_schema(&mut conn);
        assert!(algorithms_has_column(
            &mut conn,
            "approximates_algorithm_id"
        ));
    }

    #[test]
    fn upgrades_legacy_spectra_table_with_spectrum_hash_column() {
        let mut conn = setup_in_memory_connection();
        sql_query(
            "CREATE TABLE spectra (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                raw_name TEXT NOT NULL,
                source_file TEXT NOT NULL,
                precursor_mz REAL NOT NULL,
                num_peaks INTEGER NOT NULL,
                peaks TEXT NOT NULL
            ) STRICT",
        )
        .execute(&mut conn)
        .expect("failed to create legacy spectra table");

        sql_query(
            "INSERT INTO spectra (name, raw_name, source_file, precursor_mz, num_peaks, peaks)
             VALUES
             ('a', 'a', 'legacy.mgf', 100.0, 5, '[]'),
             ('b', 'b', 'legacy.mgf', 200.0, 6, '[]')",
        )
        .execute(&mut conn)
        .expect("failed to seed legacy spectra rows");

        assert!(!table_has_column(&mut conn, "spectra", "spectrum_hash"));
        ensure_spectra_hash_schema(&mut conn);
        assert!(table_has_column(&mut conn, "spectra", "spectrum_hash"));

        let hashes: Vec<String> = sql_query(
            "SELECT spectrum_hash AS name
             FROM spectra
             ORDER BY id",
        )
        .load::<TableInfoRow>(&mut conn)
        .expect("failed to load backfilled spectrum hashes")
        .into_iter()
        .map(|row| row.name)
        .collect();
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0], "__legacy_spectrum_1");
        assert_eq!(hashes[1], "__legacy_spectrum_2");
    }
}
