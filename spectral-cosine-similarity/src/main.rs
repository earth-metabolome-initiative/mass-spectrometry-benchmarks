use clap::Parser;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use spectral_cosine_similarity::progress::{
    FIXED_STAGE_UNITS, NON_COMPUTE_STAGE_COUNT, PipelineProgress, StageProgress,
};
use spectral_cosine_similarity::{compute, db, download, ntfy::NtfyNotifier, prepare, report};
use std::any::Any;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(about = "Spectral cosine similarity benchmark pipeline")]
struct Cli {
    /// Limit the number of spectra loaded (useful for quick tests)
    #[arg(long)]
    max_spectra: Option<usize>,

    /// Enable ntfy notifications for section completion/failure/final completion.
    #[arg(long)]
    ntfy: bool,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

fn main() {
    let cli = Cli::parse();
    let mut notifier = cli.ntfy.then(NtfyNotifier::new_random_topic);
    if let Some(notifier) = notifier.as_ref() {
        println!("[ntfy] Subscribe: {}", notifier.subscription_url());
    }

    let pipeline_started = Instant::now();
    let mut stage_hint: Option<&'static str> = None;
    let run_result = panic::catch_unwind(AssertUnwindSafe(|| {
        run_pipeline(
            cli.max_spectra,
            &mut notifier,
            &mut stage_hint,
            pipeline_started,
        )
    }));

    if let Err(payload) = run_result {
        if let Some(notifier) = notifier.as_ref() {
            notifier.notify_pipeline_failed(
                stage_hint,
                pipeline_started.elapsed(),
                &panic_message(payload.as_ref()),
            );
        }
        panic::resume_unwind(payload);
    }
}

fn run_pipeline(
    max_spectra: Option<usize>,
    notifier: &mut Option<NtfyNotifier>,
    stage_hint: &mut Option<&'static str>,
    pipeline_started: Instant,
) {
    *stage_hint = Some("Python preflight");
    eprintln!("[preflight] Checking Python runtime (`uv`) and required packages");
    compute::preflight_python_environment();

    let conn = &mut db::establish_connection(max_spectra);
    let total_units = NON_COMPUTE_STAGE_COUNT * FIXED_STAGE_UNITS;
    let mut progress = PipelineProgress::new(total_units);

    *stage_hint = Some("Initialize DB");
    let initialize_started = Instant::now();
    {
        let mut stage = progress.start_stage("Initialize DB", FIXED_STAGE_UNITS);
        stage.set_substep("Initializing schema and metadata");
        db::initialize(conn);
        stage.inc(FIXED_STAGE_UNITS);
    }
    notify_stage_completion(
        notifier,
        "Initialize DB",
        initialize_started.elapsed(),
        Some(format!("{} experiments configured", experiment_count(conn))),
    );

    *stage_hint = Some("Download");
    let download_started = Instant::now();
    {
        let mut stage = progress.start_stage("Download", FIXED_STAGE_UNITS);
        download::run_with_progress(Some(&mut stage));
    }
    let download_metric = download_size_metric(Path::new(download::DATASET_PATH));
    notify_stage_completion(
        notifier,
        "Download",
        download_started.elapsed(),
        download_metric,
    );

    *stage_hint = Some("Prepare");
    let prepare_started = Instant::now();
    let spectra_before = spectra_count(conn);
    {
        let mut stage = progress.start_stage("Prepare", FIXED_STAGE_UNITS);
        prepare::run_with_progress(conn, max_spectra, Some(&mut stage));
    }
    let spectra_after = spectra_count(conn);
    notify_stage_completion(
        notifier,
        "Prepare",
        prepare_started.elapsed(),
        Some(format!(
            "{} spectra added ({} total)",
            spectra_after.saturating_sub(spectra_before),
            spectra_after
        )),
    );

    let compute_units = compute::estimate_remaining_work(conn, max_spectra);
    progress.add_total_units(compute_units);

    *stage_hint = Some("Compute");
    let compute_started = Instant::now();
    let results_before = results_count(conn);
    {
        let mut stage = progress.start_stage("Compute", compute_units);
        compute::run_with_progress_and_notifier(
            conn,
            max_spectra,
            Some(&mut stage),
            notifier.as_ref(),
        );
    }
    let results_after = results_count(conn);
    notify_stage_completion(
        notifier,
        "Compute",
        compute_started.elapsed(),
        Some(format!(
            "{} benchmark rows added ({} total)",
            results_after.saturating_sub(results_before),
            results_after
        )),
    );

    *stage_hint = Some("Report");
    let report_started = Instant::now();
    let artifact_count;
    {
        let mut stage = progress.start_stage("Report", FIXED_STAGE_UNITS);
        let report_cfg = report::ReportConfig {
            requested_max_spectra: max_spectra,
            ..report::ReportConfig::default()
        };
        let artifacts = report::generate(conn, &report_cfg, Some(&mut stage));
        artifact_count = usize::from(artifacts.timing_svg.is_some())
            + usize::from(artifacts.rmse_svg.is_some())
            + 1;
    }
    notify_stage_completion(
        notifier,
        "Report",
        report_started.elapsed(),
        Some(format!("{artifact_count} artifacts generated")),
    );

    progress.finish_all();
    *stage_hint = None;
    if let Some(notifier) = notifier.as_ref() {
        notifier.notify_pipeline_completed(pipeline_started.elapsed());
    }
}

fn notify_stage_completion(
    notifier: &mut Option<NtfyNotifier>,
    stage: &'static str,
    elapsed: std::time::Duration,
    key_count: Option<String>,
) {
    if let Some(notifier) = notifier.as_ref() {
        notifier.notify_stage_completed(stage, elapsed, key_count.as_deref());
    }
}

fn query_count(conn: &mut diesel::sqlite::SqliteConnection, query: &str) -> i64 {
    sql_query(query)
        .get_result::<CountRow>(conn)
        .unwrap_or_else(|err| panic!("failed to query count using '{query}': {err}"))
        .n
}

fn experiment_count(conn: &mut diesel::sqlite::SqliteConnection) -> i64 {
    query_count(conn, "SELECT COUNT(*) AS n FROM experiments")
}

fn spectra_count(conn: &mut diesel::sqlite::SqliteConnection) -> i64 {
    query_count(conn, "SELECT COUNT(*) AS n FROM spectra")
}

fn results_count(conn: &mut diesel::sqlite::SqliteConnection) -> i64 {
    query_count(conn, "SELECT COUNT(*) AS n FROM results")
}

fn download_size_metric(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    Some(format!(
        "{:.1} MB downloaded",
        metadata.len() as f64 / 1_048_576.0
    ))
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(msg) = payload.downcast_ref::<&'static str>() {
        return (*msg).to_string();
    }
    if let Some(msg) = payload.downcast_ref::<String>() {
        return msg.clone();
    }
    "panic payload was not a string".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn stage_count_queries_match_schema_table_names() {
        let temp_dir = TempDir::new().expect("failed to create temporary test directory");
        let db_path = temp_dir.path().join("benchmark.db");
        let mut conn = db::establish_connection_at(&db_path);
        db::initialize(&mut conn);

        assert_eq!(experiment_count(&mut conn), 4);
        assert_eq!(spectra_count(&mut conn), 0);
        assert_eq!(results_count(&mut conn), 0);
    }
}
