use clap::Parser;
use spectral_cosine_similarity::{compute, db, fingerprint, prepare, report};

#[derive(Parser)]
#[command(about = "Spectral cosine similarity benchmark pipeline")]
struct Cli {
    /// Number of random pairs to sample
    #[arg(long)]
    num_pairs: usize,
}

fn main() {
    let cli = Cli::parse();
    run_pipeline(cli.num_pairs);
}

fn run_pipeline(num_pairs: usize) {
    compute::preflight_python_environment();
    let conn = &mut db::establish_connection();
    db::initialize(conn);
    prepare::run(conn);
    compute::run(conn, num_pairs);
    fingerprint::run(conn);
    report::generate(conn, &report::ReportConfig::default());
}
