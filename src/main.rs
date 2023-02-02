use std::{
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use age::cli_common::read_secret;
use clap::Parser;
use printpdf::{LineDashPattern, Point, Svg};
use qrcode::{render::svg, types::QrError, EcLevel, QrCode};

use crate::paper_rage::encryption::encrypt_plaintext;

mod paper_rage;

fn generate_qrcode_svg(text: String) -> Result<Svg, Box<dyn std::error::Error>> {
    // QR Code Error Correction Capability (approx.)
    //     H: 30%
    //     Q: 25%
    //     M: 15%
    //     L: 7%
    let levels = [EcLevel::H, EcLevel::Q, EcLevel::M, EcLevel::L];

    // Find the best level of EC level possible for the data
    let mut result: Result<QrCode, QrError> = Result::Err(QrError::DataTooLong);
    for ec_level in levels.iter() {
        result = QrCode::with_error_correction_level(text.clone(), *ec_level);

        if result.is_ok() {
            break;
        }
    }
    let code = result?;

    println!(
        "QR code EC level: {:?}, Version: {:?}",
        code.error_correction_level(),
        code.version()
    );

    let image = code
        .render()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .quiet_zone(false)
        .build();

    let svg = Svg::parse(image.as_str())?;

    Ok(svg)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = paper_rage::cli::Args::parse();

    if args.fonts_license {
        let license = include_bytes!("assets/fonts/license.txt");
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

    let pdf = paper_rage::Document::new(args.title.clone())?;

    if args.grid {
        pdf.draw_grid();
    }

    pdf.insert_title_text(args.title);

    let qrcode = generate_qrcode_svg(encrypted.clone())?;
    pdf.insert_qr_code(qrcode);

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
