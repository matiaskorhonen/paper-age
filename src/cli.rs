use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Page title (max. 60 characters)
    #[arg(long, default_value = "Paper Rage")]
    pub title: String,

    /// Output file name
    #[arg(short, long, default_value = "out.pdf")]
    pub output: String,

    /// Print out the license for the embedded fonts
    #[arg(long, default_value_t = false, exclusive = true)]
    pub fonts_license: bool,

    /// The path to the file to read, use - to read from stdin (max. 712 characters/bytes)
    pub input: Option<PathBuf>,
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
