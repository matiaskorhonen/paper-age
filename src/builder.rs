//! PaperAge

use anyhow::{anyhow, Result};
use printpdf::{
    Color, ExternalXObject, FontId, Layer, LayerInternalId, Line, LineDashPattern, LinePoint, Mm,
    Op, ParsedFont, PdfDocument, PdfPage, PdfWarnMsg, Point, Pt, Rgb, Svg, TextItem, XObjectId,
    XObjectTransform,
};

use crate::page::*;

pub mod svg;

/// PaperAge version
pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

/// Font width / height = 3 / 5
const FONT_RATIO: f32 = 3.0 / 5.0;

/// Container for all the data required to insert elements into the PDF
pub struct DocumentBuilder {
    pub title: String,

    pub page_size: PageSize,

    pub grid: bool,

    pub notes_label: String,

    pub skip_notes_line: bool,
}

impl DocumentBuilder {
    pub fn new(title: String, page_size: PageSize) -> Self {
        Self {
            title,
            page_size,
            grid: false,
            notes_label: "".to_string(),
            skip_notes_line: false,
        }
    }

    pub fn build(&self, encrypted_data: &str) -> Result<PdfDocument> {
        let producer = format!("Paper Rage v{}", VERSION.unwrap_or("0.0.0"));
        let mut warnings = vec![];

        let mut doc = PdfDocument::new(&self.title);
        doc.metadata.info.producer = producer;

        // Load fonts
        let title_font_data = include_bytes!("assets/fonts/IBMPlexMono-Medium.ttf");
        let title_font = ParsedFont::from_bytes(title_font_data, 0, &mut warnings)
            .ok_or(anyhow!("Can't load title font!"))?;
        let title_font_id = doc.add_font(&title_font);
        let code_font_data = include_bytes!("assets/fonts/IBMPlexMono-Regular.ttf");
        let code_font = ParsedFont::from_bytes(code_font_data, 1, &mut warnings)
            .ok_or(anyhow!("Can't load title font!"))?;
        let code_font_id = doc.add_font(&code_font);

        // Create QR-code
        let qrcode = self.generate_qr_code(encrypted_data, &mut warnings)?;
        let qrcode_id = doc.add_xobject(&qrcode);

        // Create Layers
        let background_layer = Layer::new("Background");
        let background_layer_id = doc.add_layer(&background_layer);

        let layer = Layer::new("Foreground");
        let layer_id = doc.add_layer(&layer);

        // Build
        let content: Vec<Op> = [
            self.build_background_layer(&background_layer_id),
            self.build_foreground_layer(
                encrypted_data,
                &layer_id,
                &title_font_id,
                &code_font_id,
                &qrcode_id,
                &qrcode,
            ),
        ]
        .into_iter()
        .flatten()
        .collect();
        let page = PdfPage::new(
            self.page_size.dimensions().width,
            self.page_size.dimensions().height,
            content,
        );
        doc.with_pages(vec![page]);
        Ok(doc)
    }

    pub fn generate_qr_code(
        &self,
        encrypted_data: &str,
        warnings: &mut Vec<PdfWarnMsg>,
    ) -> Result<ExternalXObject> {
        let image = svg::qrcode(encrypted_data)?;
        Svg::parse(image.as_str(), warnings)
            .map_err(|_| anyhow!("The QR code generation failed for an unknown reason"))
    }

