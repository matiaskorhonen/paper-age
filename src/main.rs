use std::fs::File;
use std::io;
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
use qrcode::EcLevel;
use qrcode::QrCode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Plaintext to encrypt (max. 640 characters)
    #[arg(
        short = 't',
        long,
        default_value = "",
        required_unless_present = "fonts_license"
    )]
    plaintext: String,

    /// Passphrase
    #[arg(
        short,
        long,
        default_value = "",
        requires = "plaintext",
        required_unless_present = "fonts_license"
    )]
    passphrase: String,

    /// Page title (max. 60 characters)
    #[arg(long, default_value = "Paper Rage", requires = "plaintext")]
    title: String,

    // Output file name
    #[arg(short, long, default_value = "out.pdf", requires = "plaintext")]
    output: String,

    // Print out the license for the embedded fonts
    #[arg(long, default_value_t = false, exclusive = true)]
    fonts_license: bool,
}

fn encrypt_plaintext(
    plaintext: String,
    passphrase: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let plaintext = plaintext.as_bytes();
    let passphrase = passphrase.as_str();

    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

    let mut encrypted = vec![];

    let armored_writer = ArmoredWriter::wrap_output(&mut encrypted, AsciiArmor)?;

    let mut writer = encryptor.wrap_output(armored_writer)?;

    writer.write_all(plaintext)?;

    let output = writer.finish().and_then(|armor| armor.finish())?;

    let utf8 = std::string::String::from_utf8(output.to_owned())?;

    return Ok(utf8);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.fonts_license {
        let license = include_bytes!("assets/fonts/license.txt");
        io::stdout().write_all(license)?;
        return Ok(());
    }

    // Encrypt the plaintext to a ciphertext using the passphrase...
    let encrypted = encrypt_plaintext(args.plaintext, args.passphrase)?;
    io::stdout().write_all(encrypted.as_bytes())?;

    let code = QrCode::with_error_correction_level(encrypted.clone(), EcLevel::H).unwrap();

    let image = code
        .render()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    let qrcode = Svg::parse(image.as_str())?;

    let width = Mm(210.0);
    let height = Mm(297.0);
    let margin = Mm(15.0);

    let desired_qr_size = (height / 2.0) - margin * 3.0;
    let initial_qr_size = Mm::from(qrcode.height.into_pt(300.0));
    let qr_scale = desired_qr_size / initial_qr_size;

    println!(
        "QR code size: {:?}x{:?} (scale: {:?})",
        qrcode.width, qrcode.height, qr_scale
    );

    let (doc, page, layer) = PdfDocument::new("Paper Rage", width, height, "Layer 1");

    let code_data = include_bytes!("assets/fonts/IBMPlexMono-Regular.ttf");
    let code_font = doc.add_external_font(BufReader::new(Cursor::new(code_data)))?;

    let title_data = include_bytes!("assets/fonts/IBMPlexMono-Medium.ttf");
    let title_font = doc.add_external_font(BufReader::new(Cursor::new(title_data)))?;

    let current_layer = doc.get_page(page).get_layer(layer);

    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(5);
    let outline_color = Color::Rgb(Rgb::new(0.75, 0.75, 0.75, None));
    current_layer.set_outline_color(outline_color);
    current_layer.set_line_dash_pattern(dash_pattern);

    let divider = Line {
        points: vec![
            (Point::new(Mm(0.0), height / 2.0), false),
            (Point::new(width, height / 2.0), false),
        ],
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    };

    current_layer.add_shape(divider);

    current_layer.use_text(args.title, 14.0, margin, height - margin, &title_font);

    let font_size = 13.0;
    let line_height = font_size * 1.2;

    current_layer.begin_text_section();

    current_layer.set_text_cursor(margin, (height / 2.0) - Mm::from(Pt(font_size)) - margin);
    current_layer.set_line_height(line_height);
    current_layer.set_font(&code_font, font_size);

    for line in encrypted.lines() {
        current_layer.write_text(line.clone(), &code_font);
        current_layer.add_line_break();
    }

    current_layer.end_text_section();

    let scale = qr_scale;
    let dpi = 300.0;
    let code_width = qrcode.width.into_pt(dpi) * scale;
    let code_height = qrcode.height.into_pt(dpi) * scale;

    let translate_x = (width.into_pt() - code_width) / 2.0;
    let translate_y = height.into_pt() - code_height - (margin.into_pt() * 2.0);

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

    let file = File::create(args.output)?;
    doc.save(&mut BufWriter::new(file))?;

    Ok(())
}
