use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use indicatif::{ProgressBar, ProgressStyle};
use mass_spectrometry::prelude::*;

use crate::db;
use crate::models::*;
use crate::pair_selection::generate_pairs;
use crate::report;

const FLUSH_BATCH: usize = 500;
const CHART_UPDATE_INTERVAL: usize = 50_000;
const RUST_LIBRARY_NAME: &str = "mass-spectrometry-traits";

struct ComputeContext {
    experiments: Vec<Experiment>,
    spectra_map: HashMap<i32, GenericSpectrum<f32, f32>>,
    id_pairs: Vec<(i32, i32)>,
}

/// Compute similarities and timings for all implementations in a single pass,
/// interleaving Rust and Python in batches so that charts stay up to date.
pub fn run(conn: &mut SqliteConnection, max_spectra: Option<usize>) {
    run_with_matchms(conn, max_spectra, run_matchms_default);
}

pub fn run_with_matchms<F>(conn: &mut SqliteConnection, max_spectra: Option<usize>, run_matchms: F)
where
    F: Fn(Option<usize>),
{
    compute_all(conn, max_spectra, &run_matchms);
    print_timing_report(conn);
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
    eprintln!("[compute] Running matchms...");
    let db = db::db_path(max_spectra);
    let batch_size = CHART_UPDATE_INTERVAL.to_string();
    let status = std::process::Command::new("uv")
        .args([
            "run",
            "python3",
            "scripts/matchms_compute.py",
            db,
            &batch_size,
        ])
        .status()
        .expect("failed to run matchms_compute.py");

    if !status.success() {
        panic!(
            "[compute] matchms_compute.py exited with {status}. \
Install Python benchmark dependencies with `uv sync` in spectral-cosine-similarity/ \
and ensure both `matchms` and `ms_entropy` import successfully."
        );
    }
}

fn load_compute_context(conn: &mut SqliteConnection) -> ComputeContext {
    let experiments = db::load_experiments(conn);

    let all_spectra: Vec<SpectrumRow> = crate::schema::spectra::table
        .order(crate::schema::spectra::id.asc())
        .load(conn)
        .expect("failed to load spectra");
    let spectra_map: HashMap<i32, GenericSpectrum<f32, f32>> = all_spectra
        .iter()
        .map(|s| (s.id, s.to_generic_spectrum()))
        .collect();

    let spectra_ids: Vec<i32> = all_spectra.iter().map(|s| s.id).collect();
    let id_pairs = generate_pairs(&spectra_ids);

    ComputeContext {
        experiments,
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
    build_similarity: B,
) -> usize
where
    F: Fn(Option<usize>),
    B: Fn(&ExperimentParams) -> S,
    S: ScalarSimilarity<
            GenericSpectrum<f32, f32>,
            GenericSpectrum<f32, f32>,
            Similarity = (f32, u16),
        >,
{
    let impl_id = db::get_implementation_id(conn, algorithm_name, RUST_LIBRARY_NAME);

    // Load existing results to skip completed work
    let existing: HashSet<(i32, i32, i32)> = crate::schema::results::table
        .filter(crate::schema::results::implementation_id.eq(impl_id))
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
        eprintln!("[compute] {algorithm_name} (Rust): nothing to compute");
        return 0;
    }

    eprintln!("[compute] {algorithm_name} (Rust): {} pairs...", work.len());

    let pb = ProgressBar::new(work.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(&format!(
            "[compute] {algorithm_name} (Rust) {{bar:40}} {{pos}}/{{len}} [{{eta}}]"
        ))
        .unwrap(),
    );

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
            let _ = black_box(similarity.similarity(black_box(left), black_box(right)));
        }

        // Timed runs
        let mut times_ns: Vec<u128> = Vec::with_capacity(params.n_reps as usize);
        let mut last_result = (0.0f32, 0u16);
        for _ in 0..params.n_reps {
            let t0 = Instant::now();
            last_result = black_box(similarity.similarity(black_box(left), black_box(right)));
            times_ns.push(t0.elapsed().as_nanos());
        }

        let (score, matches) = last_result;
        times_ns.sort_unstable();
        let median_ns = times_ns[params.n_reps as usize / 2];
        let median_us = median_ns as f32 / 1000.0;

        batch.push(NewResult {
            left_id,
            right_id,
            experiment_id: exp.id,
            implementation_id: impl_id,
            score,
            matches: matches as i32,
            median_time_us: median_us,
        });

        total_done += 1;

        if batch.len() >= FLUSH_BATCH {
            flush_results(conn, &mut batch);

            if total_done >= next_chart_update {
                next_chart_update = total_done + CHART_UPDATE_INTERVAL;
                pb.suspend(|| {
                    run_matchms(max_spectra);
                });
                report::run(conn);
            }
        }

        pb.inc(1);
    }

    flush_results(conn, &mut batch);
    pb.finish_and_clear();

    eprintln!("[compute] {algorithm_name} (Rust): {total_done} pairs computed");

    total_done
}

