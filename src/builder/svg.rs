//! Generate SVG formnat QR codes
use log::{debug, info};
use qrcode::{render::svg, types::QrError, EcLevel, QrCode};

/// Generate a QR code svg for the given string. The error correction level of
/// the QR code is optimised (less data → more error correction)
pub fn qrcode(text: String) -> Result<String, QrError> {
    // QR Code Error Correction Capability (approx.)
    //     H: 30%
    //     Q: 25%
    //     M: 15%
    //     L: 7%
    let levels = [EcLevel::H, EcLevel::Q, EcLevel::M, EcLevel::L];

    // Find the best level of EC level possible for the data
    let mut result: Result<QrCode, QrError> = Result::Err(QrError::DataTooLong);
    for ec_level in levels.iter() {
        debug!("Trying EC level {:?}", *ec_level);
        result = QrCode::with_error_correction_level(text.clone(), *ec_level);

        if result.is_ok() {
            break;
        }
    }
    let code = result?;

    info!("QR code EC level: {:?}", code.error_correction_level());
    info!("QR code version: {:?}", code.version());

    let image = code
        .render()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .quiet_zone(false)
        .build();

    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_qrcode() {
        let svg = qrcode(String::from("Some value")).unwrap();
        assert_eq!(
            svg,
            String::from(include_str!("../../tests/data/some_value.svg")).trim()
        );
    }

    #[test]
    fn test_input_too_large() {
        let result = qrcode(String::from(include_str!("../../tests/data/too_large.txt")));
        assert!(result.is_err());
    }
}
