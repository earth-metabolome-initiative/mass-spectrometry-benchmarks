use std::collections::HashMap;
use std::path::Path;

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
const MS_ENTROPY_LIB_NAME: &str = "ms_entropy";

const N_WARMUP: u32 = 3;
const N_REPS: u32 = 5;

const TOLERANCE: f64 = 0.01;
const MZ_POWER: f64 = 0.0;
const INTENSITY_POWER: f64 = 1.0;

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

    // Seed algorithms: (name, description, approximates)
    const ALGO_SEEDS: &[(&str, &str, Option<&str>)] = &[
        (
            "CosineHungarian",
            "Hungarian algorithm-based cosine similarity",
            None,
        ),
        (
            "CosineGreedy",
            "Greedy cosine similarity",
            Some("CosineHungarian"),
        ),
        (
            "LinearCosine",
            "Linear-time cosine similarity for well-separated spectra",
            Some("CosineHungarianMerged"),
        ),
        (
            "ModifiedCosine",
            "Exact precursor-shift-aware modified cosine similarity",
            None,
        ),
        (
            "ModifiedGreedyCosine",
            "Greedy precursor-shift-aware modified cosine similarity",
            Some("ModifiedCosine"),
        ),
        (
            "ModifiedCosineHungarian",
            "Hungarian precursor-shift-aware modified cosine similarity",
            Some("ModifiedCosine"),
        ),
        (
            "ModifiedLinearCosine",
            "Linear-time modified cosine similarity for well-separated spectra",
            Some("ModifiedCosineMerged"),
        ),
        (
            "CosineHungarianMerged",
            "Hungarian cosine on pre-merged spectra (merged-peaks baseline)",
            None,
        ),
        (
            "ModifiedCosineMerged",
            "Modified Hungarian cosine on pre-merged spectra (merged-peaks baseline)",
            None,
        ),
        (
            "EntropySimilarityWeighted",
            "Weighted spectral entropy similarity",
            None,
        ),
        (
            "EntropySimilarityUnweighted",
            "Unweighted spectral entropy similarity",
            None,
        ),
        (
            "ModifiedLinearEntropyWeighted",
            "Weighted modified linear entropy similarity",
            None,
        ),
        (
            "ModifiedLinearEntropyUnweighted",
            "Unweighted modified linear entropy similarity",
            None,
        ),
    ];

    let mut algo_ids: HashMap<&str, i32> = HashMap::new();
    for &(name, description, _) in ALGO_SEEDS {
        algo_ids.insert(name, ensure_algorithm(conn, name, Some(description)));
    }
    for &(name, _, approximates) in ALGO_SEEDS {
        let approx_id =
            approximates.map(|a| *algo_ids.get(a).expect("unknown approximation target"));
        set_algorithm_approximation(conn, algo_ids[name], approx_id);
    }

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

    // Seed implementations: (algorithm, library_id, is_reference)
    let impl_seeds: &[(&str, i32, bool)] = &[
        ("CosineHungarian", rust_lib_id, false),
        ("CosineHungarian", matchms_lib_id, true),
        ("CosineGreedy", matchms_lib_id, true),
        ("ModifiedCosine", rust_lib_id, true),
        ("ModifiedGreedyCosine", matchms_lib_id, false),
        ("ModifiedCosineHungarian", matchms_lib_id, false),
        ("EntropySimilarityWeighted", rust_lib_id, true),
        ("EntropySimilarityWeighted", ms_entropy_lib_id, false),
        ("EntropySimilarityUnweighted", rust_lib_id, true),
        ("EntropySimilarityUnweighted", ms_entropy_lib_id, false),
        ("LinearCosine", rust_lib_id, false),
        ("ModifiedLinearCosine", rust_lib_id, false),
        ("CosineHungarianMerged", rust_lib_id, true),
        ("ModifiedCosineMerged", rust_lib_id, true),
        ("ModifiedLinearEntropyWeighted", rust_lib_id, true),
        ("ModifiedLinearEntropyUnweighted", rust_lib_id, true),
    ];
    for &(algo_name, lib_id, is_ref) in impl_seeds {
        ensure_implementation(conn, algo_ids[algo_name], lib_id, is_ref);
    }

    // Seed experiment
    ensure_experiment(
        conn,
        &ExperimentParams {
            tolerance: TOLERANCE,
            mz_power: MZ_POWER,
            intensity_power: INTENSITY_POWER,
            n_warmup: N_WARMUP,
            n_reps: N_REPS,
        },
    );
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
        && let Some(version) = extract_mass_spec_field(&content, "version = ")
    {
        return version;
    }
    "unknown".to_string()
}

