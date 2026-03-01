use clap::Parser;
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
    db::initialize(conn);

    download::run(cli.allow_unverified_download);

    prepare::run(conn, cli.max_spectra);
    compute::run(conn, cli.max_spectra);
    report::run(conn);
}
