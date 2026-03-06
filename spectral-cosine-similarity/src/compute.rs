use std::collections::HashMap;
use std::hint::black_box;
use std::process::{Command, Stdio};
use std::time::Instant;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use mass_spectrometry::prelude::*;

use crate::db;
use crate::models::*;
use crate::pair_selection;

const FLUSH_BATCH: usize = 500;
const SUBSTEP_UPDATE_INTERVAL: usize = 5_000;
const GLOBAL_WARMUP_PAIR_SAMPLE: usize = 100;
const RUST_LIBRARY_NAME: &str = "mass-spectrometry-traits";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Preprocessing {
    None,
    SiriusMerge,
    MsEntropyClean,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlgoKind {
    HungarianCosine,
    GreedyCosine,
    LinearCosine,
    ModifiedCosine,
    ModifiedGreedyCosine,
    ModifiedLinearCosine,
    EntropyWeighted,
    EntropyUnweighted,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct PythonAlgoSpec {
    algorithm_name: &'static str,
    library_name: &'static str,
}

const RUST_ALGO_SPECS: [(&str, Preprocessing, AlgoKind); 8] = [
    ("CosineHungarian", Preprocessing::None, AlgoKind::HungarianCosine),
    ("CosineGreedy", Preprocessing::None, AlgoKind::GreedyCosine),
    ("LinearCosine", Preprocessing::SiriusMerge, AlgoKind::LinearCosine),
    ("ModifiedCosine", Preprocessing::None, AlgoKind::ModifiedCosine),
    ("ModifiedGreedyCosine", Preprocessing::None, AlgoKind::ModifiedGreedyCosine),
    ("ModifiedLinearCosine", Preprocessing::SiriusMerge, AlgoKind::ModifiedLinearCosine),
    ("EntropySimilarityWeighted", Preprocessing::MsEntropyClean, AlgoKind::EntropyWeighted),
    ("EntropySimilarityUnweighted", Preprocessing::MsEntropyClean, AlgoKind::EntropyUnweighted),
];

#[cfg(test)]
const PYTHON_ALGO_SPECS: [PythonAlgoSpec; 5] = [
    PythonAlgoSpec {
        algorithm_name: "CosineHungarian",
        library_name: "matchms",
    },
    PythonAlgoSpec {
        algorithm_name: "CosineGreedy",
        library_name: "matchms",
    },
    PythonAlgoSpec {
        algorithm_name: "ModifiedGreedyCosine",
        library_name: "matchms",
    },
    PythonAlgoSpec {
        algorithm_name: "EntropySimilarityWeighted",
        library_name: "ms_entropy",
    },
    PythonAlgoSpec {
        algorithm_name: "EntropySimilarityUnweighted",
        library_name: "ms_entropy",
    },
];

fn algorithm_cli_label(algorithm_name: &str, library_name: &str) -> String {
    format!("{algorithm_name} ({library_name})")
}

type SpectraMap = HashMap<i32, GenericSpectrum<f64, f64>>;

pub fn preflight_python_environment() {
    let uv_check = Command::new("uv")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match uv_check {
        Ok(status) if status.success() => {}
        Ok(status) => {
            panic!(
                "[preflight] `uv --version` exited with {status}. \
                 Install `uv` and ensure it is available on PATH."
            );
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            panic!(
                "[preflight] `uv` is required but was not found on PATH. \
                 Install `uv` (https://docs.astral.sh/uv/) and rerun."
            );
        }
        Err(err) => {
            panic!("[preflight] failed to run `uv --version`: {err}");
        }
    }

    let import_check = Command::new("uv")
        .args(["run", "python3", "-c", "import matchms, ms_entropy"])
        .status();
    match import_check {
        Ok(status) if status.success() => {}
        Ok(status) => {
            panic!(
                "[preflight] Python dependency check failed with {status}. \
                 Run `uv sync` in spectral-cosine-similarity/ and ensure both \
                 `matchms` and `ms_entropy` import successfully."
            );
        }
        Err(err) => {
            panic!("[preflight] failed to run python dependency check via `uv run`: {err}");
        }
    }
}

/// Compute similarities and timings for all implementations (production entry point).
pub fn run(
    conn: &mut SqliteConnection,
    max_spectra: usize,
    num_pairs: usize,
) {
    run_with_python_runner(conn, max_spectra, num_pairs, run_python_default);
}

/// Compute similarities with an injectable Python runner (for tests).
pub fn run_with_python_runner<F>(
    conn: &mut SqliteConnection,
    max_spectra: usize,
    num_pairs: usize,
    run_python: F,
) where
    F: Fn(),
{
    let (experiments, spectra_map, id_pairs) = load_compute_context(conn, max_spectra, num_pairs);
    run_rust_compute_passes(conn, &experiments, &spectra_map, &id_pairs);

    let python_impl_count = python_implementation_ids(conn).len() as u64;
    if python_impl_count > 0 {
        eprintln!("[compute] Python: running all algorithms");
        run_python();
        eprintln!("[compute] Python: complete");
    }
}

fn run_python_default() {
    let db = db::db_path();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("python_reference_compute.py");
    let mut cmd = Command::new("uv");
    cmd.args(["run", "python3"]);
    cmd.arg(&script);
    cmd.arg(db);
    let status = cmd
        .status()
        .unwrap_or_else(|err| panic!("[compute] failed to launch `uv run python3`: {err}"));

    if !status.success() {
        panic!(
            "[compute] python_reference_compute.py exited with {status}. \
             Install Python benchmark dependencies with `uv sync` in spectral-cosine-similarity/ \
             and ensure both `matchms` and `ms_entropy` import successfully."
        );
    }
}

fn flush_results(conn: &mut SqliteConnection, batch: &mut Vec<NewResult>) {
    if batch.is_empty() {
        return;
    }
    for chunk in batch.chunks(FLUSH_BATCH) {
        diesel::insert_into(crate::schema::results::table)
            .values(chunk)
            .execute(conn)
            .expect("failed to insert results");
    }
    batch.clear();
}

fn load_compute_context(
    conn: &mut SqliteConnection,
    max_spectra: usize,
    num_pairs: usize,
) -> (Vec<Experiment>, SpectraMap, Vec<(i32, i32)>) {
    let experiments = db::load_experiments(conn);

    let limit = i64::try_from(max_spectra).unwrap_or(i64::MAX);
    let all_spectra: Vec<SpectrumRow> = crate::schema::spectra::table
        .order(crate::schema::spectra::id.asc())
        .limit(limit)
        .load(conn)
        .expect("failed to load spectra");

    let spectrum_ids: Vec<i32> = all_spectra.iter().map(|s| s.id).collect();
    let spectra_map: SpectraMap = all_spectra
        .iter()
        .map(|s| (s.id, s.to_generic_spectrum()))
        .collect();

    eprintln!("[compute] Sampling {num_pairs} pairs");
    let id_pairs = pair_selection::sample_pairs(&spectrum_ids, num_pairs);

    eprintln!("[compute] {} pairs ready", id_pairs.len());

    // Write selected pairs to DB so Python reads the same set.
    diesel::delete(crate::schema::selected_pairs::table)
        .execute(conn)
        .expect("failed to clear selected_pairs");
    for chunk in id_pairs.chunks(FLUSH_BATCH) {
        let new_pairs: Vec<NewSelectedPair> = chunk
            .iter()
            .map(|&(l, r)| NewSelectedPair { left_id: l, right_id: r })
            .collect();
        diesel::insert_into(crate::schema::selected_pairs::table)
            .values(&new_pairs)
            .execute(conn)
            .expect("failed to insert selected_pairs");
    }

    (experiments, spectra_map, id_pairs)
}

fn compute_rust_algorithm<B, S>(
    conn: &mut SqliteConnection,
    experiments: &[Experiment],
    spectra_map: &SpectraMap,
    id_pairs: &[(i32, i32)],
    algorithm_name: &str,
    build_similarity: B,
) -> usize
where
    B: Fn(&ExperimentParams) -> S,
    S: ScalarSimilarity<
        GenericSpectrum<f64, f64>,
        GenericSpectrum<f64, f64>,
        Similarity = std::result::Result<(f64, usize), SimilarityComputationError>,
    >,
{
    let impl_id = db::get_implementation_id(conn, algorithm_name, RUST_LIBRARY_NAME);
    let algorithm_label = algorithm_cli_label(algorithm_name, RUST_LIBRARY_NAME);
    let work_len = id_pairs.len() * experiments.len();

    if work_len == 0 {
        eprintln!("[compute] {algorithm_label}: nothing to compute");
        return 0;
    }

    eprintln!("[compute] {algorithm_label}: 0/{work_len}");

    let mut batch: Vec<NewResult> = Vec::with_capacity(work_len);
    let mut total_done: usize = 0;

    for exp in experiments {
        let params = exp.parse_params();
        let similarity = build_similarity(&params);

        let warmup_pairs: Vec<(i32, i32)> = id_pairs
            .iter()
            .copied()
            .take(GLOBAL_WARMUP_PAIR_SAMPLE)
            .collect();
        for _ in 0..params.n_warmup {
            for (left_id, right_id) in &warmup_pairs {
                let left = spectra_map.get(left_id).expect("left spectrum not found");
                let right = spectra_map.get(right_id).expect("right spectrum not found");
                let _ = black_box(similarity.similarity(black_box(left), black_box(right)))
                    .unwrap_or_else(|err| {
                        panic!(
                            "[compute] {algorithm_label} warmup failed for \
                             ({left_id}, {right_id}), experiment={}: {err:?}",
                            exp.id
                        )
                    });
            }
        }

        for &(left_id, right_id) in id_pairs {
            let left = spectra_map.get(&left_id).expect("left spectrum not found");
            let right = spectra_map.get(&right_id).expect("right spectrum not found");

            let mut times_ns: Vec<u128> = Vec::with_capacity(params.n_reps as usize);
            let mut last_result = (0.0f64, 0usize);
            for _ in 0..params.n_reps {
                let t0 = Instant::now();
                last_result =
                    black_box(similarity.similarity(black_box(left), black_box(right)))
                        .unwrap_or_else(|err| {
                            panic!(
                                "[compute] {algorithm_label} failed for \
                                 ({left_id}, {right_id}), experiment={}: {err:?}",
                                exp.id
                            )
                        });
                times_ns.push(t0.elapsed().as_nanos());
            }

            let (score, matches) = last_result;
            let matches = i32::try_from(matches).unwrap_or_else(|_| {
                panic!(
                    "[compute] {algorithm_label} matches={matches} does not fit i32 \
                     for ({left_id}, {right_id})"
                )
            });
            times_ns.sort_unstable();
            let median_ns = times_ns[params.n_reps as usize / 2];
            let median_us = median_ns as f64 / 1000.0;

            batch.push(NewResult {
                left_id,
                right_id,
                experiment_id: exp.id,
                implementation_id: impl_id,
                score,
                matches,
                median_time_us: median_us,
            });

            total_done += 1;

            if total_done == work_len || total_done.is_multiple_of(SUBSTEP_UPDATE_INTERVAL) {
                eprintln!("[compute] {algorithm_label}: {total_done}/{work_len}");
            }
        }
    }

    flush_results(conn, &mut batch);
    eprintln!("[compute] {algorithm_label}: {total_done} pairs computed");
    total_done
}

fn run_algo(
    conn: &mut SqliteConnection,
    experiments: &[Experiment],
    spectra_map: &SpectraMap,
    id_pairs: &[(i32, i32)],
    algorithm_name: &str,
    kind: AlgoKind,
) -> usize {
    match kind {
        AlgoKind::HungarianCosine => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            HungarianCosine::new(p.mz_power, p.intensity_power, p.tolerance).expect("failed to build HungarianCosine")
        }),
        AlgoKind::GreedyCosine => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            GreedyCosine::new(p.mz_power, p.intensity_power, p.tolerance).expect("failed to build GreedyCosine")
        }),
        AlgoKind::LinearCosine => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            LinearCosine::new(p.mz_power, p.intensity_power, p.tolerance).expect("failed to build LinearCosine")
        }),
        AlgoKind::ModifiedCosine => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            ModifiedHungarianCosine::new(p.mz_power, p.intensity_power, p.tolerance).expect("failed to build ModifiedHungarianCosine")
        }),
        AlgoKind::ModifiedGreedyCosine => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            ModifiedGreedyCosine::new(p.mz_power, p.intensity_power, p.tolerance).expect("failed to build ModifiedGreedyCosine")
        }),
        AlgoKind::ModifiedLinearCosine => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            ModifiedLinearCosine::new(p.mz_power, p.intensity_power, p.tolerance).expect("failed to build ModifiedLinearCosine")
        }),
        AlgoKind::EntropyWeighted => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            LinearEntropy::new(p.mz_power, p.intensity_power, p.tolerance, true).expect("failed to build LinearEntropy weighted")
        }),
        AlgoKind::EntropyUnweighted => compute_rust_algorithm(conn, experiments, spectra_map, id_pairs, algorithm_name, |p| {
            LinearEntropy::new(p.mz_power, p.intensity_power, p.tolerance, false).expect("failed to build LinearEntropy unweighted")
        }),
    }
}

