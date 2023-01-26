use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Cursor;
use std::io::Write;

use age::armor::ArmoredWriter;
use age::armor::Format::AsciiArmor;
use age::secrecy::Secret;
use clap::Parser;
use printpdf::*;
use qrcode::render::svg;
use qrcode::QrCode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Plaintext to encrypt
    #[arg(short = 't', long)]
    plaintext: String,

    /// Passphrase
    #[arg(short, long)]
    passphrase: String,

    /// Page title
    #[arg(long, default_value = "Paper Rage")]
    title: String,

    // Output file name
    #[arg(short, long, default_value = "out.pdf")]
    output: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let plaintext = args.plaintext.as_bytes();
    let passphrase = args.passphrase.as_str();

    // Encrypt the plaintext to a ciphertext using the passphrase...
    let encrypted: String = {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

        let mut encrypted = vec![];

        let armored_writer = match ArmoredWriter::wrap_output(&mut encrypted, AsciiArmor) {
            Ok(w) => w,
            Err(error) => panic!("Error: {:?}", error),
        };

        let mut writer = match encryptor.wrap_output(armored_writer) {
            Ok(w) => w,
            Err(error) => panic!("Error: {:?}", error),
        };

        match writer.write_all(plaintext) {
            Ok(()) => (),
            Err(error) => panic!("Error: {:?}", error),
        }

        let output = match writer.finish().and_then(|armor| armor.finish()) {
            Ok(e) => e.to_owned(),
            Err(error) => panic!("Error: {:?}", error),
        };

        match std::string::String::from_utf8(output) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    };

    println!("{}", encrypted);

    let code = QrCode::new(encrypted.clone()).unwrap();

    let image = code
        .render()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    let qrcode = match Svg::parse(image.as_str()) {
        Ok(qr) => qr,
        Err(error) => panic!("Error: {:?}", error),
    };

    let width = Mm(210.0);
    let height = Mm(297.0);
    let margin = Mm(15.0);

    let (doc, page, layer) = PdfDocument::new("Paper Rage", width, height, "Layer 1");

    let mono_data = include_bytes!("assets/fonts/IBMPlexMono-Regular.ttf");
    let mono_font = doc
        .add_external_font(BufReader::new(Cursor::new(mono_data)))
        .unwrap();

    let sans_data = include_bytes!("assets/fonts/IBMPlexSans-Medium.ttf");
    let sans_font = doc
        .add_external_font(BufReader::new(Cursor::new(sans_data)))
        .unwrap();

    let current_layer = doc.get_page(page).get_layer(layer);

    current_layer.use_text(args.title, 24.0, margin, height / 2.0, &sans_font);

    current_layer.begin_text_section();

    let font_size = 12.0;
    let line_height = Mm(font_size);
    let line_count = encrypted.lines().count();
    for (i, line) in encrypted.lines().enumerate() {
        let offset = line_height / 2.0 * ((line_count - i) as f64);
        current_layer.use_text(line, font_size, margin, offset + margin, &mono_font);
        current_layer.add_line_break();
    }

    current_layer.end_text_section();

    let scale = 4.0;
    let dpi = 300.0;
    let code_width = qrcode.width.into_pt(dpi) * scale;
    let code_height = qrcode.height.into_pt(dpi) * scale;

    let translate_x = (width.into_pt() - code_width) / 2.0;
    let translate_y = height.into_pt() - code_height - margin.into_pt();

    qrcode.add_to_layer(
        &current_layer,
        SvgTransform {
            translate_x: Some(translate_x),
            translate_y: Some(translate_y),
            rotate: None,
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(dpi),
        },
    );

    doc.save(&mut BufWriter::new(File::create(args.output).unwrap()))
        .unwrap();

    Ok(())
}
