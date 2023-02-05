#![doc = include_str!("../README.md")]

use std::{
    env,
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use age::{
    cli_common::read_secret,
    secrecy::{Secret, SecretString},
};
use clap::Parser;
use printpdf::{LineDashPattern, Point};
use qrcode::types::QrError;

#[macro_use]
extern crate log;

use crate::paper_age::encryption::encrypt_plaintext;

mod paper_age;

/// Maximum length of the document title
const TITLE_MAX_LEN: usize = 64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = paper_age::cli::Args::parse();

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

    let path = args.input.unwrap();
    let mut reader: BufReader<Box<dyn Read>> = {
        if path == PathBuf::from("-") {
            BufReader::new(Box::new(stdin().lock()))
        } else if path.is_file() {
            BufReader::new(Box::new(File::open(&path).unwrap()))
        } else {
            error!("File not found: {}", path.display());
            std::process::exit(exitcode::NOINPUT);
        }
    };

    let passphrase = get_passphrase()?;

    // Encrypt the plaintext to a ciphertext using the passphrase...
    let (plaintext_len, encrypted) = encrypt_plaintext(&mut reader, passphrase)?;

    info!("Plaintext length: {plaintext_len:?} bytes");
    info!("Encrypted length: {:?} bytes", encrypted.len());

    let pdf = paper_age::Document::new(args.title.clone())?;

    if args.grid {
        pdf.draw_grid();
    }

    pdf.insert_title_text(args.title);

    match pdf.insert_qr_code(encrypted.clone()) {
        Ok(()) => (),
        Err(error) => {
            if error.is::<QrError>() {
                error!("Too much data after encryption, please try a smaller file");
                std::process::exit(exitcode::DATAERR);
            } else {
                error!("The QR code generation failed for an unknown reason");
                std::process::exit(exitcode::SOFTWARE);
            }
        }
    }

    pdf.insert_passphrase();

    pdf.draw_line(
        vec![
            Point::new(pdf.dimensions.margin, pdf.dimensions.height / 2.0),
            Point::new(
                pdf.dimensions.width - pdf.dimensions.margin,
                pdf.dimensions.height / 2.0,
            ),
        ],
        1.0,
        LineDashPattern {
            dash_1: Some(5),
            ..LineDashPattern::default()
        },
    );

    pdf.insert_pem_text(encrypted);

    pdf.insert_footer();

    let file = File::create(output)?;
    pdf.doc.save(&mut BufWriter::new(file))?;

    Ok(())
}

/// Get the passphrase from an interactive prompt or from the PAPERAGE_PASSPHRASE
/// environment variable
fn get_passphrase() -> Result<Secret<String>, io::Error> {
    let env_passphrase = env::var("PAPERAGE_PASSPHRASE");

    if let Ok(value) = env_passphrase {
        return Ok(SecretString::from(value));
    }

    match read_secret("Type passphrase", "Passphrase", None) {
        Ok(secret) => Ok(secret),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{e}"))),
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
