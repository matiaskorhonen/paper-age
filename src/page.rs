use std::fmt;

use printpdf::{Mm, Point};

/// PDF dimensions
#[derive(Clone, Copy, Debug)]
pub struct PageDimensions {
    pub width: Mm,
    pub height: Mm,
    pub margin: Mm,
}

impl PartialEq for PageDimensions {
    fn eq(&self, other: &PageDimensions) -> bool {
        self.width == other.width && self.height == other.height && self.margin == other.margin
    }
}

impl PageDimensions {
    /// Center point of the page
    pub fn center(&self) -> Point {
        Point::new(self.width / 2.0, self.height / 2.0)
    }

    /// Vertical center left of the page (with margin)
    pub fn center_left(&self) -> Point {
        Point::new(self.margin, self.height / 2.0)
    }

    /// Vertical center right of the page (with margin)
    pub fn center_right(&self) -> Point {
        Point::new(self.width - self.margin, self.height / 2.0)
    }

    /// Top left of the page (with margin)
    pub fn top_left(&self) -> Point {
        Point::new(self.margin, self.height - self.margin)
    }

    /// Top right of the page (with margin)
    pub fn top_right(&self) -> Point {
        Point::new(self.width - self.margin, self.height - self.margin)
    }

    /// Bottom left of the page (with margin)
    pub fn bottom_left(&self) -> Point {
        Point::new(self.margin, self.margin)
    }

    /// Bottom right of the page (with margin)
    pub fn bottom_right(&self) -> Point {
        Point::new(self.width - self.margin, self.margin)
    }
}

/// A4 dimensions with a 10mm margin
pub const A4_PAGE: PageDimensions = PageDimensions {
    width: Mm(210.0),
    height: Mm(297.0),
    margin: Mm(10.0),
};

/// Letter dimensions with a 10mm margin
pub const LETTER_PAGE: PageDimensions = PageDimensions {
    width: Mm(215.9),
    height: Mm(279.4),
    margin: Mm(10.0),
};

/// Default to an A4 page
impl Default for PageDimensions {
    fn default() -> Self {
        A4_PAGE
    }
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum PageSize {
    A4,
    Letter,
}

impl PageSize {
    /// Page dimensions
    pub fn dimensions(&self) -> PageDimensions {
        match self {
            PageSize::A4 => A4_PAGE,
            PageSize::Letter => LETTER_PAGE,
        }
    }

    /// QR code size for the page size
    pub fn qrcode_size(&self) -> Mm {
        match self {
            PageSize::A4 => Mm(110.0),
            PageSize::Letter => Mm(102.0),
        }
    }

    /// The left edge of the QR code on the page
    pub fn qrcode_left_edge(&self) -> Mm {
        (self.dimensions().width - self.qrcode_size()) / 2.0
    }
}

impl fmt::Display for PageSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIMENSIONS: PageDimensions = PageDimensions {
        width: Mm(100.0),
        height: Mm(200.0),
        margin: Mm(10.0),
    };

    #[test]
    fn page_dimensions_center() {
        assert_eq!(TEST_DIMENSIONS.center(), Point::new(Mm(50.0), Mm(100.0)));
    }

    #[test]
    fn page_dimensions_center_left() {
        assert_eq!(
            TEST_DIMENSIONS.center_left(),
            Point::new(Mm(10.0), Mm(100.0))
        );
    }

    #[test]
    fn page_dimensions_center_right() {
        assert_eq!(
            TEST_DIMENSIONS.center_right(),
            Point::new(Mm(90.0), Mm(100.0))
        );
    }

    #[test]
    fn page_dimensions_top_left() {
        assert_eq!(TEST_DIMENSIONS.top_left(), Point::new(Mm(10.0), Mm(190.0)));
    }

    #[test]
    fn page_dimensions_top_right() {
        assert_eq!(TEST_DIMENSIONS.top_right(), Point::new(Mm(90.0), Mm(190.0)));
    }

    #[test]
    fn page_dimensions_bottom_left() {
        assert_eq!(
            TEST_DIMENSIONS.bottom_left(),
            Point::new(Mm(10.0), Mm(10.0))
        );
    }

    #[test]
    fn page_dimensions_bottom_right() {
        assert_eq!(
            TEST_DIMENSIONS.bottom_right(),
            Point::new(Mm(90.0), Mm(10.0))
        );
    }
}
