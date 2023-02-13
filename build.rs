use clap::CommandFactory;
use clap_complete::{generate_to, shells::Shell};

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

    // Create a man directory the same level as the binary, even though build scripts shouldn't
    let man_dir = out_dir.join("../../../man");
    let man_path = std::fs::canonicalize(man_dir.join("paper-age.1"))?;
    std::fs::create_dir_all(man_dir.clone())?;
    std::fs::write(man_path.clone(), buffer)?;
    println!("cargo:warning=generated man page: {man_path:?}");

    // Create a completion directory the same level as the binary
    let completion_dir = std::fs::canonicalize(out_dir.join("../../../completion"))?;
    std::fs::create_dir_all(completion_dir.clone())?;
    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        let path = generate_to(shell, &mut cmd, "paper-age", completion_dir.clone())?;
        println!("cargo:warning=generated {shell:?} completion file: {path:?}",);
    }

    // Re-run if the cli or page files change
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=src/page.rs");

    Ok(())
}
