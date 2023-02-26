//! Command line arguments
use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

use crate::page::PageSize;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Page title (max. 64 characters)
    #[arg(short, long, default_value = "PaperAge")]
    pub title: String,

    /// Output file name. Use - for STDOUT.
    #[arg(short, long, default_value = "out.pdf")]
    pub output: PathBuf,

    /// Paper size
    #[arg(short = 's', long, default_value_t = PageSize::A4)]
    pub page_size: PageSize,

    /// Overwrite the output file if it already exists
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// Draw a grid pattern for debugging layout issues
    #[arg(short, long, default_value_t = false)]
    pub grid: bool,

    /// Print out the license for the embedded fonts
    #[arg(long, default_value_t = false, exclusive = true)]
    pub fonts_license: bool,

    /// Verbose output for debugging
    #[clap(flatten)]
    pub verbose: Verbosity,

    /// Disable drawing of footer
    #[arg(short, long, default_value_t = false)]
    pub no_footer: bool,

    /// The path to the file to read. Defaults to standard input. Max. ~1.9KB.
    pub input: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_args() {
        use clap::CommandFactory;
        super::Args::command().debug_assert()
    }

    #[test]
    fn test_args() {
        let args = Args::parse_from([
            "paper-age",
            "-f",
            "-g",
            "--title",
            "Hello",
            "--output",
            "test.pdf",
            "input.txt",
        ]);
        assert!(args.force);
        assert!(args.grid);
        assert_eq!(args.title, "Hello");
        assert_eq!(args.output.to_str().unwrap(), "test.pdf");
        assert_eq!(args.input.unwrap().to_str().unwrap(), "input.txt");
    }

    #[test]
    fn test_defaults() {
        let args = Args::parse_from(["paper-age"]);
        assert_eq!(args.title, "PaperAge");
        assert_eq!(args.output.to_str().unwrap(), "out.pdf");
        assert_eq!(args.input, None);
        assert!(!args.force);
    }

    #[test]
    fn test_fonts_license() {
        let args = Args::parse_from(["paper-age", "--fonts-license"]);
        assert!(args.fonts_license);
    }

    #[test]
    fn test_fonts_license_conflict() -> Result<(), Box<dyn std::error::Error>> {
        let result = Args::try_parse_from(["paper-age", "--fonts-license", "--grid"]);

        assert!(result.is_err());

        Ok(())
    }
}
