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

    // Write to the same level as the binary, even though build scripts shouldn't
    std::fs::write(out_dir.join("../../../paper-age.1"), buffer)?;

    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        let path = generate_to(shell, &mut cmd, "paper-age", out_dir.join("../../../"))?;
        println!("cargo:info=completion file is generated: {:?}", path);
    }

    Ok(())
}
