//! Convenience function for end-to-end PDF generation

use std::fmt;
use std::io::BufRead;

use age::secrecy::SecretString;

use crate::builder;
use crate::encryption;
use crate::page::{PageSize, DEFAULT_FOOTER};

/// Errors that can occur during PDF generation
#[derive(Debug)]
#[non_exhaustive]
pub enum PaperAgeError {
    /// An argument was invalid (e.g. the title or footer is too long)
    InvalidInput(String),
    /// The plaintext data could not be encrypted
    Encryption(String),
    /// The PDF document could not be initialized
    DocumentInit(String),
    /// The PDF could not be created (e.g. QR code too large)
    PdfCreation(String),
}

impl fmt::Display for PaperAgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaperAgeError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            PaperAgeError::Encryption(msg) => write!(f, "Encryption failed: {msg}"),
            PaperAgeError::DocumentInit(msg) => write!(f, "Document initialization failed: {msg}"),
            PaperAgeError::PdfCreation(msg) => write!(f, "PDF creation failed: {msg}"),
        }
    }
}

impl std::error::Error for PaperAgeError {}

/// Optional settings for [`create_pdf`].
///
/// Start from [`PdfOptions::default()`] and chain the setters for the settings
/// you want to change. New settings may be added in future releases without
/// breaking existing code.
///
/// # Example
///
/// ```
/// use paper_age::convenience::PdfOptions;
/// use paper_age::page::PageSize;
///
/// let options = PdfOptions::default()
///     .page_size(PageSize::Letter)
///     .footer("Keep this somewhere safe");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PdfOptions {
    notes_label: String,
    skip_notes_line: bool,
    page_size: PageSize,
    grid: bool,
    footer: String,
}

impl Default for PdfOptions {
    fn default() -> Self {
        PdfOptions {
            notes_label: "Passphrase:".to_string(),
            skip_notes_line: false,
            page_size: PageSize::A4,
            grid: false,
            footer: DEFAULT_FOOTER.to_string(),
        }
    }
}

impl PdfOptions {
    /// Label for the notes field (default: `"Passphrase:"`)
    pub fn notes_label(mut self, notes_label: impl Into<String>) -> Self {
        self.notes_label = notes_label.into();
        self
    }

    /// Whether to omit the notes placeholder line (default: `false`)
    pub fn skip_notes_line(mut self, skip_notes_line: bool) -> Self {
        self.skip_notes_line = skip_notes_line;
        self
    }

    /// The page size to use (default: [`PageSize::A4`])
    pub fn page_size(mut self, page_size: PageSize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Whether to draw a debug grid on the page (default: `false`)
    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    /// Footer text at the bottom of the page, at most
    /// [`FOOTER_MAX_LEN`](builder::FOOTER_MAX_LEN) characters (default:
    /// [`DEFAULT_FOOTER`])
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = footer.into();
        self
    }
}

