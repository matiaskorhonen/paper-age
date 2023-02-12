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
    pub fn center(&self) -> Point {
        Point::new(self.width / 2.0, self.height / 2.0)
    }

    pub fn center_left(&self) -> Point {
        Point::new(Mm(0.0) + self.margin, self.height / 2.0)
    }

    pub fn center_right(&self) -> Point {
        Point::new(self.width - self.margin, self.height / 2.0)
    }

    pub fn top_left(&self) -> Point {
        Point::new(Mm(0.0), self.height - self.margin)
    }

    pub fn top_right(&self) -> Point {
        Point::new(self.width - self.margin, self.height - self.margin)
    }

    pub fn bottom_left(&self) -> Point {
        Point::new(self.margin, self.margin)
    }

    pub fn bottom_right(&self) -> Point {
        Point::new(self.width, self.margin)
    }
}

/// A4 dimensions with a 10mm margin
pub const A4_PAGE: PageDimensions = PageDimensions {
    width: Mm(210.0),
    height: Mm(297.0),
    margin: Mm(10.0),
};

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
    pub fn dimensions(&self) -> PageDimensions {
        match self {
            PageSize::A4 => A4_PAGE,
            PageSize::Letter => LETTER_PAGE,
        }
    }

    pub fn qrcode_size(&self) -> Mm {
        match self {
            PageSize::A4 => Mm(110.0),
            PageSize::Letter => Mm(100.0),
        }
    }

    pub fn qrcode_left_edge(&self) -> Mm {
        (self.dimensions().width - self.qrcode_size()) / 2.0
    }
}

impl fmt::Display for PageSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}
