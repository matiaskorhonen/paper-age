use clap::CommandFactory;

#[path = "src/cli.rs"]
mod cli;

#[path = "src/page.rs"]
pub mod page;

fn main() -> std::io::Result<()> {
    let out_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);
    let cmd = cli::Args::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    // Write to the same level as the binary, even though build scripts shouldn't
    std::fs::write(out_dir.join("../../../paper-age.1"), buffer)?;

    Ok(())
}
