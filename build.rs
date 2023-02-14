use clap::CommandFactory;
use clap_complete::{generate_to, shells::Shell};
use path_absolutize::*;

#[path = "src/cli.rs"]
mod cli;

#[path = "src/page.rs"]
pub mod page;

fn main() -> std::io::Result<()> {
    let out_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);
    let mut cmd = cli::Args::command();

    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    // Create a man directory at the same level as the binary, even though build
    // scripts shouldn't
    let man_dir = &out_dir.join("../../../man");
    let absolute_man_dir = man_dir.absolutize()?;
    std::fs::create_dir_all(&absolute_man_dir)?;
    let man_path = absolute_man_dir.join("paper-age.1");
    std::fs::write(man_path, buffer)?;

    // Create a completion directory the same level as the binary
    let completion_dir = out_dir.join("../../../completion");
    let absolute_completion_dir = completion_dir.absolutize()?;
    std::fs::create_dir_all(absolute_completion_dir.clone())?;
    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        generate_to(
            shell,
            &mut cmd,
            "paper-age",
            absolute_completion_dir.as_ref(),
        )?;
    }

    // Re-run if the cli or page files change
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=src/page.rs");

    Ok(())
}