fn rust_lib_git_commit() -> Option<String> {
    let lock_path = "Cargo.lock";
    let content = std::fs::read_to_string(lock_path).ok()?;
    let source = extract_mass_spec_field(&content, "source = ")?;
    source.split('#').nth(1).map(|s| s.to_string())
}

fn extract_mass_spec_field(lock_content: &str, field_prefix: &str) -> Option<String> {
    let mut in_mass_spec = false;
    for line in lock_content.lines() {
        if line.starts_with("name = \"mass_spectrometry\"") {
            in_mass_spec = true;
        } else if in_mass_spec && line.starts_with(field_prefix) {
            return Some(
                line.trim_start_matches(field_prefix)
                    .trim_matches('"')
                    .to_string(),
            );
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

        assert_eq!(
            extract_mass_spec_field(lock, "version = "),
            Some("0.9.1".to_string())
        );
        let source = extract_mass_spec_field(lock, "source = ").unwrap();
        assert_eq!(source.split('#').nth(1), Some("abc123def"));
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

        assert_eq!(algorithm_count, 13);
        assert_eq!(implementation_count, 16);
        assert_eq!(experiment_count, 1);
    }

    #[test]
    fn resolves_seeded_implementation_ids_are_unique() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let all_ids: Vec<i32> = implementations::table
            .select(implementations::id)
            .load(&mut conn)
            .expect("failed to load implementation ids");

        assert_eq!(all_ids.len(), 16);
        let unique: std::collections::HashSet<i32> = all_ids.iter().copied().collect();
        assert_eq!(
            unique.len(),
            all_ids.len(),
            "implementation IDs must be unique"
        );
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

        assert_eq!(refs_by_algorithm.len(), 13);
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

        for approx_name in ["ModifiedGreedyCosine", "ModifiedCosineHungarian"] {
            let approx_id = algorithms::table
                .filter(algorithms::name.eq(approx_name))
                .select(algorithms::id)
                .first::<i32>(&mut conn)
                .unwrap_or_else(|_| panic!("failed to load {approx_name} id"));
            assert_eq!(
                refs_by_algorithm.get(&approx_id).copied().unwrap_or(0),
                0,
                "{approx_name} should have no reference implementation"
            );
        }
    }

    #[test]
    fn seeds_approximation_relationships() {
        let mut conn = setup_in_memory_connection();
        initialize(&mut conn);

        let expected: &[(&str, Option<&str>)] = &[
            ("CosineHungarian", None),
            ("CosineGreedy", Some("CosineHungarian")),
            ("LinearCosine", Some("CosineHungarianMerged")),
            ("CosineHungarianMerged", None),
            ("ModifiedCosine", None),
            ("ModifiedGreedyCosine", Some("ModifiedCosine")),
            ("ModifiedCosineHungarian", Some("ModifiedCosine")),
            ("ModifiedLinearCosine", Some("ModifiedCosineMerged")),
            ("ModifiedCosineMerged", None),
            ("EntropySimilarityWeighted", None),
            ("EntropySimilarityUnweighted", None),
            ("ModifiedLinearEntropyWeighted", None),
            ("ModifiedLinearEntropyUnweighted", None),
        ];

        for &(algo_name, expected_approx) in expected {
            let (approx_id, approx_name): (Option<i32>, Option<String>) = {
                let row = algorithms::table
                    .filter(algorithms::name.eq(algo_name))
                    .select((algorithms::approximates_algorithm_id, algorithms::name))
                    .first::<(Option<i32>, String)>(&mut conn)
                    .unwrap_or_else(|_| panic!("algorithm {algo_name} not found"));

                let name = row.0.map(|id| {
                    algorithms::table
                        .filter(algorithms::id.eq(id))
                        .select(algorithms::name)
                        .first::<String>(&mut conn)
                        .expect("approximation target not found")
                });
                (row.0, name)
            };

            match expected_approx {
                None => assert!(
                    approx_id.is_none(),
                    "{algo_name} should not approximate anything, got {approx_name:?}"
                ),
                Some(target) => assert_eq!(
                    approx_name.as_deref(),
                    Some(target),
                    "{algo_name} should approximate {target}"
                ),
            }
        }
    }
}