    fn build_background_layer(&self, layer: &LayerInternalId) -> Vec<Op> {
        let fill_color = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));
        let contents = [vec![Op::SetFillColor { col: fill_color }]];
        self.build_layer(layer, contents)
    }

    fn build_foreground_layer(
        &self,
        encrypted_data: &str,
        layer: &LayerInternalId,
        title_font_id: &FontId,
        code_font_id: &FontId,
        qrcode_id: &XObjectId,
        qrcode: &ExternalXObject,
    ) -> Vec<Op> {
        let contents = [
            self.grid.then(|| self.draw_grid()).unwrap_or(vec![]),
            self.insert_title_text(title_font_id),
            self.insert_qr_code(qrcode_id, qrcode),
            self.insert_notes_field(
                self.notes_label.clone(),
                self.skip_notes_line,
                title_font_id,
            ),
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
            ),
            self.insert_pem_text(encrypted_data, code_font_id),
            self.insert_footer(title_font_id),
        ];
        self.build_layer(layer, contents)
    }

    fn build_layer<C: IntoIterator<Item = Vec<Op>>>(
        &self,
        layer: &LayerInternalId,
        contents: C,
    ) -> Vec<Op> {
        let mut layer_content = vec![];
        layer_content.push(Op::BeginLayer {
            layer_id: layer.clone(),
        });
        for content in contents {
            layer_content.extend(content)
        }
        layer_content.push(Op::EndLayer {
            layer_id: layer.clone(),
        });
        layer_content
    }

    /// Insert the given title at the top of the PDF
    fn insert_title_text(&self, font: &FontId) -> Vec<Op> {
        let font_size = 14.0;

        // Align the title with the QR code if the title is narrower than the QR code
        let margin = {
            if self.title.len() <= 37 {
                self.page_size.qrcode_left_edge()
            } else {
                self.page_size.dimensions().margin
            }
        };

        vec![
            Op::StartTextSection,
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
            },
            Op::SetFontSize {
                size: Pt(font_size),
                font: font.clone(),
            },
            Op::SetTextCursor {
                pos: Point {
                    x: margin.into(),
                    y: (self.page_size.dimensions().height
                        - self.page_size.dimensions().margin
                        - Mm::from(Pt(font_size)))
                    .into(),
                },
            },
            Op::WriteText {
                items: vec![TextItem::Text(self.title.clone())],
                font: font.clone(),
            },
            Op::EndTextSection,
        ]
    }

    /// Insert the given PEM ciphertext in the bottom half of the page
    fn insert_pem_text(&self, pem: &str, font: &FontId) -> Vec<Op> {
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

        let mut content = vec![
            Op::StartTextSection,
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
            },
            Op::SetLineHeight {
                lh: Pt(line_height),
            },
            Op::SetFontSize {
                size: Pt(font_size),
                font: font.clone(),
            },
            Op::SetTextCursor {
                pos: Point {
                    x: self.page_size.dimensions().margin.into(),
                    y: ((self.page_size.dimensions().height / 2.0)
                        - Mm::from(Pt(font_size))
                        - self.page_size.dimensions().margin)
                        .into(),
                },
            },
        ];
        for line in pem.lines() {
            content.extend(vec![
                Op::WriteText {
                    items: vec![TextItem::Text(line.to_string())],
                    font: font.clone(),
                },
                Op::AddLineBreak,
            ])
        }
        content.push(Op::EndTextSection);
        content
    }

    /// Insert the QR code of the PEM encoded ciphertext in the top half of the page
    fn insert_qr_code(&self, qrcode_id: &XObjectId, qrcode: &ExternalXObject) -> Vec<Op> {
        let qrcode_width = qrcode.width.expect("QR code need to have not zero width");
        let qrcode_height = qrcode.height.expect("QR code need to have not zero height");
        let desired_qr_size = self.page_size.qrcode_size();
        let initial_qr_size = Mm::from(qrcode_height.into_pt(300.0));
        let qr_scale = desired_qr_size / initial_qr_size;

        let scale = qr_scale;
        let dpi = 300.0;
        let code_width = qrcode_width.into_pt(dpi) * scale;
        let code_height = qrcode_height.into_pt(dpi) * scale;

        let translate_x = (self.page_size.dimensions().width.into_pt() - code_width) / 2.0;
        let translate_y = self.page_size.dimensions().height.into_pt()
            - code_height
            - (self.page_size.dimensions().margin.into_pt() * 2.0);

        vec![Op::UseXobject {
            id: qrcode_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(translate_x),
                translate_y: Some(translate_y),
                rotate: None,
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(dpi),
            },
        }]
    }

    /// Draw a grid debugging layout issues
    fn draw_grid(&self) -> Vec<Op> {
        let mut operations = vec![];
        let grid_size = Mm(5.0);
        let thickness = 0.0;

        let mut x = Mm(0.0);
        let mut y = self.page_size.dimensions().height;
        while x < self.page_size.dimensions().width {
            x += grid_size;

            let line = self.draw_line(
                vec![
                    Point::new(x, self.page_size.dimensions().height),
                    Point::new(x, Mm(0.0)),
                ],
                thickness,
                LineDashPattern::default(),
            );
            operations.extend(line);

            while y > Mm(0.0) {
                y -= grid_size;

                let line = self.draw_line(
                    vec![
                        Point::new(self.page_size.dimensions().width, y),
                        Point::new(Mm(0.0), y),
                    ],
                    thickness,
                    LineDashPattern::default(),
                );
                operations.extend(line);
            }
        }
        operations
    }

    /// Draw a line on the page
    fn draw_line(
        &self,
        points: Vec<Point>,
        thickness: f32,
        dash_pattern: LineDashPattern,
    ) -> Vec<Op> {
        let line = Line {
            points: points
                .into_iter()
                .map(|p| LinePoint { p, bezier: false })
                .collect(),
            is_closed: false,
        };

        vec![
            Op::SaveGraphicsState,
            Op::SetOutlineColor {
                col: Color::Rgb(Rgb::new(0.75, 0.75, 0.75, None)),
            },
            Op::SetLineDashPattern { dash: dash_pattern },
            Op::SetOutlineThickness { pt: Pt(thickness) },
            Op::DrawLine { line },
            Op::RestoreGraphicsState,
        ]
    }

    /// Insert the notes field label and placeholder in the PDF
    pub fn insert_notes_field(&self, label: String, skip_line: bool, font: &FontId) -> Vec<Op> {
        const MAX_LABEL_LEN: usize = 32;

        let baseline =
            self.page_size.dimensions().height / 2.0 + self.page_size.dimensions().margin;

        let label_len = label.len();

        let font_size = 13.0;

        let mut content = vec![
            Op::StartTextSection,
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
            },
            Op::SetFontSize {
                size: Pt(font_size),
                font: font.clone(),
            },
            Op::SetTextCursor {
                pos: Point {
                    x: self.page_size.qrcode_left_edge().into(),
                    y: baseline.into(),
                },
            },
            Op::WriteText {
                items: vec![TextItem::Text(label)],
                font: font.clone(),
            },
            Op::EndTextSection,
        ];

        if label_len <= MAX_LABEL_LEN && !skip_line {
            content.extend(self.draw_line(
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
            ))
        }
        content
    }

    /// Add the footer at the bottom of the page
    pub fn insert_footer(&self, font: &FontId) -> Vec<Op> {
        vec![
            Op::StartTextSection,
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
            },
            Op::SetFontSize {
                size: Pt(13.0),
                font: font.clone(),
            },
            Op::SetTextCursor {
                pos: Point {
                    x: self.page_size.dimensions().margin.into(),
                    y: self.page_size.dimensions().margin.into(),
                },
            },
            Op::WriteText {
                items: vec![TextItem::Text(
                    "Scan QR code and decrypt using Age <https://age-encryption.org>".to_string(),
                )],
                font: font.clone(),
            },
            Op::EndTextSection,
        ]
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
    let doc = DocumentBuilder::new(title, PageSize::A4);
    assert_eq!(doc.page_size.dimensions(), crate::page::A4_PAGE);
}

#[test]
fn test_new_letter_document() {
    let title = String::from("Hello Letter!");
    let doc = DocumentBuilder::new(title, PageSize::Letter);
    assert_eq!(doc.page_size.dimensions(), crate::page::LETTER_PAGE);
}

#[test]
fn test_qrcode() {
    let mut warnings = vec![];
    let doc = DocumentBuilder::new(String::from("QR code"), PageSize::A4);
    let result = doc.generate_qr_code("payload", &mut warnings);
    assert!(result.is_ok());
}

#[test]
fn test_qrcode_too_large() {
    let mut warnings = vec![];
    let doc = DocumentBuilder::new(String::from("QR code"), PageSize::A4);
    let result = doc.generate_qr_code(include_str!("../tests/data/too_large.txt"), &mut warnings);
    assert!(result.is_err());
    assert!(result.unwrap_err().is::<qrcode::types::QrError>());
}
