use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hint::black_box;
use std::result::Result as StdResult;
use std::time::Instant;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use indicatif::{ProgressBar, ProgressStyle};
use mass_spectrometry::prelude::*;

use crate::benchmark_topology;
use crate::db;
use crate::models::*;
use crate::pair_selection::generate_pairs;
use crate::progress::StageProgress;
use crate::report;

const FLUSH_BATCH: usize = 500;
const CHART_UPDATE_INTERVAL: usize = 50_000;
const SUBSTEP_UPDATE_INTERVAL: usize = 5_000;
const RUST_LIBRARY_NAME: &str = "mass-spectrometry-traits";
const RUST_ALGORITHMS: [&str; 6] = [
    "CosineHungarian",
    "CosineGreedy",
    "ModifiedCosine",
    "ModifiedGreedyCosine",
    "EntropySimilarityWeighted",
    "EntropySimilarityUnweighted",
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
        panic!(
            "[compute] failed to build {algorithm_name} with params {:?}: {err:?}",
            params
        )
    })
}

struct ComputeContext {
    experiments: Vec<Experiment>,
    spectrum_ids: Vec<i32>,
    spectra_map: HashMap<i32, GenericSpectrum<f32, f32>>,
    id_pairs: Vec<(i32, i32)>,
}

fn set_substep(progress: &mut Option<&mut dyn StageProgress>, message: &str) {
    if let Some(p) = progress.as_deref_mut() {
        p.set_substep(message);
    } else {
        eprintln!("{message}");
    }
}

fn clear_substep(progress: &mut Option<&mut dyn StageProgress>) {
    if let Some(p) = progress.as_deref_mut() {
        p.clear_substep();
    }
}

fn inc_progress(progress: &mut Option<&mut dyn StageProgress>, units: u64) {
    if let Some(p) = progress.as_deref_mut() {
        p.inc(units);
    }
}

/// Compute similarities and timings for all implementations in a single pass,
/// interleaving Rust and Python in batches so that charts stay up to date.
pub fn run(conn: &mut SqliteConnection, max_spectra: Option<usize>) {
    run_with_progress(conn, max_spectra, None);
}

pub fn run_with_progress(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    progress: Option<&mut dyn StageProgress>,
) {
    run_with_matchms_and_progress(conn, max_spectra, run_matchms_default, progress);
}

pub fn run_with_matchms<F>(conn: &mut SqliteConnection, max_spectra: Option<usize>, run_matchms: F)
where
    F: Fn(Option<usize>),
{
    run_with_matchms_and_progress(conn, max_spectra, run_matchms, None);
}

