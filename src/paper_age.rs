use std::io::{BufReader, Cursor};

use printpdf::{
    Color, IndirectFontRef, Line, LineDashPattern, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfLayerReference, PdfPageIndex, Point, Pt, Rgb, Svg, SvgTransform,
};

pub mod cli;
pub mod encryption;
pub mod svg;

pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
pub struct PageDimensions {
    pub width: Mm,
    pub height: Mm,
    pub margin: Mm,
}

pub const A4_PAGE: PageDimensions = PageDimensions {
    width: Mm(210.0),
    height: Mm(297.0),
    margin: Mm(10.0),
};

impl Default for PageDimensions {
    fn default() -> Self {
        A4_PAGE
    }
}

pub struct Document {
    pub doc: PdfDocumentReference,
    pub page: PdfPageIndex,
    pub layer: PdfLayerIndex,
    pub title_font: IndirectFontRef,
    pub code_font: IndirectFontRef,
    pub dimensions: PageDimensions,
}

impl Document {
    pub fn new(title: String) -> Result<Document, Box<dyn std::error::Error>> {
        let dimensions: PageDimensions = Default::default();

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

    fn get_current_layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.page).get_layer(self.layer)
    }

    pub fn insert_title_text(&self, title: String) {
        let current_layer = self.get_current_layer();

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

    pub fn insert_pem_text(&self, pem: String) {
        let current_layer = self.get_current_layer();

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

    pub fn insert_qr_code(&self, text: String) -> Result<(), Box<dyn std::error::Error>> {
        let image = svg::qrcode(text)?;
        let qrcode = Svg::parse(image.as_str())?;

        let current_layer = self.get_current_layer();

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

        Ok(())
    }

    pub fn draw_grid(&self) {
        let grid_size = Mm(5.0);
        let thickness = 0.0;

        let mut x = Mm(0.0);
        let mut y = self.dimensions.height;
        while x < self.dimensions.width {
            x += grid_size;

            self.draw_line(
                vec![
                    Point::new(x, self.dimensions.height),
                    Point::new(x, Mm(0.0)),
                ],
                thickness,
                LineDashPattern::default(),
            );

            while y > Mm(0.0) {
                y -= grid_size;

                self.draw_line(
                    vec![Point::new(self.dimensions.width, y), Point::new(Mm(0.0), y)],
                    thickness,
                    LineDashPattern::default(),
                );
            }
        }
    }

    pub fn draw_line(&self, points: Vec<Point>, thickness: f64, dash_pattern: LineDashPattern) {
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

    pub fn insert_passphrase(&self) {
        let current_layer = self.get_current_layer();

        let baseline = self.dimensions.height / 2.0 + self.dimensions.margin;

        current_layer.use_text("Passphrase: ", 13.0, Mm(50.0), baseline, &self.title_font);

        self.draw_line(
            vec![
                Point::new(Mm(50.0) + Mm(30.0), baseline - Mm(1.0)),
                Point::new(Mm(110.0) + Mm(50.0), baseline - Mm(1.0)),
            ],
            1.0,
            LineDashPattern::default(),
        )
    }

    pub fn insert_footer(&self) {
        let current_layer = self.get_current_layer();

        current_layer.use_text(
            "Scan QR code and decrypt using Age <https://age-encryption.org>",
            13.0,
            self.dimensions.margin,
            self.dimensions.margin,
            &self.title_font,
        );
    }
}
