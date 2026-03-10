use std::collections::HashMap;
use std::hint::black_box;
use std::process::{Command, Stdio};
use std::time::Instant;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;
use indicatif::{ProgressBar, ProgressStyle};
use mass_spectrometry::prelude::*;

use crate::db;
use crate::models::*;
use crate::pair_selection;

const FLUSH_BATCH: usize = 10_000;
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
    LinearCosine,
    ModifiedCosine,
    ModifiedLinearCosine,
    EntropyWeighted,
    EntropyUnweighted,
    ModifiedEntropyWeighted,
    ModifiedEntropyUnweighted,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct PythonAlgoSpec {
    algorithm_name: &'static str,
    library_name: &'static str,
}

const RUST_ALGO_SPECS: [(&str, Preprocessing, AlgoKind); 10] = [
    (
        "CosineHungarian",
        Preprocessing::None,
        AlgoKind::HungarianCosine,
    ),
    (
        "LinearCosine",
        Preprocessing::SiriusMerge,
        AlgoKind::LinearCosine,
    ),
    (
        "ModifiedCosine",
        Preprocessing::None,
        AlgoKind::ModifiedCosine,
    ),
    (
        "ModifiedLinearCosine",
        Preprocessing::SiriusMerge,
        AlgoKind::ModifiedLinearCosine,
    ),
    (
        "CosineHungarianMerged",
        Preprocessing::SiriusMerge,
        AlgoKind::HungarianCosine,
    ),
    (
        "ModifiedCosineMerged",
        Preprocessing::SiriusMerge,
        AlgoKind::ModifiedCosine,
    ),
    (
        "EntropySimilarityWeighted",
        Preprocessing::MsEntropyClean,
        AlgoKind::EntropyWeighted,
    ),
    (
        "EntropySimilarityUnweighted",
        Preprocessing::MsEntropyClean,
        AlgoKind::EntropyUnweighted,
    ),
    (
        "ModifiedLinearEntropyWeighted",
        Preprocessing::MsEntropyClean,
        AlgoKind::ModifiedEntropyWeighted,
    ),
    (
        "ModifiedLinearEntropyUnweighted",
        Preprocessing::MsEntropyClean,
        AlgoKind::ModifiedEntropyUnweighted,
    ),
];

#[cfg(test)]
const PYTHON_ALGO_SPECS: [PythonAlgoSpec; 6] = [
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
        algorithm_name: "ModifiedCosineHungarian",
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

type SpectraMap = HashMap<i32, GenericSpectrum>;

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
        .args([
            "run",
            "python3",
            "-c",
            "import matchms, ms_entropy, huggingface_hub, skfp",
        ])
        .status();
    match import_check {
        Ok(status) if status.success() => {}
        Ok(status) => {
            panic!(
                "[preflight] Python dependency check failed with {status}. \
                 Run `uv sync` in spectral-cosine-similarity/ and ensure \
                 `matchms`, `ms_entropy`, `huggingface_hub`, and `skfp` import successfully."
            );
        }
        Err(err) => {
            panic!("[preflight] failed to run python dependency check via `uv run`: {err}");
        }
    }
}

/// Compute similarities and timings for all implementations (production entry point).
pub fn run(conn: &mut SqliteConnection, num_pairs: usize) {
    run_with_python_runner(conn, num_pairs, run_python_default);
}

/// Compute similarities with an injectable Python runner (for tests).
pub fn run_with_python_runner<F>(conn: &mut SqliteConnection, num_pairs: usize, run_python: F)
where
    F: Fn(),
{
    let (experiments, spectra_map, id_pairs) = load_compute_context(conn, num_pairs);

    sql_query("DROP INDEX IF EXISTS idx_results_impl_pair_exp")
        .execute(conn)
        .expect("failed to drop results index before compute");

    run_rust_compute_passes(conn, &experiments, &spectra_map, &id_pairs);

    let python_impl_count = python_implementation_ids(conn).len() as u64;
    if python_impl_count > 0 {
        eprintln!("[compute] Python: running all algorithms");
        run_python();
        eprintln!("[compute] Python: complete");
    }

    sql_query(
        "CREATE INDEX IF NOT EXISTS idx_results_impl_pair_exp \
         ON results(implementation_id, left_id, right_id, experiment_id)",
    )
    .execute(conn)
    .expect("failed to recreate results index after compute");
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
    let total = batch.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template("[compute] DB flush: {bar:40} {pos}/{len} ({eta})")
            .expect("invalid progress bar template"),
    );

    batch.sort_unstable_by(|a, b| {
        (a.left_id, a.right_id, a.experiment_id, a.implementation_id).cmp(&(
            b.left_id,
            b.right_id,
            b.experiment_id,
            b.implementation_id,
        ))
    });

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        for chunk in batch.chunks(FLUSH_BATCH) {
            diesel::insert_into(crate::schema::results::table)
                .values(chunk)
                .execute(conn)
                .expect("failed to insert results");
            pb.inc(chunk.len() as u64);
        }
        Ok(())
    })
    .expect("failed to flush results transaction");

    pb.finish_and_clear();
    batch.clear();
}

