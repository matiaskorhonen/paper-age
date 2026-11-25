#[cfg(feature = "cli")]
use clap::CommandFactory;
#[cfg(feature = "cli")]
use clap_complete::{generate_to, shells::Shell};

#[cfg(feature = "cli")]
#[path = "src/cli.rs"]
mod cli;

#[cfg(feature = "cli")]
#[path = "src/page.rs"]
pub mod page;

fn main() -> std::io::Result<()> {
    #[cfg(feature = "cli")]
    {
        let out_dir = std::path::PathBuf::from(
            std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?,
        );
        let mut cmd = cli::Args::command();

        let man = clap_mangen::Man::new(cmd.clone());
        let mut buffer: Vec<u8> = Default::default();
        man.render(&mut buffer)?;

        // Write the man page and completions next to the binary (in the profile
        // directory, e.g. target/release), even though build scripts shouldn't.
        let profile_dir = profile_dir(&out_dir)?;

        let man_dir = profile_dir.join("man");
        std::fs::create_dir_all(&man_dir)?;
        std::fs::write(man_dir.join("paper-age.1"), buffer)?;

        let completion_dir = profile_dir.join("completion");
        std::fs::create_dir_all(&completion_dir)?;
        for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
            generate_to(shell, &mut cmd, "paper-age", &completion_dir)?;
        }

        // Re-run if the cli or page files change
        println!("cargo:rerun-if-changed=src/cli.rs");
        println!("cargo:rerun-if-changed=src/page.rs");
    }

    Ok(())
}

/// Finds the profile directory (e.g. `target/release`) from `OUT_DIR`.
///
/// `OUT_DIR` is `<profile>/build/paper-age-<hash>/out` on older Cargo versions and
/// `<profile>/build/paper-age/<hash>/out` since Cargo 1.100, so rather than going up
/// a fixed number of levels, look for the `build` directory and take its parent.
#[cfg(feature = "cli")]
fn profile_dir(out_dir: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    out_dir
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|name| name == "build"))
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "could not find the profile directory from OUT_DIR: {}",
                out_dir.display()
            ))
        })
}