pub fn run_with_matchms_and_progress<F>(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    run_matchms: F,
    mut progress: Option<&mut dyn StageProgress>,
) where
    F: Fn(Option<usize>),
{
    compute_all(conn, max_spectra, &run_matchms, &mut progress);
    set_substep(&mut progress, "[compute] Building timing summary");
    print_timing_report(conn);
    clear_substep(&mut progress);
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

fn run_matchms_default(max_spectra: Option<usize>) {
    let db = db::db_path(max_spectra);
    let batch_size = CHART_UPDATE_INTERVAL.to_string();
    let mut cmd = std::process::Command::new("uv");
    cmd.args([
        "run",
        "python3",
        "scripts/python_reference_compute.py",
        db,
        &batch_size,
    ]);
    if let Some(max_spectra) = max_spectra {
        cmd.arg(max_spectra.to_string());
    }
    let status = cmd
        .status()
        .expect("failed to run python_reference_compute.py");

    if !status.success() {
        panic!(
            "[compute] python_reference_compute.py exited with {status}. \
Install Python benchmark dependencies with `uv sync` in spectral-cosine-similarity/ \
and ensure both `matchms` and `ms_entropy` import successfully."
        );
    }
}

fn count_results_by_implementation(
    conn: &mut SqliteConnection,
    implementation_ids: &[i32],
    spectrum_ids_filter: Option<&[i32]>,
) -> HashMap<i32, i64> {
    if implementation_ids.is_empty() {
        return HashMap::new();
    }
    if let Some(ids) = spectrum_ids_filter
        && ids.is_empty()
    {
        return HashMap::new();
    }

    let rows: Vec<(i32, i64)> = if let Some(ids) = spectrum_ids_filter {
        crate::schema::results::table
            .filter(crate::schema::results::implementation_id.eq_any(implementation_ids))
            .filter(crate::schema::results::left_id.eq_any(ids))
            .filter(crate::schema::results::right_id.eq_any(ids))
            .group_by(crate::schema::results::implementation_id)
            .select((
                crate::schema::results::implementation_id,
                diesel::dsl::count_star(),
            ))
            .load(conn)
            .expect("failed to count filtered implementation results")
    } else {
        crate::schema::results::table
            .filter(crate::schema::results::implementation_id.eq_any(implementation_ids))
            .group_by(crate::schema::results::implementation_id)
            .select((
                crate::schema::results::implementation_id,
                diesel::dsl::count_star(),
            ))
            .load(conn)
            .expect("failed to count implementation results")
    };

    rows.into_iter().collect()
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

fn count_python_results(conn: &mut SqliteConnection, spectrum_ids_filter: Option<&[i32]>) -> i64 {
    let python_ids = python_implementation_ids(conn);
    count_results_by_implementation(conn, &python_ids, spectrum_ids_filter)
        .into_values()
        .sum()
}

fn tracked_implementation_ids(conn: &mut SqliteConnection) -> Vec<i32> {
    let mut ids: Vec<i32> = benchmark_topology::load_resolved_implementations(conn)
        .into_iter()
        .filter(|row| {
            (row.library_name == RUST_LIBRARY_NAME
                && RUST_ALGORITHMS.contains(&row.algorithm_name.as_str()))
                || row.library_language == "python"
        })
        .map(|row| row.implementation_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn load_limited_spectrum_ids(conn: &mut SqliteConnection, max_spectra: usize) -> Vec<i32> {
    let limit = i64::try_from(max_spectra).unwrap_or(i64::MAX);
    crate::schema::spectra::table
        .order(crate::schema::spectra::id.asc())
        .limit(limit)
        .select(crate::schema::spectra::id)
        .load(conn)
        .expect("failed to load limited spectrum ids")
}

/// Estimate remaining compute workload units.
///
/// One unit is one `(left_id, right_id, experiment_id, implementation_id)` result row
/// that still needs to be produced by either Rust or Python implementations.
pub fn estimate_remaining_work(conn: &mut SqliteConnection, max_spectra: Option<usize>) -> u64 {
    let spectrum_ids = max_spectra.map(|max| load_limited_spectrum_ids(conn, max));
    let n_spectra = if let Some(ids) = spectrum_ids.as_ref() {
        ids.len() as i64
    } else {
        crate::schema::spectra::table
            .select(diesel::dsl::count_star())
            .first::<i64>(conn)
            .expect("failed to count spectra")
    };
    let n_pairs = n_spectra.saturating_mul(n_spectra + 1) / 2;
    let experiments_count = crate::schema::experiments::table
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .expect("failed to count experiments");
    let expected_per_implementation = n_pairs.saturating_mul(experiments_count);
    if expected_per_implementation == 0 {
        return 0;
    }
    let spectrum_ids_filter = spectrum_ids.as_deref();
    let implementation_ids = tracked_implementation_ids(conn);
    let existing_by_implementation =
        count_results_by_implementation(conn, &implementation_ids, spectrum_ids_filter);

    implementation_ids
        .into_iter()
        .map(|implementation_id| {
            let existing = existing_by_implementation
                .get(&implementation_id)
                .copied()
                .unwrap_or(0);
            expected_per_implementation.saturating_sub(existing) as u64
        })
        .sum()
}

fn load_compute_context(conn: &mut SqliteConnection, max_spectra: Option<usize>) -> ComputeContext {
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
    let spectra_map: HashMap<i32, GenericSpectrum<f32, f32>> = all_spectra
        .iter()
        .map(|s| (s.id, s.to_generic_spectrum()))
        .collect();

    let id_pairs = generate_pairs(&spectrum_ids);

    ComputeContext {
        experiments,
        spectrum_ids,
        spectra_map,
        id_pairs,
    }
}

fn compute_rust_algorithm<F, B, S>(
    conn: &mut SqliteConnection,
    context: &ComputeContext,
    algorithm_name: &str,
    max_spectra: Option<usize>,
    run_matchms: &F,
    progress: &mut Option<&mut dyn StageProgress>,
    build_similarity: B,
) -> usize
where
    F: Fn(Option<usize>),
    B: Fn(&ExperimentParams) -> S,
    S: ScalarSimilarity<
            GenericSpectrum<f32, f32>,
            GenericSpectrum<f32, f32>,
            Similarity = StdResult<(f32, usize), SimilarityComputationError>,
        >,
{
    let impl_id = db::get_implementation_id(conn, algorithm_name, RUST_LIBRARY_NAME);
    let spectrum_ids_filter = max_spectra.map(|_| context.spectrum_ids.as_slice());
    let algorithm_label = algorithm_cli_label(algorithm_name, RUST_LIBRARY_NAME);

    // Load existing results to skip completed work
    let mut existing_query = crate::schema::results::table
        .filter(crate::schema::results::implementation_id.eq(impl_id))
        .into_boxed();
    if let Some(ids) = spectrum_ids_filter {
        existing_query = existing_query
            .filter(crate::schema::results::left_id.eq_any(ids))
            .filter(crate::schema::results::right_id.eq_any(ids));
    }
    let existing: HashSet<(i32, i32, i32)> = existing_query
        .order((
            crate::schema::results::left_id.asc(),
            crate::schema::results::right_id.asc(),
            crate::schema::results::experiment_id.asc(),
        ))
        .select((
            crate::schema::results::left_id,
            crate::schema::results::right_id,
            crate::schema::results::experiment_id,
        ))
        .load::<(i32, i32, i32)>(conn)
        .expect("failed to load existing results")
        .into_iter()
        .collect();

    // Work items: pairs not yet in results
    let mut work: Vec<(i32, i32, &Experiment)> = Vec::new();
    for &(left_id, right_id) in &context.id_pairs {
        for exp in &context.experiments {
            if !existing.contains(&(left_id, right_id, exp.id)) {
                work.push((left_id, right_id, exp));
            }
        }
    }

    if work.is_empty() {
        set_substep(
            progress,
            &format!("[compute] {algorithm_label}: nothing to compute"),
        );
        return 0;
    }

    set_substep(
        progress,
        &format!("[compute] {algorithm_label}: 0/{}", work.len()),
    );
    let work_len = work.len();

    let pb = if progress.is_none() {
        let pb = ProgressBar::new(work.len() as u64);
        pb.set_style(
            ProgressStyle::with_template(&format!(
                "[compute] {algorithm_label} {{bar:40}} {{pos}}/{{len}} [{{eta}}]"
            ))
            .expect("invalid compute progress style"),
        );
        Some(pb)
    } else {
        None
    };

    let mut batch: Vec<NewResult> = Vec::with_capacity(FLUSH_BATCH);
    let mut total_done: usize = 0;
    let mut next_chart_update: usize = CHART_UPDATE_INTERVAL;

    for (left_id, right_id, exp) in work {
        let left = context
            .spectra_map
            .get(&left_id)
            .expect("left spectrum not found");
        let right = context
            .spectra_map
            .get(&right_id)
            .expect("right spectrum not found");
        let params = exp.parse_params();
        let similarity = build_similarity(&params);

        // Warmup the specific algorithm implementation for this pair/parameter set.
        for _ in 0..params.n_warmup {
            let _ = black_box(similarity.similarity(black_box(left), black_box(right)))
                .unwrap_or_else(|err| {
                    panic!(
                        "[compute] {algorithm_label} warmup failed for left_id={left_id}, right_id={right_id}, experiment_id={}: {err:?}",
                        exp.id
                    )
                });
        }

        // Timed runs
        let mut times_ns: Vec<u128> = Vec::with_capacity(params.n_reps as usize);
        let mut last_result = (0.0f32, 0usize);
        for _ in 0..params.n_reps {
            let t0 = Instant::now();
            last_result = black_box(similarity.similarity(black_box(left), black_box(right)))
                .unwrap_or_else(|err| {
                    panic!(
                        "[compute] {algorithm_label} run failed for left_id={left_id}, right_id={right_id}, experiment_id={}: {err:?}",
                        exp.id
                    )
                });
            times_ns.push(t0.elapsed().as_nanos());
        }

        let (score, matches) = last_result;
        let matches = i32::try_from(matches).unwrap_or_else(|_| {
            panic!(
                "[compute] {algorithm_label} produced matches={} that does not fit i32 for left_id={left_id}, right_id={right_id}, experiment_id={}",
                matches, exp.id
            )
        });
        times_ns.sort_unstable();
        let median_ns = times_ns[params.n_reps as usize / 2];
        let median_us = median_ns as f32 / 1000.0;

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

            if total_done >= next_chart_update {
                next_chart_update = total_done + CHART_UPDATE_INTERVAL;
                set_substep(progress, "[compute] Running Python implementations");
                let before_python = count_python_results(conn, spectrum_ids_filter);
                if let Some(pb) = pb.as_ref() {
                    pb.suspend(|| {
                        run_matchms(max_spectra);
                    });
                } else {
                    run_matchms(max_spectra);
                }
                let after_python = count_python_results(conn, spectrum_ids_filter);
                let added_python_rows = after_python.saturating_sub(before_python);
                inc_progress(progress, added_python_rows as u64);

                set_substep(progress, "[compute] Refreshing report charts");
                let report_config = report::ReportConfig::default();
                if progress.is_some() {
                    let report_progress =
                        progress.as_deref_mut().map(|p| p as &mut dyn StageProgress);
                    report::generate(conn, &report_config, report_progress);
                } else {
                    report::generate(conn, &report_config, None);
                }
            }
        }

        if let Some(pb) = pb.as_ref() {
            pb.inc(1);
        } else if total_done == work_len || total_done.is_multiple_of(SUBSTEP_UPDATE_INTERVAL) {
            set_substep(
                progress,
                &format!("[compute] {algorithm_label}: {total_done}/{work_len}",),
            );
        }
    }

    flush_results(conn, &mut batch);
    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    set_substep(
        progress,
        &format!("[compute] {algorithm_label}: {total_done} pairs computed"),
    );

    total_done
}

fn compute_all<F>(
    conn: &mut SqliteConnection,
    max_spectra: Option<usize>,
    run_matchms: &F,
    progress: &mut Option<&mut dyn StageProgress>,
) where
    F: Fn(Option<usize>),
{
    let context = load_compute_context(conn, max_spectra);
    let spectrum_ids_filter = max_spectra.map(|_| context.spectrum_ids.as_slice());

    set_substep(progress, "[compute] Starting Rust algorithms");

    compute_rust_algorithm(
        conn,
        &context,
        "CosineHungarian",
        max_spectra,
        run_matchms,
        progress,
        |params| {
            build_similarity_or_panic("CosineHungarian", params, || {
                HungarianCosine::new(params.mz_power, params.intensity_power, params.tolerance)
            })
        },
    );
    compute_rust_algorithm(
        conn,
        &context,
        "CosineGreedy",
        max_spectra,
        run_matchms,
        progress,
        |params| {
            build_similarity_or_panic("CosineGreedy", params, || {
                GreedyCosine::new(params.mz_power, params.intensity_power, params.tolerance)
            })
        },
    );
    compute_rust_algorithm(
        conn,
        &context,
        "ModifiedCosine",
        max_spectra,
        run_matchms,
        progress,
        |params| {
            build_similarity_or_panic("ModifiedCosine", params, || {
                ModifiedHungarianCosine::new(
                    params.mz_power,
                    params.intensity_power,
                    params.tolerance,
                )
            })
        },
    );
    compute_rust_algorithm(
        conn,
        &context,
        "ModifiedGreedyCosine",
        max_spectra,
        run_matchms,
        progress,
        |params| {
            build_similarity_or_panic("ModifiedGreedyCosine", params, || {
                ModifiedGreedyCosine::new(params.mz_power, params.intensity_power, params.tolerance)
            })
        },
    );
    compute_rust_algorithm(
        conn,
        &context,
        "EntropySimilarityWeighted",
        max_spectra,
        run_matchms,
        progress,
        |params| {
            build_similarity_or_panic("EntropySimilarityWeighted", params, || {
                EntropySimilarity::weighted(params.tolerance)
            })
        },
    );
    compute_rust_algorithm(
        conn,
        &context,
        "EntropySimilarityUnweighted",
        max_spectra,
        run_matchms,
        progress,
        |params| {
            build_similarity_or_panic("EntropySimilarityUnweighted", params, || {
                EntropySimilarity::unweighted(params.tolerance)
            })
        },
    );

    // Final matchms run to catch any remaining work
    set_substep(progress, "[compute] Final Python implementation pass");
    let before_python = count_python_results(conn, spectrum_ids_filter);
    run_matchms(max_spectra);
    let after_python = count_python_results(conn, spectrum_ids_filter);
    let added_python_rows = after_python.saturating_sub(before_python);
    inc_progress(progress, added_python_rows as u64);
}

fn print_timing_report(conn: &mut SqliteConnection) {
    let resolved_rows = benchmark_topology::load_resolved_implementations(conn);
    let mut by_algorithm: BTreeMap<String, Vec<(String, i32, bool)>> = BTreeMap::new();
    let mut reference_by_algorithm: HashMap<String, (String, String, i32)> = HashMap::new();
    for row in resolved_rows {
        by_algorithm
            .entry(row.algorithm_name.clone())
            .or_default()
            .push((
                row.library_name.clone(),
                row.implementation_id,
                row.is_reference,
            ));

        if let Some(reference_implementation_id) = row.canonical_reference_implementation_id {
            let reference_library =
                row.canonical_reference_library_name
                    .clone()
                    .unwrap_or_else(|| {
                        panic!(
                            "missing canonical reference library for algorithm '{}'",
                            row.algorithm_name
                        )
                    });
            let reference_info = (
                row.canonical_algorithm_name.clone(),
                reference_library,
                reference_implementation_id,
            );
            match reference_by_algorithm.entry(row.algorithm_name.clone()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(reference_info);
                }
                std::collections::hash_map::Entry::Occupied(entry) => {
                    if entry.get() != &reference_info {
                        panic!(
                            "algorithm '{}' has conflicting canonical references",
                            row.algorithm_name
                        );
                    }
                }
            }
        }
    }

    for libraries in by_algorithm.values_mut() {
        libraries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    }

    eprintln!("\n=== Timing Report ===\n");

    let mut algorithm_names: Vec<String> = by_algorithm.keys().cloned().collect();
    algorithm_names.sort();

    for algorithm in algorithm_names {
        let libraries = by_algorithm
            .get(&algorithm)
            .expect("algorithm key must exist in grouped timing report");
        let rust_impl = libraries
            .iter()
            .find(|(library, _, _)| library == RUST_LIBRARY_NAME)
            .map(|(_, implementation_id, _)| *implementation_id);

        let canonical_reference = reference_by_algorithm.get(&algorithm);
        let canonical_algorithm = canonical_reference
            .map(|(canonical_algorithm_name, _, _)| canonical_algorithm_name.as_str())
            .unwrap_or(algorithm.as_str());
        let reference_impl =
            canonical_reference.map(|(_, reference_library, implementation_id)| {
                (reference_library.as_str(), *implementation_id)
            });

        let rust_stats = rust_impl.and_then(|implementation_id| {
            timing_stats(
                conn,
                implementation_id,
                &format!("{algorithm} ({RUST_LIBRARY_NAME})"),
            )
        });
        let reference_stats = reference_impl.and_then(|(reference_library, implementation_id)| {
            timing_stats(
                conn,
                implementation_id,
                &format!("{canonical_algorithm} ({reference_library})"),
            )
        });

        if let Some(ref stats) = rust_stats {
            print_algo_stats(stats);
        }
        if let Some(ref stats) = reference_stats {
            print_algo_stats(stats);
        }

        if let (Some(rust), Some(reference)) = (&rust_stats, &reference_stats)
            && rust.mean > 0.0
            && reference.mean > 0.0
        {
            let speedup = reference.mean / rust.mean;
            if let Some((reference_library, _)) = reference_impl {
                if canonical_algorithm == algorithm {
                    eprintln!("Speedup ({algorithm}, Rust vs {reference_library}): {speedup:.1}x");
                } else {
                    eprintln!(
                        "Speedup ({algorithm} vs {canonical_algorithm} reference, Rust vs {reference_library}): {speedup:.1}x"
                    );
                }
            }
        }

        for (library, implementation_id, is_reference) in libraries.iter() {
            if library == RUST_LIBRARY_NAME || *is_reference {
                continue;
            }
            if let Some(stats) = timing_stats(
                conn,
                *implementation_id,
                &format!("{algorithm} ({library})"),
            ) {
                print_algo_stats(&stats);
            }
        }
    }
}

struct AlgoStats {
    name: String,
    count: i64,
    mean: f64,
    median: f64,
    min: f64,
    max: f64,
}

fn timing_stats(
    conn: &mut SqliteConnection,
    implementation_id: i32,
    label: &str,
) -> Option<AlgoStats> {
    let rows: Vec<f32> = crate::schema::results::table
        .filter(crate::schema::results::implementation_id.eq(implementation_id))
        .select(crate::schema::results::median_time_us)
        .load(conn)
        .expect("failed to load timings");

    if rows.is_empty() {
        return None;
    }

    let count = rows.len() as i64;
    let sum: f64 = rows.iter().map(|&t| t as f64).sum();
    let mean = sum / count as f64;

    let mut sorted = rows.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2] as f64;
    let min = sorted[0] as f64;
    let max = sorted[sorted.len() - 1] as f64;

    Some(AlgoStats {
        name: label.to_string(),
        count,
        mean,
        median,
        min,
        max,
    })
}

fn print_algo_stats(s: &AlgoStats) {
    eprintln!(
        "{}: {} pairs, mean={:.1}us, median={:.1}us, min={:.1}us, max={:.1}us",
        s.name, s.count, s.mean, s.median, s.min, s.max
    );
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
        let matchms_modified =
            db::get_implementation_id(&mut conn, "ModifiedCosineApprox", "matchms");
        let matchms_modified_greedy =
            db::get_implementation_id(&mut conn, "ModifiedGreedyCosine", "matchms");
        assert!(python_impls.contains(&matchms_modified));
        assert!(python_impls.contains(&matchms_modified_greedy));
        assert_eq!(python_impls.len(), 6);
    }
}
