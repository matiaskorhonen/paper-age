use std::{
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Cursor, Read, Write},
    path::PathBuf,
};

use age::armor::ArmoredWriter;
use age::armor::Format::AsciiArmor;
use age::cli_common::read_secret;
use age::secrecy::Secret;
use clap::Parser;
use printpdf::{
    Color, IndirectFontRef, Line, LineDashPattern, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfPageIndex, Point, Pt, Rgb, Svg, SvgTransform,
};
use qrcode::{render::svg, types::QrError, EcLevel, QrCode};

mod cli;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

fn encrypt_plaintext(
    reader: &mut BufReader<Box<dyn Read>>,
    passphrase: Secret<String>,
) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let mut plaintext: Vec<u8> = vec![];
    reader.read_to_end(&mut plaintext)?;

    let encryptor = age::Encryptor::with_user_passphrase(passphrase);

    let mut encrypted = vec![];

    let armored_writer = ArmoredWriter::wrap_output(&mut encrypted, AsciiArmor)?;

    let mut writer = encryptor.wrap_output(armored_writer)?;

    writer.write_all(&plaintext)?;

    let output = writer.finish().and_then(|armor| armor.finish())?;

    let utf8 = std::string::String::from_utf8(output.to_owned())?;

    Ok((plaintext.len(), utf8))
}

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

#[derive(Clone, Copy)]
struct PageDimensions {
    width: Mm,
    height: Mm,
    margin: Mm,
}

const A4_PAGE: PageDimensions = PageDimensions {
    width: Mm(210.0),
    height: Mm(297.0),
    margin: Mm(10.0),
};

impl Default for PageDimensions {
    fn default() -> Self {
        A4_PAGE
    }
}

struct Document {
    doc: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    title_font: IndirectFontRef,
    code_font: IndirectFontRef,
    dimensions: PageDimensions,
}

impl Document {
    pub fn new(title: String) -> Result<Document, Box<dyn std::error::Error>> {
        let dimensions = PageDimensions::default();

        let (mut doc, page, layer) =
            PdfDocument::new(title, dimensions.width, dimensions.height, "Layer 1");

        let producer = format!("Paper Rage v{}", VERSION.unwrap_or("0.0.0"));
        doc = doc.with_producer(producer);

        let code_data = include_bytes!("assets/fonts/IBMPlexMono-Regular.ttf");
        let code_font = doc.add_external_font(BufReader::new(Cursor::new(code_data)))?;

        let title_data = include_bytes!("assets/fonts/IBMPlexMono-Medium.ttf");
        let title_font = doc.add_external_font(BufReader::new(Cursor::new(title_data)))?;

        Ok(Document {
            doc,
            page,
            layer,
            title_font,
            code_font,
            dimensions,
        })
    }

    fn insert_title_text(&self, title: String) {
        let current_layer = self.doc.get_page(self.page).get_layer(self.layer);

        let font_size = 14.0;

        // Align the title with the QR code if the title is narrower than the QR code
        let margin = {
            if title.len() <= 37 {
                Mm(50.0)
            } else {
                self.dimensions.margin
            }
        };

        current_layer.use_text(
            title,
            font_size,
            margin,
            self.dimensions.height - self.dimensions.margin - Mm::from(Pt(font_size)),
            &self.title_font,
        );
    }

    fn insert_pem_text(&self, pem: String) {
        let current_layer = self.doc.get_page(self.page).get_layer(self.layer);

        let mut font_size = 13.0;
        let mut line_height = 15.0;

        // Rudimentary text scaling to get the Ascii Armor text to fit
        if pem.lines().count() > 39 {
            font_size = 7.0;
            line_height = 8.0;
        } else if pem.lines().count() > 27 {
            font_size = 8.0;
            line_height = 9.0;
        } else if pem.lines().count() > 22 {
            font_size = 10.0;
            line_height = 12.0;
        }

        current_layer.begin_text_section();

        current_layer.set_text_cursor(
            self.dimensions.margin,
            (self.dimensions.height / 2.0) - Mm::from(Pt(font_size)) - self.dimensions.margin,
        );
        current_layer.set_line_height(line_height);
        current_layer.set_font(&self.code_font, font_size);

        for line in pem.lines() {
            current_layer.write_text(line, &self.code_font);
            current_layer.add_line_break();
        }

        current_layer.end_text_section();
    }

