use std::collections::HashMap;
use std::hint::black_box;
use std::process::{Command, Stdio};
use std::result::Result as StdResult;
use std::time::Instant;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use mass_spectrometry::prelude::*;

use crate::db;
use crate::models::*;
use crate::pair_selection;
use crate::progress::StageProgress;

const FLUSH_BATCH: usize = 500;
const SUBSTEP_UPDATE_INTERVAL: usize = 5_000;
const GLOBAL_WARMUP_PAIR_SAMPLE: usize = 100;
const RUST_LIBRARY_NAME: &str = "mass-spectrometry-traits";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RustAlgoKind {
    CosineHungarian,
    CosineGreedy,
    LinearCosine,
    ModifiedCosine,
    ModifiedGreedyCosine,
    ModifiedLinearCosine,
    EntropySimilarityWeighted,
    EntropySimilarityUnweighted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Preprocessing {
    None,
    SiriusMerge,
    MsEntropyClean,
}

#[derive(Clone, Copy, Debug)]
struct RustAlgoSpec {
    kind: RustAlgoKind,
    algorithm_name: &'static str,
    preprocessing: Preprocessing,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct PythonAlgoSpec {
    algorithm_name: &'static str,
    library_name: &'static str,
}

const RUST_ALGO_SPECS: [RustAlgoSpec; 8] = [
    RustAlgoSpec {
        kind: RustAlgoKind::CosineHungarian,
        algorithm_name: "CosineHungarian",
        preprocessing: Preprocessing::None,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::CosineGreedy,
        algorithm_name: "CosineGreedy",
        preprocessing: Preprocessing::None,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::LinearCosine,
        algorithm_name: "LinearCosine",
        preprocessing: Preprocessing::SiriusMerge,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::ModifiedCosine,
        algorithm_name: "ModifiedCosine",
        preprocessing: Preprocessing::None,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::ModifiedGreedyCosine,
        algorithm_name: "ModifiedGreedyCosine",
        preprocessing: Preprocessing::None,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::ModifiedLinearCosine,
        algorithm_name: "ModifiedLinearCosine",
        preprocessing: Preprocessing::SiriusMerge,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::EntropySimilarityWeighted,
        algorithm_name: "EntropySimilarityWeighted",
        preprocessing: Preprocessing::MsEntropyClean,
    },
    RustAlgoSpec {
        kind: RustAlgoKind::EntropySimilarityUnweighted,
        algorithm_name: "EntropySimilarityUnweighted",
        preprocessing: Preprocessing::MsEntropyClean,
    },
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

fn build_similarity_or_panic<T>(
    algorithm_name: &str,
    params: &ExperimentParams,
    build: impl FnOnce() -> StdResult<T, SimilarityConfigError>,
) -> T {
    build().unwrap_or_else(|err| {
        panic!("[compute] failed to build {algorithm_name} with params {params:?}: {err:?}")
    })
}

struct ComputeContext {
    experiments: Vec<Experiment>,
    spectrum_ids: Vec<i32>,
    spectra_map: HashMap<i32, GenericSpectrum<f64, f64>>,
    id_pairs: Vec<(i32, i32)>,
}

fn emit(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(p) = progress.as_deref_mut() {
        p.set_message(message);
    } else {
        eprintln!("{message}");
    }
}

fn inc_progress(progress: &mut Option<&mut dyn StageProgress>, units: u64) {
    if let Some(p) = progress.as_deref_mut() {
        p.inc(units);
    }
}

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

/// Estimate total compute workload units (without checking existing results).
pub fn estimate_work(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    num_pairs: Option<usize>,
) -> u64 {
    let n_spectra: i64 = {
        let total = crate::schema::spectra::table
            .select(diesel::dsl::count_star())
            .first::<i64>(conn)
            .expect("failed to count spectra");
        if let Some(max) = max_spectra {
            total.min(max as i64)
        } else {
            total
        }
    };
    let total_pairs = n_spectra * (n_spectra + 1) / 2;
    let n_pairs = if let Some(requested) = num_pairs {
        total_pairs.min(requested as i64)
    } else {
        total_pairs
    };
    let n_experiments = crate::schema::experiments::table
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .expect("failed to count experiments");
    let n_implementations = crate::schema::implementations::table
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .expect("failed to count implementations");
    (n_pairs * n_experiments * n_implementations) as u64
}

/// Compute similarities and timings for all implementations (production entry point).
pub fn run(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    num_pairs: Option<usize>,
    progress: Option<&mut dyn StageProgress>,
) {
    run_with_python_runner(conn, max_spectra, num_pairs, run_python_default, progress);
}

/// Compute similarities with an injectable Python runner (for tests).
pub fn run_with_python_runner<F>(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    num_pairs: Option<usize>,
    run_python: F,
    mut progress: Option<&mut dyn StageProgress>,
) where
    F: Fn(Option<usize>, Option<usize>),
{
    let context = load_compute_context(conn, max_spectra, num_pairs, &mut progress);
    run_rust_compute_passes(conn, &context, &mut progress);

    let python_impl_count = python_implementation_ids(conn).len() as u64;
    if python_impl_count > 0 {
        emit(&mut progress, "[compute] Python: running all algorithms");
        run_python(max_spectra, num_pairs);
        let expected_python_units = (context.id_pairs.len() as u64)
            * (context.experiments.len() as u64)
            * python_impl_count;
        inc_progress(&mut progress, expected_python_units);
        emit(&mut progress, "[compute] Python: complete");
    }
}

fn run_python_default(max_spectra: Option<usize>, num_pairs: Option<usize>) {
    let db = db::db_path();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("python_reference_compute.py");
    let mut cmd = Command::new("uv");
    cmd.args(["run", "python3"]);
    cmd.arg(&script);
    cmd.arg(db);
    if let Some(max_spectra) = max_spectra {
        cmd.arg("--max-spectra");
        cmd.arg(max_spectra.to_string());
    }
    if let Some(num_pairs) = num_pairs {
        cmd.arg("--num-pairs");
        cmd.arg(num_pairs.to_string());
    }
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
    max_spectra: Option<usize>,
    num_pairs: Option<usize>,
    progress: &mut Option<&mut dyn StageProgress>,
) -> ComputeContext {
    let experiments = db::load_experiments(conn);

    let mut spectra_query = crate::schema::spectra::table
        .order(crate::schema::spectra::id.asc())
        .into_boxed();
    if let Some(max_spectra) = max_spectra {
        let limit = i64::try_from(max_spectra).unwrap_or(i64::MAX);
        spectra_query = spectra_query.limit(limit);
    }
    let all_spectra: Vec<SpectrumRow> = spectra_query.load(conn).expect("failed to load spectra");

    let spectrum_ids: Vec<i32> = all_spectra.iter().map(|s| s.id).collect();
    let spectra_map: HashMap<i32, GenericSpectrum<f64, f64>> = all_spectra
        .iter()
        .map(|s| (s.id, s.to_generic_spectrum()))
        .collect();

    let id_pairs = if let Some(n) = num_pairs {
        emit(progress, &format!("[compute] Sampling {n} pairs"));
        pair_selection::sample_pairs(&spectrum_ids, n)
    } else {
        let total = spectrum_ids.len() * (spectrum_ids.len() + 1) / 2;
        emit(progress, &format!("[compute] Generating {total} pairs"));
        pair_selection::generate_pairs(&spectrum_ids)
    };

    emit(
        progress,
        &format!("[compute] {} pairs ready", id_pairs.len()),
    );

    ComputeContext {
        experiments,
        spectrum_ids,
        spectra_map,
        id_pairs,
    }
}

fn compute_rust_algorithm<B, S>(
    conn: &mut SqliteConnection,
    context: &ComputeContext,
    algorithm_name: &str,
    progress: &mut Option<&mut dyn StageProgress>,
    build_similarity: B,
) -> usize
where
    B: Fn(&ExperimentParams) -> S,
    S: ScalarSimilarity<
        GenericSpectrum<f64, f64>,
        GenericSpectrum<f64, f64>,
        Similarity = StdResult<(f64, usize), SimilarityComputationError>,
    >,
{
    let impl_id = db::get_implementation_id(conn, algorithm_name, RUST_LIBRARY_NAME);
    let algorithm_label = algorithm_cli_label(algorithm_name, RUST_LIBRARY_NAME);
    let work_len = context.id_pairs.len() * context.experiments.len();

    if work_len == 0 {
        emit(
            progress,
            &format!("[compute] {algorithm_label}: nothing to compute"),
        );
        return 0;
    }

    emit(
        progress,
        &format!("[compute] {algorithm_label}: 0/{work_len}"),
    );

    let mut batch: Vec<NewResult> = Vec::with_capacity(FLUSH_BATCH);
    let mut total_done: usize = 0;

    for exp in &context.experiments {
        let params = exp.parse_params();
        let similarity = build_similarity(&params);

        // Warmup once per (implementation, experiment).
        let warmup_pairs: Vec<(i32, i32)> = context
            .id_pairs
            .iter()
            .copied()
            .take(GLOBAL_WARMUP_PAIR_SAMPLE)
            .collect();
        for _ in 0..params.n_warmup {
            for (left_id, right_id) in &warmup_pairs {
                let left = context
                    .spectra_map
                    .get(left_id)
                    .expect("left spectrum not found");
                let right = context
                    .spectra_map
                    .get(right_id)
                    .expect("right spectrum not found");
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

        for &(left_id, right_id) in &context.id_pairs {
            let left = context
                .spectra_map
                .get(&left_id)
                .expect("left spectrum not found");
            let right = context
                .spectra_map
                .get(&right_id)
                .expect("right spectrum not found");

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
            inc_progress(progress, 1);

            if batch.len() >= FLUSH_BATCH {
                flush_results(conn, &mut batch);
            }

            if total_done == work_len || total_done.is_multiple_of(SUBSTEP_UPDATE_INTERVAL) {
                emit(
                    progress,
                    &format!("[compute] {algorithm_label}: {total_done}/{work_len}"),
                );
            }
        }
    }

    flush_results(conn, &mut batch);
    emit(
        progress,
        &format!("[compute] {algorithm_label}: {total_done} pairs computed"),
    );
    total_done
}

fn compute_rust_algorithm_for_kind(
    conn: &mut SqliteConnection,
    context: &ComputeContext,
    spec: RustAlgoSpec,
    progress: &mut Option<&mut dyn StageProgress>,
) -> usize {
    match spec.kind {
        RustAlgoKind::CosineHungarian => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    HungarianCosine::new(params.mz_power, params.intensity_power, params.tolerance)
                })
            })
        }
        RustAlgoKind::CosineGreedy => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    GreedyCosine::new(params.mz_power, params.intensity_power, params.tolerance)
                })
            })
        }
        RustAlgoKind::LinearCosine => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    LinearCosine::new(params.mz_power, params.intensity_power, params.tolerance)
                })
            })
        }
        RustAlgoKind::ModifiedCosine => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    ModifiedHungarianCosine::new(
                        params.mz_power,
                        params.intensity_power,
                        params.tolerance,
                    )
                })
            })
        }
        RustAlgoKind::ModifiedGreedyCosine => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    ModifiedGreedyCosine::new(
                        params.mz_power,
                        params.intensity_power,
                        params.tolerance,
                    )
                })
            })
        }
        RustAlgoKind::ModifiedLinearCosine => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    ModifiedLinearCosine::new(
                        params.mz_power,
                        params.intensity_power,
                        params.tolerance,
                    )
                })
            })
        }
        RustAlgoKind::EntropySimilarityWeighted => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    LinearEntropy::new(
                        params.mz_power,
                        params.intensity_power,
                        params.tolerance,
                        true,
                    )
                })
            })
        }
        RustAlgoKind::EntropySimilarityUnweighted => {
            compute_rust_algorithm(conn, context, spec.algorithm_name, progress, |params| {
                build_similarity_or_panic(spec.algorithm_name, params, || {
                    LinearEntropy::new(
                        params.mz_power,
                        params.intensity_power,
                        params.tolerance,
                        false,
                    )
                })
            })
        }
    }
}

