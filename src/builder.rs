//! PaperAge
use std::io::{BufReader, Cursor};

use printpdf::{
    Color, IndirectFontRef, Line, LineDashPattern, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfLayerReference, PdfPageIndex, Point, Pt, Rgb, Svg, SvgTransform,
};

use crate::page::*;

pub mod svg;

/// PaperAge version
pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

/// Font width / height = 3 / 5
pub const FONT_RATIO: f64 = 3.0 / 5.0;

/// Container for all the data required to insert elements into the PDF
pub struct Document {
    /// A reference to the printpdf PDF document
    pub doc: PdfDocumentReference,

    /// Index of the first page in the PDF document
    pub page: PdfPageIndex,

    /// Index of the initial layer in the PDF document
    pub layer: PdfLayerIndex,

    /// Reference to the medium weight font
    pub title_font: IndirectFontRef,

    /// Reference to the regular weight font
    pub code_font: IndirectFontRef,

    /// Page size
    pub page_size: PageSize,
}

impl Document {
    /// Initialize the PDF with default dimensions and the required fonts. Also
    /// sets the title and the producer in the PDF metadata.
    pub fn new(title: String, page_size: PageSize) -> Result<Document, Box<dyn std::error::Error>> {
        debug!("Initializing PDF");

        let dimensions = page_size.dimensions();

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
            page_size,
        })
    }

    /// Get the default layer from the PDF
    fn get_current_layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.page).get_layer(self.layer)
    }

    /// Calculate the left margin when text is centered
    fn calc_center_margin(&self, font_size: f64, text_length: usize) -> Mm {
        let text_width = font_size * FONT_RATIO * text_length as f64;
        (self.page_size.dimensions().width - Mm::from(Pt(text_width))) / 2.0
    }

    /// Insert the given title at the top of the PDF
    pub fn insert_title_text(&self, title: String, center: bool) {
        debug!("Inserting title: {}", title.as_str());

        let current_layer = self.get_current_layer();

        let font_size = 14.0;

        // Align the title with the QR code if the title is narrower than the QR code
        let left_margin = {
            if center {
                self.calc_center_margin(font_size, title.len())
            } else if title.len() <= 37 {
                self.page_size.qrcode_left_edge()
            } else {
                self.page_size.dimensions().margin
            }
        };

        current_layer.use_text(
            title,
            font_size,
            left_margin,
            self.page_size.dimensions().height
                - self.page_size.dimensions().margin
                - Mm::from(Pt(font_size)),
            &self.title_font,
        );
    }

    /// Insert the given PEM ciphertext in the bottom half of the page
    pub fn insert_pem_text(&self, pem: String, center: bool) {
        debug!("Inserting PEM encoded ciphertext");

        let current_layer = self.get_current_layer();

        let mut font_size = 13.0;
        let mut line_height = 15.0;

        // Rudimentary text scaling to get the Ascii Armor text to fit
        if pem.lines().count() > 42 {
            font_size = 6.5;
            line_height = 7.0;
        } else if pem.lines().count() > 39 {
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

        let left_margin = if center {
            self.calc_center_margin(font_size, 64)
        } else {
            self.page_size.dimensions().margin
        };

        current_layer.set_text_cursor(
            left_margin,
            (self.page_size.dimensions().height / 2.0)
                - Mm::from(Pt(font_size))
                - self.page_size.dimensions().margin,
        );
        current_layer.set_line_height(line_height);
        current_layer.set_font(&self.code_font, font_size);

        for line in pem.lines() {
            current_layer.write_text(line, &self.code_font);
            current_layer.add_line_break();
        }

        current_layer.end_text_section();
    }

    /// Insert the QR code of the PEM encoded ciphertext in the top half of the page
    pub fn insert_qr_code(&self, text: String) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Inserting QR code");

        let image = svg::qrcode(text)?;
        let qrcode = Svg::parse(image.as_str())?;

        let current_layer = self.get_current_layer();

        let desired_qr_size = self.page_size.qrcode_size();
        let initial_qr_size = Mm::from(qrcode.height.into_pt(300.0));
        let qr_scale = desired_qr_size / initial_qr_size;

        let scale = qr_scale;
        let dpi = 300.0;
        let code_width = qrcode.width.into_pt(dpi) * scale;
        let code_height = qrcode.height.into_pt(dpi) * scale;

        let translate_x = (self.page_size.dimensions().width.into_pt() - code_width) / 2.0;
        let translate_y = self.page_size.dimensions().height.into_pt()
            - code_height
            - (self.page_size.dimensions().margin.into_pt() * 2.0);

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

        Ok(())
    }

    /// Draw a grid debugging layout issues
    pub fn draw_grid(&self) {
        debug!("Drawing grid");

        let grid_size = Mm(5.0);
        let thickness = 0.0;

        let mut x = Mm(0.0);
        let mut y = self.page_size.dimensions().height;
        while x < self.page_size.dimensions().width {
            x += grid_size;

            self.draw_line(
                vec![
                    Point::new(x, self.page_size.dimensions().height),
                    Point::new(x, Mm(0.0)),
                ],
                thickness,
                LineDashPattern::default(),
            );

            while y > Mm(0.0) {
                y -= grid_size;

                self.draw_line(
                    vec![
                        Point::new(self.page_size.dimensions().width, y),
                        Point::new(Mm(0.0), y),
                    ],
                    thickness,
                    LineDashPattern::default(),
                );
            }
        }
    }

    /// Draw a line on the page
    pub fn draw_line(&self, points: Vec<Point>, thickness: f64, dash_pattern: LineDashPattern) {
        trace!("Drawing line");

        let current_layer = self.get_current_layer();

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

    /// Insert the passphrase label and placeholder in the PDF
    pub fn insert_passphrase(&self) {
        debug!("Inserting passphrase placeholder");

        let current_layer = self.get_current_layer();

        let baseline =
            self.page_size.dimensions().height / 2.0 + self.page_size.dimensions().margin;

        current_layer.use_text(
            "Passphrase: ",
            13.0,
            self.page_size.qrcode_left_edge(),
            baseline,
            &self.title_font,
        );

        self.draw_line(
            vec![
                Point::new(
                    self.page_size.qrcode_left_edge() + Mm(30.0),
                    baseline - Mm(1.0),
                ),
                Point::new(
                    self.page_size.qrcode_left_edge() + self.page_size.qrcode_size(),
                    baseline - Mm(1.0),
                ),
            ],
            1.0,
            LineDashPattern::default(),
        )
    }

    /// Add the footer at the bottom of the page
    pub fn insert_footer(&self, center: bool) {
        debug!("Inserting footer");

        let current_layer = self.get_current_layer();
        let text = "Scan QR code and decrypt using Age <https://age-encryption.org>";
        let font_size = 13.0;

        let left_margin = if center {
            self.calc_center_margin(font_size, text.len())
        } else {
            self.page_size.dimensions().margin
        };

        current_layer.use_text(
            text,
            font_size,
            left_margin,
            self.page_size.dimensions().margin,
            &self.title_font,
        );
    }
}

#[test]
fn test_paper_dimensions_default() {
    let default = PageDimensions::default();
    assert_eq!(default.width, Mm(210.0));
    assert_eq!(default.height, Mm(297.0));
}

#[test]
fn test_new_document() {
    let title = String::from("Hello World!");
    let result = Document::new(title, PageSize::A4);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.page_size.dimensions(), crate::page::A4_PAGE);
}

#[test]
fn test_new_letter_document() {
    let title = String::from("Hello Letter!");
    let result = Document::new(title, PageSize::Letter);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.page_size.dimensions(), crate::page::LETTER_PAGE);
}

#[test]
fn test_qrcode() {
    let result = Document::new(String::from("QR code"), PageSize::A4);
    let document = result.unwrap();
    let result = document.insert_qr_code(String::from("payload"));
    assert!(result.is_ok());
}

#[test]
fn test_qrcode_too_large() {
    let document = Document::new(String::from("QR code"), PageSize::A4).unwrap();
    let result = document.insert_qr_code(String::from(include_str!("../tests/data/too_large.txt")));

    assert!(result.is_err());
    assert!(result.unwrap_err().is::<qrcode::types::QrError>());
}