/// Generate a PaperAge PDF from plaintext data and a passphrase.
///
/// This is a high-level convenience function that handles encryption and PDF
/// generation in a single call.
///
/// # Arguments
///
/// * `title` - The document title (appears in the PDF and its metadata), at
///   most [`TITLE_MAX_LEN`](builder::TITLE_MAX_LEN) characters
/// * `data` - A buffered reader providing the plaintext data to encrypt
/// * `passphrase` - The passphrase used to encrypt the data
/// * `options` - Optional settings, see [`PdfOptions`]
///
/// # Returns
///
/// The PDF file contents as a `Vec<u8>`, or a [`PaperAgeError`] describing
/// what went wrong.
///
/// # Example
///
/// ```no_run
/// use paper_age::convenience::{create_pdf, PdfOptions};
/// use paper_age::page::PageSize;
///
/// let pdf_bytes = create_pdf(
///     "My Secret",
///     &mut &b"secret data to encrypt"[..],
///     "hunter2",
///     &PdfOptions::default().page_size(PageSize::Letter),
/// ).expect("PDF generation failed");
/// ```
pub fn create_pdf(
    title: impl Into<String>,
    data: &mut dyn BufRead,
    passphrase: &str,
    options: &PdfOptions,
) -> Result<Vec<u8>, PaperAgeError> {
    let title = title.into();

    if title.len() > builder::TITLE_MAX_LEN {
        return Err(PaperAgeError::InvalidInput(format!(
            "The title cannot be longer than {} characters",
            builder::TITLE_MAX_LEN
        )));
    }

    if options.footer.len() > builder::FOOTER_MAX_LEN {
        return Err(PaperAgeError::InvalidInput(format!(
            "The footer cannot be longer than {} characters",
            builder::FOOTER_MAX_LEN
        )));
    }

    let passphrase_secret = SecretString::from(passphrase.to_owned());

    let (_plaintext_len, encrypted) = encryption::encrypt_plaintext(data, passphrase_secret)
        .map_err(|e| PaperAgeError::Encryption(e.to_string()))?;

    let pdf = builder::Document::new(title, options.page_size.clone())
        .map_err(|e| PaperAgeError::DocumentInit(e.to_string()))?
        .with_footer(options.footer.clone());

    let bytes = pdf
        .create_pdf(
            options.grid,
            options.notes_label.clone(),
            options.skip_notes_line,
            encrypted,
        )
        .map_err(|e| PaperAgeError::PdfCreation(e.to_string()))?;

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_defaults() {
        let options = PdfOptions::default();
        assert_eq!(options.notes_label, "Passphrase:");
        assert!(!options.skip_notes_line);
        assert_eq!(options.page_size, PageSize::A4);
        assert!(!options.grid);
        assert_eq!(options.footer, DEFAULT_FOOTER);
    }

    #[test]
    fn test_options_setters() {
        let options = PdfOptions::default()
            .notes_label("Recovery key:")
            .skip_notes_line(true)
            .page_size(PageSize::Letter)
            .grid(true)
            .footer("Custom footer");
        assert_eq!(options.notes_label, "Recovery key:");
        assert!(options.skip_notes_line);
        assert_eq!(options.page_size, PageSize::Letter);
        assert!(options.grid);
        assert_eq!(options.footer, "Custom footer");
    }

    #[test]
    fn test_create_pdf_defaults() {
        let result = create_pdf(
            "Test Document",
            &mut &b"hello world"[..],
            "passphrase",
            &PdfOptions::default(),
        );
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_create_pdf_with_options() {
        let options = PdfOptions::default()
            .notes_label("Recovery key:")
            .skip_notes_line(true)
            .page_size(PageSize::Letter)
            .grid(true)
            .footer("Custom footer");
        let result = create_pdf(
            "Custom Document".to_string(),
            &mut &b"secret data"[..],
            "hunter2",
            &options,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_pdf_empty_data() {
        let result = create_pdf("Empty", &mut &b""[..], "passphrase", &PdfOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_pdf_title_at_max_len() {
        let result = create_pdf(
            "x".repeat(builder::TITLE_MAX_LEN),
            &mut &b"secret data"[..],
            "passphrase",
            &PdfOptions::default(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_pdf_title_too_long() {
        let result = create_pdf(
            "x".repeat(builder::TITLE_MAX_LEN + 1),
            &mut &b"secret data"[..],
            "passphrase",
            &PdfOptions::default(),
        );
        assert!(matches!(result, Err(PaperAgeError::InvalidInput(_))));
    }

    #[test]
    fn test_create_pdf_footer_at_max_len() {
        let result = create_pdf(
            "Footer",
            &mut &b"secret data"[..],
            "passphrase",
            &PdfOptions::default().footer("x".repeat(builder::FOOTER_MAX_LEN)),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_pdf_footer_too_long() {
        let result = create_pdf(
            "Footer",
            &mut &b"secret data"[..],
            "passphrase",
            &PdfOptions::default().footer("x".repeat(builder::FOOTER_MAX_LEN + 1)),
        );
        assert!(matches!(result, Err(PaperAgeError::InvalidInput(_))));
    }

    /// A reader whose reads always fail
    struct FailingReader;

    impl std::io::Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("read failed"))
        }
    }

    #[test]
    fn test_create_pdf_read_error() {
        let mut reader = std::io::BufReader::new(FailingReader);
        let result = create_pdf(
            "Read error",
            &mut reader,
            "passphrase",
            &PdfOptions::default(),
        );
        assert!(matches!(result, Err(PaperAgeError::Encryption(_))));
    }

    #[test]
    fn test_create_pdf_data_too_large() {
        let data = vec![b'x'; 4096];
        let result = create_pdf(
            "Too large",
            &mut &data[..],
            "passphrase",
            &PdfOptions::default(),
        );
        assert!(matches!(result, Err(PaperAgeError::PdfCreation(_))));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            PaperAgeError::InvalidInput("footer too long".to_string()).to_string(),
            "Invalid input: footer too long"
        );
        assert_eq!(
            PaperAgeError::Encryption("bad key".to_string()).to_string(),
            "Encryption failed: bad key"
        );
        assert_eq!(
            PaperAgeError::DocumentInit("missing font".to_string()).to_string(),
            "Document initialization failed: missing font"
        );
        assert_eq!(
            PaperAgeError::PdfCreation("QR too large".to_string()).to_string(),
            "PDF creation failed: QR too large"
        );
    }
}