fn load_compute_context(
    conn: &mut SqliteConnection,
    num_pairs: usize,
) -> (Vec<Experiment>, SpectraMap, Vec<(i32, i32)>) {
    let experiments = db::load_experiments(conn);

    let all_spectra: Vec<SpectrumRow> = crate::schema::spectra::table
        .order(crate::schema::spectra::id.asc())
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
    let pb = ProgressBar::new(id_pairs.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[compute] Saving pairs: {bar:40} {pos}/{len} ({eta})")
            .expect("invalid progress bar template"),
    );
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(crate::schema::selected_pairs::table)
            .execute(conn)
            .expect("failed to clear selected_pairs");
        for chunk in id_pairs.chunks(FLUSH_BATCH) {
            let new_pairs: Vec<NewSelectedPair> = chunk
                .iter()
                .map(|&(l, r)| NewSelectedPair {
                    left_id: l,
                    right_id: r,
                })
                .collect();
            diesel::insert_into(crate::schema::selected_pairs::table)
                .values(&new_pairs)
                .execute(conn)
                .expect("failed to insert selected_pairs");
            pb.inc(chunk.len() as u64);
        }
        Ok(())
    })
    .expect("failed to write selected_pairs");
    pb.finish_and_clear();

    (experiments, spectra_map, id_pairs)
}

fn compute_rust_algorithm<B, SIM, SP, Score>(
    conn: &mut SqliteConnection,
    experiments: &[Experiment],
    spectra_map: &HashMap<i32, SP>,
    id_pairs: &[(i32, i32)],
    algorithm_name: &str,
    build_similarity: B,
) -> usize
where
    Score: Into<f64> + Copy + Default,
    B: Fn(&ExperimentParams) -> SIM,
    SIM: ScalarSimilarity<
            SP,
            SP,
            Similarity = std::result::Result<(Score, usize), SimilarityComputationError>,
        >,
{
    let impl_id = db::get_implementation_id(conn, algorithm_name, RUST_LIBRARY_NAME);
    let algorithm_label = algorithm_cli_label(algorithm_name, RUST_LIBRARY_NAME);
    let work_len = id_pairs.len() * experiments.len();

    if work_len == 0 {
        eprintln!("[compute] {algorithm_label}: nothing to compute");
        return 0;
    }

    let pb = ProgressBar::new(work_len as u64);
    pb.set_style(
        ProgressStyle::with_template("[compute] {msg}: {bar:40} {pos}/{len} ({eta})")
            .expect("invalid progress bar template"),
    );
    pb.set_message(algorithm_label.clone());

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
            let right = spectra_map
                .get(&right_id)
                .expect("right spectrum not found");

            let mut times_ns: Vec<u128> = Vec::with_capacity(params.n_reps as usize);
            let mut last_result = (Score::default(), 0usize);
            for _ in 0..params.n_reps {
                let t0 = Instant::now();
                last_result = black_box(similarity.similarity(black_box(left), black_box(right)))
                    .unwrap_or_else(|err| {
                        panic!(
                            "[compute] {algorithm_label} failed for \
                                 ({left_id}, {right_id}), experiment={}: {err:?}",
                            exp.id
                        )
                    });
                times_ns.push(t0.elapsed().as_nanos());
            }

            let (score_raw, matches) = last_result;
            let score: f64 = score_raw.into();
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
            pb.set_position(total_done as u64);
        }
    }

    flush_results(conn, &mut batch);
    pb.finish_with_message(format!("{algorithm_label}: {total_done} pairs computed"));
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
        AlgoKind::HungarianCosine => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                HungarianCosine::new(p.mz_power, p.intensity_power, p.tolerance)
                    .expect("failed to build HungarianCosine")
            },
        ),
        AlgoKind::LinearCosine => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                LinearCosine::new(p.mz_power, p.intensity_power, p.tolerance)
                    .expect("failed to build LinearCosine")
            },
        ),
        AlgoKind::ModifiedCosine => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                ModifiedHungarianCosine::new(p.mz_power, p.intensity_power, p.tolerance)
                    .expect("failed to build ModifiedHungarianCosine")
            },
        ),
        AlgoKind::ModifiedLinearCosine => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                ModifiedLinearCosine::new(p.mz_power, p.intensity_power, p.tolerance)
                    .expect("failed to build ModifiedLinearCosine")
            },
        ),
        AlgoKind::EntropyWeighted => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                LinearEntropy::new(p.mz_power, p.intensity_power, p.tolerance, true)
                    .expect("failed to build LinearEntropy weighted")
            },
        ),
        AlgoKind::EntropyUnweighted => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                LinearEntropy::new(p.mz_power, p.intensity_power, p.tolerance, false)
                    .expect("failed to build LinearEntropy unweighted")
            },
        ),
        AlgoKind::ModifiedEntropyWeighted => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                ModifiedLinearEntropy::new(p.mz_power, p.intensity_power, p.tolerance, true)
                    .expect("failed to build ModifiedLinearEntropy weighted")
            },
        ),
        AlgoKind::ModifiedEntropyUnweighted => compute_rust_algorithm(
            conn,
            experiments,
            spectra_map,
            id_pairs,
            algorithm_name,
            |p| {
                ModifiedLinearEntropy::new(p.mz_power, p.intensity_power, p.tolerance, false)
                    .expect("failed to build ModifiedLinearEntropy unweighted")
            },
        ),
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
    let cleaner = MsEntropyCleanSpectrum::builder()
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
        run_algo(
            conn,
            experiments,
            effective_map,
            id_pairs,
            algorithm_name,
            kind,
        );
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
        let matchms_modified_hungarian =
            db::get_implementation_id(&mut conn, "ModifiedCosineHungarian", "matchms");
        assert!(python_impls.contains(&matchms_modified_hungarian));
        assert_eq!(python_impls.len(), 6);
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