fn build_sirius_merged_context(context: &ComputeContext) -> ComputeContext {
    let min_tolerance = context
        .experiments
        .iter()
        .map(|exp| exp.parse_params().tolerance)
        .fold(f64::INFINITY, f64::min);

    let merger = SiriusMergeClosePeaks::new(min_tolerance)
        .expect("failed to build SiriusMergeClosePeaks from experiment tolerance");

    let merged_map: HashMap<i32, GenericSpectrum<f64, f64>> = context
        .spectra_map
        .iter()
        .map(|(&id, spectrum)| (id, merger.process(spectrum)))
        .collect();

    ComputeContext {
        experiments: context.experiments.clone(),
        spectrum_ids: context.spectrum_ids.clone(),
        spectra_map: merged_map,
        id_pairs: context.id_pairs.clone(),
    }
}

fn build_entropy_cleaned_context(context: &ComputeContext) -> ComputeContext {
    let cleaner = MsEntropyCleanSpectrum::<f64>::builder()
        .build()
        .expect("failed to build MsEntropyCleanSpectrum with default parameters");

    let cleaned_map: HashMap<i32, GenericSpectrum<f64, f64>> = context
        .spectra_map
        .iter()
        .map(|(&id, spectrum)| (id, cleaner.process(spectrum)))
        .collect();

    ComputeContext {
        experiments: context.experiments.clone(),
        spectrum_ids: context.spectrum_ids.clone(),
        spectra_map: cleaned_map,
        id_pairs: context.id_pairs.clone(),
    }
}

fn run_rust_compute_passes(
    conn: &mut SqliteConnection,
    context: &ComputeContext,
    progress: &mut Option<&mut dyn StageProgress>,
) {
    emit(progress, "[compute] Starting Rust algorithms");

    let any_sirius = RUST_ALGO_SPECS
        .iter()
        .any(|s| s.preprocessing == Preprocessing::SiriusMerge);
    let any_entropy = RUST_ALGO_SPECS
        .iter()
        .any(|s| s.preprocessing == Preprocessing::MsEntropyClean);
    let sirius_context = if any_sirius {
        Some(build_sirius_merged_context(context))
    } else {
        None
    };
    let entropy_context = if any_entropy {
        Some(build_entropy_cleaned_context(context))
    } else {
        None
    };

    for spec in RUST_ALGO_SPECS {
        let effective_context = match spec.preprocessing {
            Preprocessing::None => context,
            Preprocessing::SiriusMerge => {
                sirius_context.as_ref().expect("sirius context must exist")
            }
            Preprocessing::MsEntropyClean => {
                entropy_context.as_ref().expect("entropy context must exist")
            }
        };
        compute_rust_algorithm_for_kind(conn, effective_context, spec, progress);
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
