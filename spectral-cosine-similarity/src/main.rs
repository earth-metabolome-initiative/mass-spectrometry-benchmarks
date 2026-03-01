use clap::Parser;
use spectral_cosine_similarity::progress::{
    FIXED_STAGE_UNITS, NON_COMPUTE_STAGE_COUNT, PipelineProgress, StageProgress,
};
use spectral_cosine_similarity::{compute, db, download, prepare, report};

#[derive(Parser)]
#[command(about = "Spectral cosine similarity benchmark pipeline")]
struct Cli {
    /// Limit the number of spectra loaded (useful for quick tests)
    #[arg(long)]
    max_spectra: Option<usize>,

    /// Continue download flow even if checksum verification fails.
    #[arg(long)]
    allow_unverified_download: bool,
}

fn main() {
    let cli = Cli::parse();

    let conn = &mut db::establish_connection(cli.max_spectra);
    let total_units = NON_COMPUTE_STAGE_COUNT * FIXED_STAGE_UNITS;
    let mut progress = PipelineProgress::new(total_units);

    {
        let mut stage = progress.start_stage("Initialize DB", FIXED_STAGE_UNITS);
        stage.set_substep("Initializing schema and metadata");
        db::initialize(conn);
        stage.inc(FIXED_STAGE_UNITS);
    }

    {
        let mut stage = progress.start_stage("Download", FIXED_STAGE_UNITS);
        download::run_with_progress(cli.allow_unverified_download, Some(&mut stage));
    }

    {
        let mut stage = progress.start_stage("Prepare", FIXED_STAGE_UNITS);
        prepare::run_with_progress(conn, cli.max_spectra, Some(&mut stage));
    }

    let compute_units = compute::estimate_remaining_work(conn, cli.max_spectra);
    progress.add_total_units(compute_units);

    {
        let mut stage = progress.start_stage("Compute", compute_units);
        compute::run_with_progress(conn, cli.max_spectra, Some(&mut stage));
    }

    {
        let mut stage = progress.start_stage("Report", FIXED_STAGE_UNITS);
        report::generate(conn, &report::ReportConfig::default(), Some(&mut stage));
    }

    progress.finish_all();
}