fn build_sirius_merged_map(experiments: &[Experiment], base_map: &SpectraMap) -> SpectraMap {
    let min_tolerance = experiments
        .iter()
        .map(|exp| exp.parse_params().tolerance)
        .fold(f64::INFINITY, f64::min);

    let merger = SiriusMergeClosePeaks::new(min_tolerance)
        .expect("failed to build SiriusMergeClosePeaks from experiment tolerance");

    base_map
        .iter()
        .map(|(&id, spectrum)| (id, merger.process(spectrum)))
        .collect()
}

fn build_entropy_cleaned_map(base_map: &SpectraMap) -> SpectraMap {
    let cleaner = MsEntropyCleanSpectrum::<f64>::builder()
        .build()
        .expect("failed to build MsEntropyCleanSpectrum with default parameters");

    base_map
        .iter()
        .map(|(&id, spectrum)| (id, cleaner.process(spectrum)))
        .collect()
}

fn run_rust_compute_passes(
    conn: &mut SqliteConnection,
    experiments: &[Experiment],
    spectra_map: &SpectraMap,
    id_pairs: &[(i32, i32)],
) {
    eprintln!("[compute] Starting Rust algorithms");

    let any_sirius = RUST_ALGO_SPECS
        .iter()
        .any(|&(_, prep, _)| prep == Preprocessing::SiriusMerge);
    let any_entropy = RUST_ALGO_SPECS
        .iter()
        .any(|&(_, prep, _)| prep == Preprocessing::MsEntropyClean);
    let sirius_map = if any_sirius {
        Some(build_sirius_merged_map(experiments, spectra_map))
    } else {
        None
    };
    let entropy_map = if any_entropy {
        Some(build_entropy_cleaned_map(spectra_map))
    } else {
        None
    };

    for &(algorithm_name, preprocessing, kind) in &RUST_ALGO_SPECS {
        let effective_map = match preprocessing {
            Preprocessing::None => spectra_map,
            Preprocessing::SiriusMerge => sirius_map.as_ref().expect("sirius map must exist"),
            Preprocessing::MsEntropyClean => entropy_map.as_ref().expect("entropy map must exist"),
        };
        run_algo(conn, experiments, effective_map, id_pairs, algorithm_name, kind);
    }
}

