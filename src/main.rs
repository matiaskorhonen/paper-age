#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://user-images.githubusercontent.com/43314/216838549-bc5cafc8-0211-44e2-9bcc-651c74bfc853.svg"
)]
#![doc(html_favicon_url = "https://shots.matiaskorhonen.fi/paper-age-favicon.ico")]

use std::{
    env,
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use age::secrecy::{ExposeSecret, SecretString};
use clap::Parser;
use qrcode::types::QrError;
use rpassword::prompt_password;

pub mod builder;
pub mod cli;
pub mod encryption;
pub mod page;

#[macro_use]
extern crate log;

/// Maximum length of the document title
const TITLE_MAX_LEN: usize = 64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if args.fonts_license {
        let license = include_bytes!("assets/fonts/IBMPlexMono-LICENSE.txt");
        io::stdout().write_all(license)?;
        return Ok(());
    }

    if args.title.len() > TITLE_MAX_LEN {
        error!(
            "The title cannot be longer than {} characters",
            TITLE_MAX_LEN
        );
        std::process::exit(exitcode::DATAERR);
    }

    let output = args.output;
    if output.exists() {
        if args.force {
            warn!("Overwriting existing output file: {}", output.display());
        } else {
            error!("Output file already exists: {}", output.display());
            std::process::exit(exitcode::CANTCREAT);
        }
    }

    let path = match args.input {
        Some(p) => p,
        None => PathBuf::from("-"),
    };
    let mut reader: BufReader<Box<dyn Read>> = {
        if path.to_string_lossy() == "-" {
            BufReader::new(Box::new(stdin().lock()))
        } else if path.is_file() {
            let size = path.metadata()?.len();
            if size >= 2048 {
                warn!("File too large ({size:?} bytes). The maximum file size is about 1.9 KiB.");
            }
            BufReader::new(Box::new(File::open(&path).unwrap()))
        } else {
            error!("File not found: {}", path.display());
            std::process::exit(exitcode::NOINPUT);
        }
    };

    let passphrase = get_passphrase()?;

    // Encrypt the plaintext to a ciphertext using the passphrase...
    let (plaintext_len, encrypted) = encryption::encrypt_plaintext(&mut reader, passphrase)?;

    info!("Plaintext length: {plaintext_len:?} bytes");
    info!("Encrypted length: {:?} bytes", encrypted.len());

    let pdf = builder::Document::new(args.title, args.page_size)?;

    let bytes = match pdf.create_pdf(args.grid, args.notes_label, args.skip_notes_line, encrypted) {
        Ok(b) => b,
        Err(error) => {
            if error.is::<QrError>() {
                error!("Too much data after encryption, please try a smaller file");
                std::process::exit(exitcode::DATAERR);
            } else {
                error!("The QR code generation failed for an unknown reason");
                std::process::exit(exitcode::SOFTWARE);
            }
        }
    };

    if output.to_string_lossy() == "-" {
        debug!("Writing to STDOUT");
        io::stdout().write_all(&bytes)?;
    } else {
        debug!("Writing to file: {}", output.to_string_lossy());
        let file = File::create(output)?;
        BufWriter::new(file).write_all(&bytes)?;
    }

    Ok(())
}

/// Read a secret from the user
pub fn read_secret(prompt: &str) -> Result<SecretString, io::Error> {
    let passphrase = prompt_password(format!("{}: ", prompt)).map(SecretString::from)?;

    if passphrase.expose_secret().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Passphrase can't be empty",
        ));
    }

    Ok(passphrase)
}

/// Get the passphrase from an interactive prompt or from the PAPERAGE_PASSPHRASE
/// environment variable
fn get_passphrase() -> Result<SecretString, io::Error> {
    let env_passphrase = env::var("PAPERAGE_PASSPHRASE");

    if let Ok(value) = env_passphrase {
        return Ok(SecretString::from(value));
    }

    match read_secret("Passphrase") {
        Ok(secret) => Ok(secret),
        Err(e) => Err(io::Error::other(format!("{e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::ExposeSecret;

    #[test]
    fn test_get_passphrase_from_env() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("PAPERAGE_PASSPHRASE", "secret");

        let result = get_passphrase();
        assert!(result.is_ok());

        let passphrase = result?;
        passphrase.expose_secret();

        assert_eq!(passphrase.expose_secret(), "secret");

        Ok(())
    }
}
