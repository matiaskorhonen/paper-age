use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Page title (max. 64 characters)
    #[arg(long, default_value = "Paper Rage")]
    pub title: String,

    /// Output file name
    #[arg(short, long, default_value = "out.pdf")]
    pub output: String,

    /// Draw a grid pattern for debugging layout issues
    #[arg(short, long, default_value_t = false)]
    pub grid: bool,

    /// Print out the license for the embedded fonts
    #[arg(long, default_value_t = false, exclusive = true)]
    pub fonts_license: bool,

    /// Verbose output for debugging
    #[clap(flatten)]
    pub verbose: Verbosity,

    /// The path to the file to read, use - to read from stdin (max. ~1.5KB)
    pub input: Option<PathBuf>,
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