fn python_implementation_ids(conn: &mut SqliteConnection) -> Vec<i32> {
    crate::schema::implementations::table
        .inner_join(crate::schema::libraries::table)
        .filter(crate::schema::libraries::language.eq("python"))
        .order(crate::schema::implementations::id.asc())
        .select(crate::schema::implementations::id)
        .load(conn)
        .expect("failed to load python implementations")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use diesel::Connection;
    use diesel::sqlite::SqliteConnection;

    #[test]
    fn python_implementations_include_non_reference_modified_algorithms() {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("failed to open in-memory sqlite db");
        db::initialize(&mut conn);

        let python_impls = python_implementation_ids(&mut conn);
        let matchms_modified_greedy =
            db::get_implementation_id(&mut conn, "ModifiedGreedyCosine", "matchms");
        assert!(python_impls.contains(&matchms_modified_greedy));
        assert_eq!(python_impls.len(), 5);
    }

    #[test]
    fn python_algorithm_specs_match_registered_python_implementations() {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("failed to open in-memory sqlite db");
        db::initialize(&mut conn);

        let python_impls = python_implementation_ids(&mut conn);
        for spec in PYTHON_ALGO_SPECS {
            let implementation_id =
                db::get_implementation_id(&mut conn, spec.algorithm_name, spec.library_name);
            assert!(
                python_impls.contains(&implementation_id),
                "missing implementation for {}",
                algorithm_cli_label(spec.algorithm_name, spec.library_name)
            );
        }

        assert_eq!(PYTHON_ALGO_SPECS.len(), python_impls.len());
    }
}
