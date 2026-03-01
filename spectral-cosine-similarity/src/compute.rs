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

/// Compute similarities and timings for all implementations in a single pass,
/// interleaving Rust and Python in batches so that charts stay up to date.
pub fn run(conn: &mut SqliteConnection, max_spectra: Option<usize>) {
    compute_all(conn, max_spectra);
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

fn run_matchms(max_spectra: Option<usize>) {
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
        eprintln!("[compute] WARNING: matchms_compute.py exited with {status}");
    }
}

fn compute_all(conn: &mut SqliteConnection, max_spectra: Option<usize>) {
    let impl_id = db::get_implementation_id(conn, "CosineHungarian", "mass-spectrometry-traits");
    let experiments = db::load_experiments(conn);

    let all_spectra: Vec<SpectrumRow> = crate::schema::spectra::table
        .load(conn)
        .expect("failed to load spectra");

    let spectra_map: HashMap<i32, GenericSpectrum<f32, f32>> = all_spectra
        .iter()
        .map(|s| (s.id, s.to_generic_spectrum()))
        .collect();

    let spectra_ids: Vec<i32> = all_spectra.iter().map(|s| s.id).collect();

    let id_pairs = generate_pairs(&spectra_ids);

    // Load existing results to skip completed work
    let existing: HashSet<(i32, i32, i32)> = crate::schema::results::table
        .filter(crate::schema::results::implementation_id.eq(impl_id))
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
    for &(left_id, right_id) in &id_pairs {
        for exp in &experiments {
            if !existing.contains(&(left_id, right_id, exp.id)) {
                work.push((left_id, right_id, exp));
            }
        }
    }

    if work.is_empty() {
        eprintln!("[compute] CosineHungarian (Rust): nothing to compute");
        run_matchms(max_spectra);
        return;
    }

    eprintln!("[compute] CosineHungarian (Rust): {} pairs...", work.len());

    let pb = ProgressBar::new(work.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[compute] CosineHungarian (Rust) {bar:40} {pos}/{len} [{eta}]",
        )
        .unwrap(),
    );

    let mut batch: Vec<NewResult> = Vec::with_capacity(FLUSH_BATCH);
    let mut total_done: usize = 0;
    let mut next_chart_update: usize = CHART_UPDATE_INTERVAL;

    for (left_id, right_id, exp) in &work {
        let left = &spectra_map[left_id];
        let right = &spectra_map[right_id];
        let params = exp.parse_params();

        let cosine = ExactCosine::new(params.mz_power, params.intensity_power, params.tolerance);

        // Warmup
        for _ in 0..params.n_warmup {
            let _ = black_box(cosine.similarity(black_box(left), black_box(right)));
        }

        // Timed runs
        let mut times_ns: Vec<u128> = Vec::with_capacity(params.n_reps as usize);
        let mut last_result = (0.0f32, 0u16);
        for _ in 0..params.n_reps {
            let t0 = Instant::now();
            last_result = black_box(cosine.similarity(black_box(left), black_box(right)));
            times_ns.push(t0.elapsed().as_nanos());
        }

        let (score, matches) = last_result;
        times_ns.sort_unstable();
        let median_ns = times_ns[params.n_reps as usize / 2];
        let median_us = median_ns as f32 / 1000.0;

        batch.push(NewResult {
            left_id: *left_id,
            right_id: *right_id,
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

    eprintln!("[compute] CosineHungarian (Rust): {total_done} pairs computed");

    // Final matchms run to catch any remaining work
    run_matchms(max_spectra);
}

fn print_timing_report(conn: &mut SqliteConnection) {
    let rust_impl_id =
        db::get_implementation_id(conn, "CosineHungarian", "mass-spectrometry-traits");
    let matchms_hungarian_id = db::get_implementation_id(conn, "CosineHungarian", "matchms");

    let rust_stats = timing_stats(
        conn,
        rust_impl_id,
        "CosineHungarian (mass-spectrometry-traits)",
    );
    let matchms_stats = timing_stats(conn, matchms_hungarian_id, "CosineHungarian (matchms)");

    eprintln!("\n=== Timing Report ===\n");

    if let Some(ref rs) = rust_stats {
        print_algo_stats(rs);
    }
    if let Some(ref ms) = matchms_stats {
        print_algo_stats(ms);
    }

    if let (Some(rs), Some(ms)) = (&rust_stats, &matchms_stats)
        && rs.mean > 0.0
        && ms.mean > 0.0
    {
        let speedup = ms.mean / rs.mean;
        eprintln!("Speedup (Rust vs matchms): {speedup:.1}x");
    }

    // Also report CosineGreedy if available
    if let Ok(greedy_impl_id) = crate::schema::implementations::table
        .inner_join(crate::schema::algorithms::table)
        .inner_join(crate::schema::libraries::table)
        .filter(crate::schema::algorithms::name.eq("CosineGreedy"))
        .filter(crate::schema::libraries::name.eq("matchms"))
        .select(crate::schema::implementations::id)
        .first::<i32>(conn)
        && let Some(gs) = timing_stats(conn, greedy_impl_id, "CosineGreedy (matchms)")
    {
        print_algo_stats(&gs);
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
