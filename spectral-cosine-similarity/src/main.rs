use clap::Parser;
use spectral_cosine_similarity::{compute, db, download, prepare, report};

#[derive(Parser)]
#[command(about = "Spectral cosine similarity benchmark pipeline")]
struct Cli {
    /// Maximum number of spectra to load
    #[arg(long)]
    max_spectra: usize,

    /// Number of random pairs to sample
    #[arg(long)]
    num_pairs: usize,
}

fn main() {
    let cli = Cli::parse();
    run_pipeline(cli.max_spectra, cli.num_pairs);
}

fn run_pipeline(max_spectra: usize, num_pairs: usize) {
    compute::preflight_python_environment();
    let conn = &mut db::establish_connection();
    db::initialize(conn);
    download::run();
    prepare::run(conn, max_spectra);
    compute::run(conn, max_spectra, num_pairs);
    report::generate(
        conn,
        &report::ReportConfig {
            requested_max_spectra: max_spectra,
            ..Default::default()
        },
    );
}
