use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Write};

use age::armor::ArmoredWriter;
use age::armor::Format::AsciiArmor;
use age::secrecy::Secret;
use clap::Parser;
use printpdf::{
    Color, IndirectFontRef, Line, LineDashPattern, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfLayerReference, PdfPageIndex, Point, Pt, Rgb, Svg, SvgTransform,
};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

fn generate_qrcode_svg(text: String) -> Result<Svg, Box<dyn std::error::Error>> {
    let code = QrCode::with_error_correction_level(text, EcLevel::H)?;

    let image = code
        .render()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    let svg = Svg::parse(image.as_str())?;

    Ok(svg)
}

#[derive(Clone, Copy)]
struct PageDimensions {
    width: Mm,
    height: Mm,
    margin: Mm,
}

struct Pdf {
    doc: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    title_font: IndirectFontRef,
    code_font: IndirectFontRef,
}

fn initialize_pdf(
    dimensions: PageDimensions,
    title: String,
) -> Result<Pdf, Box<dyn std::error::Error>> {
    let (doc, page, layer) =
        PdfDocument::new(title, dimensions.width, dimensions.height, "Layer 1");

    let code_data = include_bytes!("assets/fonts/IBMPlexMono-Regular.ttf");
    let code_font = doc.add_external_font(BufReader::new(Cursor::new(code_data)))?;

    let title_data = include_bytes!("assets/fonts/IBMPlexMono-Medium.ttf");
    let title_font = doc.add_external_font(BufReader::new(Cursor::new(title_data)))?;

    Ok(Pdf {
        doc: doc,
        page: page,
        layer: layer,
        title_font: title_font,
        code_font: code_font,
    })
}

fn draw_divider(current_layer: &PdfLayerReference, points: Vec<Point>) {
    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(5);
    let outline_color = Color::Rgb(Rgb::new(0.75, 0.75, 0.75, None));
    current_layer.set_outline_color(outline_color);
    current_layer.set_line_dash_pattern(dash_pattern);

    let divider = Line {
        points: points.iter().map(|p| (*p, false)).collect(),
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    };

    current_layer.add_shape(divider);
}

fn draw_grid(current_layer: &PdfLayerReference, dimensions: PageDimensions) {
    let grid_size = Mm(10.0);

    let mut x = Mm(0.0);
    let mut y = Mm(0.0);
    while x < dimensions.width {
        x = x + grid_size;

        draw_divider(
            current_layer,
            vec![Point::new(x, dimensions.height), Point::new(x, Mm(0.0))],
        );

        while y < dimensions.height {
            y = y + grid_size;

            draw_divider(
                current_layer,
                vec![Point::new(dimensions.width, y), Point::new(Mm(0.0), y)],
            );
        }
    }
}

fn insert_qr_code(current_layer: &PdfLayerReference, qrcode: Svg, dimensions: PageDimensions) {
    let desired_qr_size = (dimensions.height / 2.0) - dimensions.margin * 3.0;
    let initial_qr_size = Mm::from(qrcode.height.into_pt(300.0));
    let qr_scale = desired_qr_size / initial_qr_size;

    let scale = qr_scale;
    let dpi = 300.0;
    let code_width = qrcode.width.into_pt(dpi) * scale;
    let code_height = qrcode.height.into_pt(dpi) * scale;

    let translate_x = (dimensions.width.into_pt() - code_width) / 2.0;
    let translate_y =
        dimensions.height.into_pt() - code_height - (dimensions.margin.into_pt() * 2.0);

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
}

fn insert_title_text(
    title: String,
    pdf: &Pdf,
    current_layer: &PdfLayerReference,
    dimensions: PageDimensions,
) {
    current_layer.use_text(
        title,
        14.0,
        dimensions.margin,
        dimensions.height - dimensions.margin,
        &pdf.title_font,
    );
}

fn insert_pem_text(
    pdf: &Pdf,
    current_layer: &PdfLayerReference,
    pem: String,
    dimensions: PageDimensions,
) {
    let font_size = 13.0;
    let line_height = font_size * 1.2;

    current_layer.begin_text_section();

    current_layer.set_text_cursor(
        dimensions.margin,
        (dimensions.height / 2.0) - Mm::from(Pt(font_size)) - dimensions.margin,
    );
    current_layer.set_line_height(line_height);
    current_layer.set_font(&pdf.code_font, font_size);

    for line in pem.lines() {
        current_layer.write_text(line, &pdf.code_font);
        current_layer.add_line_break();
    }

    current_layer.end_text_section();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if args.fonts_license {
        let license = include_bytes!("assets/fonts/license.txt");
        io::stdout().write_all(license)?;
        return Ok(());
    }

    // Encrypt the plaintext to a ciphertext using the passphrase...
    let encrypted = encrypt_plaintext(args.plaintext, args.passphrase)?;
    io::stdout().write_all(encrypted.as_bytes())?;

    let a4 = PageDimensions {
        width: Mm(210.0),
        height: Mm(297.0),
        margin: Mm(15.0),
    };

    let pdf = initialize_pdf(a4, args.title.clone())?;
    let current_layer = pdf.doc.get_page(pdf.page).get_layer(pdf.layer);

    draw_grid(&current_layer, a4);

    insert_title_text(args.title, &pdf, &current_layer, a4);

    let qrcode = generate_qrcode_svg(encrypted.clone())?;
    insert_qr_code(&current_layer, qrcode, a4);

    draw_divider(
        &current_layer,
        vec![
            Point::new(Mm(0.0), a4.height / 2.0),
            Point::new(a4.width, a4.height / 2.0),
        ],
    );

    insert_pem_text(&pdf, &current_layer, encrypted, a4);

    let file = File::create(args.output)?;
    pdf.doc.save(&mut BufWriter::new(file))?;

    Ok(())
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