fn compute_all<F>(conn: &mut SqliteConnection, max_spectra: Option<usize>, run_matchms: &F)
where
    F: Fn(Option<usize>),
{
    let context = load_compute_context(conn);

    compute_rust_algorithm(
        conn,
        &context,
        "CosineHungarian",
        max_spectra,
        run_matchms,
        |params| ExactCosine::new(params.mz_power, params.intensity_power, params.tolerance),
    );
    compute_rust_algorithm(
        conn,
        &context,
        "ModifiedCosine",
        max_spectra,
        run_matchms,
        |params| ModifiedCosine::new(params.mz_power, params.intensity_power, params.tolerance),
    );
    compute_rust_algorithm(
        conn,
        &context,
        "EntropySimilarityWeighted",
        max_spectra,
        run_matchms,
        |params| EntropySimilarity::weighted(params.tolerance),
    );
    compute_rust_algorithm(
        conn,
        &context,
        "EntropySimilarityUnweighted",
        max_spectra,
        run_matchms,
        |params| EntropySimilarity::unweighted(params.tolerance),
    );

    // Final matchms run to catch any remaining work
    run_matchms(max_spectra);
}

fn print_timing_report(conn: &mut SqliteConnection) {
    let implementation_rows: Vec<(String, String, i32, bool)> =
        crate::schema::implementations::table
            .inner_join(crate::schema::algorithms::table)
            .inner_join(crate::schema::libraries::table)
            .order((
                crate::schema::algorithms::name.asc(),
                crate::schema::libraries::name.asc(),
                crate::schema::implementations::id.asc(),
            ))
            .select((
                crate::schema::algorithms::name,
                crate::schema::libraries::name,
                crate::schema::implementations::id,
                crate::schema::implementations::is_reference,
            ))
            .load(conn)
            .expect("failed to load implementations for timing report");

    let mut by_algorithm: BTreeMap<String, Vec<(String, i32, bool)>> = BTreeMap::new();
    for (algorithm, library, implementation_id, is_reference) in implementation_rows {
        by_algorithm
            .entry(algorithm)
            .or_default()
            .push((library, implementation_id, is_reference));
    }
    for libraries in by_algorithm.values_mut() {
        libraries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    }

    eprintln!("\n=== Timing Report ===\n");

    for (algorithm, libraries) in by_algorithm {
        let rust_impl = libraries
            .iter()
            .find(|(library, _, _)| library == RUST_LIBRARY_NAME)
            .map(|(_, implementation_id, _)| *implementation_id);
        let reference_impl = libraries.iter().find(|(_, _, is_reference)| *is_reference);

        let rust_stats = rust_impl.and_then(|implementation_id| {
            timing_stats(
                conn,
                implementation_id,
                &format!("{algorithm} ({RUST_LIBRARY_NAME})"),
            )
        });
        let reference_stats =
            reference_impl.and_then(|(reference_library, implementation_id, _)| {
                timing_stats(
                    conn,
                    *implementation_id,
                    &format!("{algorithm} ({reference_library})"),
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
            if let Some((reference_library, _, _)) = reference_impl {
                eprintln!("Speedup ({algorithm}, Rust vs {reference_library}): {speedup:.1}x");
            }
        }

        for (library, implementation_id, is_reference) in libraries {
            if library == RUST_LIBRARY_NAME || is_reference {
                continue;
            }
            if let Some(stats) =
                timing_stats(conn, implementation_id, &format!("{algorithm} ({library})"))
            {
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
