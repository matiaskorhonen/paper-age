use std::{
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use age::cli_common::read_secret;
use clap::Parser;
use printpdf::{LineDashPattern, Point};

use crate::paper_age::encryption::encrypt_plaintext;

mod paper_age;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = paper_age::cli::Args::parse();

    if args.fonts_license {
        let license = include_bytes!("assets/fonts/IBMPlexMono-LICENSE.txt");
        io::stdout().write_all(license)?;
        return Ok(());
    }

    let passphrase = match read_secret("Type passphrase", "Passphrase", None) {
        Ok(s) => s,
        Err(_e) => std::process::exit(exitcode::NOINPUT),
    };

    let file = args.input.unwrap();

    let mut reader: BufReader<Box<dyn Read>> = {
        if file == PathBuf::from("-") {
            BufReader::new(Box::new(stdin().lock()))
        } else {
            BufReader::new(Box::new(File::open(&file).unwrap()))
        }
    };

    // Encrypt the plaintext to a ciphertext using the passphrase...
    let (plaintext_len, encrypted) = encrypt_plaintext(&mut reader, passphrase)?;

    println!("Plaintext length: {plaintext_len:?} bytes");
    println!("Encrypted length: {:?} bytes", encrypted.len());

    let pdf = paper_age::Document::new(args.title.clone())?;

    if args.grid {
        pdf.draw_grid();
    }

    pdf.insert_title_text(args.title);

    pdf.insert_qr_code(encrypted.clone())?;

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

    let file = File::create(args.output)?;
    pdf.doc.save(&mut BufWriter::new(file))?;

    Ok(())
}