    fn insert_qr_code(&self, qrcode: Svg) {
        let current_layer = self.doc.get_page(self.page).get_layer(self.layer);

        let desired_qr_size = Mm(110.0);
        let initial_qr_size = Mm::from(qrcode.height.into_pt(300.0));
        let qr_scale = desired_qr_size / initial_qr_size;

        let scale = qr_scale;
        let dpi = 300.0;
        let code_width = qrcode.width.into_pt(dpi) * scale;
        let code_height = qrcode.height.into_pt(dpi) * scale;

        let translate_x = (self.dimensions.width.into_pt() - code_width) / 2.0;
        let translate_y = self.dimensions.height.into_pt()
            - code_height
            - (self.dimensions.margin.into_pt() * 2.0);

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

    fn draw_grid(&self) {
        let grid_size = Mm(5.0);
        let thickness = 0.0;

        let mut x = Mm(0.0);
        let mut y = self.dimensions.height;
        while x < self.dimensions.width {
            x += grid_size;

            self.draw_divider(
                vec![
                    Point::new(x, self.dimensions.height),
                    Point::new(x, Mm(0.0)),
                ],
                thickness,
                false,
            );

            while y > Mm(0.0) {
                y -= grid_size;

                self.draw_divider(
                    vec![Point::new(self.dimensions.width, y), Point::new(Mm(0.0), y)],
                    thickness,
                    false,
                );
            }
        }
    }

    fn draw_divider(&self, points: Vec<Point>, thickness: f64, dashed: bool) {
        let current_layer = self.doc.get_page(self.page).get_layer(self.layer);

        let mut dash_pattern = LineDashPattern::default();
        if dashed {
            dash_pattern.dash_1 = Some(5);
        } else {
            dash_pattern.dash_1 = None;
        }
        current_layer.set_line_dash_pattern(dash_pattern);

        let outline_color = Color::Rgb(Rgb::new(0.75, 0.75, 0.75, None));
        current_layer.set_outline_color(outline_color);

        current_layer.set_outline_thickness(thickness);

        let divider = Line {
            points: points.iter().map(|p| (*p, false)).collect(),
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };

        current_layer.add_shape(divider);
    }

    fn insert_passphrase(&self) {
        let current_layer = self.doc.get_page(self.page).get_layer(self.layer);

        let baseline = self.dimensions.height / 2.0 + self.dimensions.margin;

        current_layer.use_text("Passphrase: ", 13.0, Mm(50.0), baseline, &self.title_font);

        self.draw_divider(
            vec![
                Point::new(Mm(50.0) + Mm(30.0), baseline - Mm(1.0)),
                Point::new(Mm(110.0) + Mm(50.0), baseline - Mm(1.0)),
            ],
            1.0,
            false,
        )
    }

    fn insert_footer(&self) {
        let current_layer = self.doc.get_page(self.page).get_layer(self.layer);

        current_layer.use_text(
            "Scan QR code and decrypt using Age <https://age-encryption.org>",
            13.0,
            self.dimensions.margin,
            self.dimensions.margin,
            &self.title_font,
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();

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

    let pdf = Document::new(args.title.clone())?;

    if args.grid {
        pdf.draw_grid();
    }

    pdf.insert_title_text(args.title);

    let qrcode = generate_qrcode_svg(encrypted.clone())?;
    pdf.insert_qr_code(qrcode);

    pdf.insert_passphrase();

    pdf.draw_divider(
        vec![
            Point::new(pdf.dimensions.margin, pdf.dimensions.height / 2.0),
            Point::new(
                pdf.dimensions.width - pdf.dimensions.margin,
                pdf.dimensions.height / 2.0,
            ),
        ],
        1.0,
        true,
    );

    pdf.insert_pem_text(encrypted);

    pdf.insert_footer();

    let file = File::create(args.output)?;
    pdf.doc.save(&mut BufWriter::new(file))?;

    Ok(())
}
