//! PaperAge

use std::io::Write;

use log::{debug, trace};
use printpdf::{
    Color, DateTime, Line, LineDashPattern, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument,
    PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, Rect, Rgb, Svg, TextItem, WindingOrder,
    XObjectTransform,
};

use crate::page::*;

pub mod svg;

/// PaperAge version
pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

/// Font width / height = 3 / 5
const FONT_RATIO: f32 = 3.0 / 5.0;

const CODE_FONT_BYTES: &[u8] = include_bytes!("assets/fonts/IBMPlexMono-Regular.ttf");
const TITLE_FONT_BYTES: &[u8] = include_bytes!("assets/fonts/IBMPlexMono-Medium.ttf");

/// Container for all the data required to insert elements into the PDF
pub struct Document {
    /// The printpdf PDF document
    pub doc: PdfDocument,

    /// Operations to perform on the page
    ops: Vec<Op>,

    /// The medium weight font handle
    pub title_font: PdfFontHandle,

    /// The regular weight font handle
    pub code_font: PdfFontHandle,

    /// Page size
    pub page_size: PageSize,

    /// Document title
    pub title: String,
}

impl Document {
    /// Initialize the PDF with default dimensions and the required fonts. Also
    /// sets the title and the producer in the PDF metadata.
    pub fn new(title: String, page_size: PageSize) -> Result<Document, Box<dyn std::error::Error>> {
        debug!("Initializing PDF");

        let dimensions = page_size.dimensions();

        let mut doc = PdfDocument::new(&title);

        let producer = format!("PaperAge v{}", VERSION.unwrap_or("0.0.0"));
        let now = DateTime::now();
        doc.metadata.info.producer = producer;
        doc.metadata.info.creation_date = now;
        doc.metadata.info.modification_date = now;

        let mut warnings = Vec::new();

        let code_parsed = ParsedFont::from_bytes(CODE_FONT_BYTES, 0, &mut warnings)
            .ok_or("Failed to parse code font")?;
        let code_font_id = doc.add_font(&code_parsed);
        let code_font = PdfFontHandle::External(code_font_id);

        let title_parsed = ParsedFont::from_bytes(TITLE_FONT_BYTES, 0, &mut warnings)
            .ok_or("Failed to parse title font")?;
        let title_font_id = doc.add_font(&title_parsed);
        let title_font = PdfFontHandle::External(title_font_id);

        let ops = vec![
            // White background
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)),
            },
            Op::DrawRectangle {
                rectangle: Rect {
                    x: Pt(0.0),
                    y: Pt(0.0),
                    width: dimensions.width.into_pt(),
                    height: dimensions.height.into_pt(),
                    mode: Some(PaintMode::Fill),
                    winding_order: Some(WindingOrder::NonZero),
                },
            },
            // Reset fill color to black for text and QR code
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
            },
        ];

        Ok(Document {
            doc,
            ops,
            title_font,
            code_font,
            page_size,
            title: title.clone(),
        })
    }

    /// Insert the given title at the top of the PDF
    pub fn insert_title_text(&mut self, title: String) {
        debug!("Inserting title: {}", title.as_str());

        let font_size = 14.0;

        // Align the title with the QR code if the title is narrower than the QR code
        let margin = {
            if title.len() <= 37 {
                self.page_size.qrcode_left_edge()
            } else {
                self.page_size.dimensions().margin
            }
        };

        let y = self.page_size.dimensions().height
            - self.page_size.dimensions().margin
            - Mm::from(Pt(font_size));

        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetTextCursor {
            pos: Point::new(margin, y),
        });
        self.ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        });
        self.ops.push(Op::SetFont {
            font: self.title_font.clone(),
            size: Pt(font_size),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(title)],
        });
        self.ops.push(Op::EndTextSection);
    }

    /// Insert the given PEM ciphertext in the bottom half of the page
    pub fn insert_pem_text(&mut self, pem: String) {
        debug!("Inserting PEM encoded ciphertext");

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

        self.ops.push(Op::StartTextSection);

        self.ops.push(Op::SetTextCursor {
            pos: Point::new(
                self.page_size.dimensions().margin,
                (self.page_size.dimensions().height / 2.0)
                    - Mm::from(Pt(font_size))
                    - self.page_size.dimensions().margin,
            ),
        });
        self.ops.push(Op::SetLineHeight {
            lh: Pt(line_height),
        });
        self.ops.push(Op::SetFont {
            font: self.code_font.clone(),
            size: Pt(font_size),
        });

        for line in pem.lines() {
            self.ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.to_string())],
            });
            self.ops.push(Op::AddLineBreak);
        }

        self.ops.push(Op::EndTextSection);
    }

    /// Insert the QR code of the PEM encoded ciphertext in the top half of the page
    pub fn insert_qr_code(&mut self, text: String) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Inserting QR code");

        let image = svg::qrcode(text)?;

        let (svg_width_px, svg_height_px) =
            parse_svg_pixel_dimensions(&image).ok_or("Failed to parse SVG dimensions")?;

        let mut warnings = Vec::new();
        let xobject = Svg::parse(&image, &mut warnings)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        let xobj_id = self.doc.add_xobject(&xobject);

        let dpi = 300.0;
        let svg_height_pt = Pt(svg_height_px * 72.0 / dpi);
        let svg_width_pt = Pt(svg_width_px * 72.0 / dpi);

        let desired_qr_size = self.page_size.qrcode_size();
        let initial_qr_size = Mm::from(svg_height_pt);
        let qr_scale = desired_qr_size.0 / initial_qr_size.0;

        let scale = qr_scale;
        let code_width = Pt(svg_width_pt.0 * scale);
        let code_height = Pt(svg_height_pt.0 * scale);

        let page_width_pt = self.page_size.dimensions().width.into_pt();
        let page_height_pt = self.page_size.dimensions().height.into_pt();
        let margin_pt = self.page_size.dimensions().margin.into_pt();

        let translate_x = Pt((page_width_pt.0 - code_width.0) / 2.0);
        let translate_y = Pt(page_height_pt.0 - code_height.0 - margin_pt.0 * 2.0);

        self.ops.push(Op::UseXobject {
            id: xobj_id,
            transform: XObjectTransform {
                translate_x: Some(translate_x),
                translate_y: Some(translate_y),
                rotate: None,
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(dpi),
            },
        });

        Ok(())
    }

    /// Draw a grid debugging layout issues
    pub fn draw_grid(&mut self) {
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
    pub fn draw_line(&mut self, points: Vec<Point>, thickness: f32, dash_pattern: LineDashPattern) {
        trace!("Drawing line");

        self.ops.push(Op::SetLineDashPattern { dash: dash_pattern });

        let outline_color = Color::Rgb(Rgb::new(0.75, 0.75, 0.75, None));
        self.ops.push(Op::SetOutlineColor { col: outline_color });

        self.ops.push(Op::SetOutlineThickness { pt: Pt(thickness) });

        let divider = Line {
            points: points
                .iter()
                .map(|p| LinePoint {
                    p: *p,
                    bezier: false,
                })
                .collect(),
            is_closed: false,
        };

        self.ops.push(Op::DrawLine { line: divider });
    }

    /// Insert the notes field label and placeholder in the PDF
    pub fn insert_notes_field(&mut self, label: String, skip_line: bool) {
        debug!("Inserting notes/passphrase placeholder");
        const MAX_LABEL_LEN: usize = 32;

        let baseline =
            self.page_size.dimensions().height / 2.0 + self.page_size.dimensions().margin;

        let label_len = label.len();

        let font_size = 13.0;

        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetTextCursor {
            pos: Point::new(self.page_size.qrcode_left_edge(), baseline),
        });
        self.ops.push(Op::SetFont {
            font: self.title_font.clone(),
            size: Pt(font_size),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(label)],
        });
        self.ops.push(Op::EndTextSection);

        // If the placeholder line would be ridiculously short, don't draw it
        if label_len <= MAX_LABEL_LEN && !skip_line {
            self.draw_line(
                vec![
                    Point::new(
                        self.page_size.qrcode_left_edge()
                            + Mm::from(Pt(FONT_RATIO * font_size * label_len as f32)),
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
    }

    /// Add the footer at the bottom of the page
    pub fn insert_footer(&mut self) {
        debug!("Inserting footer");

        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetTextCursor {
            pos: Point::new(
                self.page_size.dimensions().margin,
                self.page_size.dimensions().margin,
            ),
        });
        self.ops.push(Op::SetFont {
            font: self.title_font.clone(),
            size: Pt(13.0),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(
                "Scan QR code and decrypt using Age <https://age-encryption.org>".to_string(),
            )],
        });
        self.ops.push(Op::EndTextSection);
    }

    /// Build the final PDF and return as bytes
    pub fn save_to_bytes(mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let dimensions = self.page_size.dimensions();
        let page = PdfPage::new(dimensions.width, dimensions.height, self.ops);
        self.doc.pages.push(page);

        let mut warnings = Vec::new();
        let bytes = self.doc.save(&PdfSaveOptions::default(), &mut warnings);
        Ok(bytes)
    }

    /// Build the final PDF and write to a writer
    pub fn save_to_writer<W: Write>(
        mut self,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dimensions = self.page_size.dimensions();
        let page = PdfPage::new(dimensions.width, dimensions.height, self.ops);
        self.doc.pages.push(page);

        let mut warnings = Vec::new();
        self.doc
            .save_writer(writer, &PdfSaveOptions::default(), &mut warnings);
        Ok(())
    }

    /// Build a PaperAge PDF and return its bytes.
    ///
    /// # Arguments
    /// * `grid` - Whether to draw a debug grid
    /// * `notes_label` - Label for the notes/passphrase field
    /// * `skip_notes_line` - Whether to omit the notes placeholder line
    /// * `encrypted` - The encrypted ciphertext to encode as a QR code and PEM block
    pub fn create_pdf(
        mut self,
        grid: bool,
        notes_label: String,
        skip_notes_line: bool,
        encrypted: String,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if grid {
            self.draw_grid();
        }

        self.insert_title_text(self.title.clone());

        self.insert_qr_code(encrypted.clone())?;

        self.insert_notes_field(notes_label, skip_notes_line);

        self.draw_line(
            vec![
                self.page_size.dimensions().center_left(),
                self.page_size.dimensions().center_right(),
            ],
            1.0,
            LineDashPattern {
                dash_1: Some(5),
                ..LineDashPattern::default()
            },
        );

        self.insert_pem_text(encrypted);

        self.insert_footer();

        self.save_to_bytes()
    }
}

/// Parse width and height pixel values from an SVG string
fn parse_svg_pixel_dimensions(svg: &str) -> Option<(f32, f32)> {
    let width_start = svg.find("width=\"")? + 7;
    let width_end = svg[width_start..].find('"')? + width_start;
    let width: f32 = svg[width_start..width_end].parse().ok()?;

    let height_start = svg.find("height=\"")? + 8;
    let height_end = svg[height_start..].find('"')? + height_start;
    let height: f32 = svg[height_start..height_end].parse().ok()?;

    Some((width, height))
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
    let mut document = result.unwrap();
    let result = document.insert_qr_code(String::from("payload"));
    assert!(result.is_ok());
}

#[test]
fn test_qrcode_too_large() {
    let mut document = Document::new(String::from("QR code"), PageSize::A4).unwrap();
    let result = document.insert_qr_code(String::from(include_str!("../tests/data/too_large.txt")));

    assert!(result.is_err());
    assert!(result.unwrap_err().is::<qrcode::types::QrError>());
}
