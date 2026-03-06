use clap::Parser;
use spectral_cosine_similarity::progress::{
    FIXED_STAGE_UNITS, NON_COMPUTE_STAGE_COUNT, PipelineProgress, StageProgress,
};
use spectral_cosine_similarity::{compute, db, download, ntfy, prepare, report};
use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;

#[derive(Parser)]
#[command(about = "Spectral cosine similarity benchmark pipeline")]
struct Cli {
    /// Limit the number of spectra loaded (useful for quick tests)
    #[arg(long)]
    max_spectra: Option<usize>,

    /// Sample this many random pairs instead of all N*(N+1)/2 pairs
    #[arg(long)]
    num_pairs: Option<usize>,

    /// Enable ntfy notifications for section completion/failure/final completion.
    #[arg(long)]
    ntfy: bool,
}

fn main() {
    let cli = Cli::parse();
    let notifier = cli.ntfy.then(ntfy::NtfyNotifier::new);
    if let Some(n) = notifier.as_ref() {
        println!("[ntfy] Subscribe: {}", n.subscription_url());
    }

    let pipeline_started = Instant::now();
    let run_result = panic::catch_unwind(AssertUnwindSafe(|| {
        run_pipeline(cli.max_spectra, cli.num_pairs, notifier.as_ref())
    }));

    match run_result {
        Ok(()) => {
            if let Some(n) = notifier.as_ref() {
                n.notify(
                    &ntfy::format_pipeline_completed(pipeline_started.elapsed()),
                    &["white_check_mark"],
                    3,
                );
            }
        }
        Err(payload) => {
            if let Some(n) = notifier.as_ref() {
                n.notify(
                    &ntfy::format_pipeline_failed(
                        None,
                        pipeline_started.elapsed(),
                        &panic_message(payload.as_ref()),
                    ),
                    &["x"],
                    5,
                );
            }
            panic::resume_unwind(payload);
        }
    }
}

fn run_pipeline(
    max_spectra: Option<usize>,
    num_pairs: Option<usize>,
    notifier: Option<&ntfy::NtfyNotifier>,
) {
    eprintln!("[preflight] Checking Python runtime (`uv`) and required packages");
    compute::preflight_python_environment();

    let conn = &mut db::establish_connection();
    let total_units = NON_COMPUTE_STAGE_COUNT * FIXED_STAGE_UNITS;
    let mut progress = PipelineProgress::new(total_units);

    let stage_started = Instant::now();
    {
        let mut stage = progress.start_stage("Initialize DB", FIXED_STAGE_UNITS);
        stage.set_message("Initializing schema and metadata");
        db::initialize(conn);
        stage.inc(FIXED_STAGE_UNITS);
    }
    notify_stage(notifier, "Initialize DB", stage_started.elapsed());

    let stage_started = Instant::now();
    {
        let mut stage = progress.start_stage("Download", FIXED_STAGE_UNITS);
        download::run(Some(&mut stage));
    }
    notify_stage(notifier, "Download", stage_started.elapsed());

    let stage_started = Instant::now();
    {
        let mut stage = progress.start_stage("Prepare", FIXED_STAGE_UNITS);
        prepare::run_with_progress(conn, max_spectra, Some(&mut stage));
    }
    notify_stage(notifier, "Prepare", stage_started.elapsed());

    let compute_units = compute::estimate_work(conn, max_spectra, num_pairs);
    progress.add_total_units(compute_units);

    let stage_started = Instant::now();
    {
        let mut stage = progress.start_stage("Compute", compute_units);
        compute::run(conn, max_spectra, num_pairs, Some(&mut stage));
    }
    notify_stage(notifier, "Compute", stage_started.elapsed());

    let stage_started = Instant::now();
    {
        let mut stage = progress.start_stage("Report", FIXED_STAGE_UNITS);
        let report_cfg = report::ReportConfig {
            requested_max_spectra: max_spectra,
            ..report::ReportConfig::default()
        };
        report::generate(conn, &report_cfg, Some(&mut stage));
    }
    notify_stage(notifier, "Report", stage_started.elapsed());

    progress.finish_all();
}

fn notify_stage(
    notifier: Option<&ntfy::NtfyNotifier>,
    stage: &str,
    elapsed: std::time::Duration,
) {
    if let Some(n) = notifier {
        n.notify(
            &ntfy::format_stage_completed(stage, elapsed, None),
            &["white_check_mark"],
            3,
        );
    }
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
