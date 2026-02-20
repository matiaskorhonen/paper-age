//! Age based encryption
use std::io::Write;

use age::armor::ArmoredWriter;
use age::armor::Format::AsciiArmor;
use age::secrecy::SecretString;
use log::debug;

/// Encrypt the data from the reader and PEM encode the ciphertext
pub fn encrypt_plaintext(
    reader: &mut dyn std::io::BufRead,
    passphrase: SecretString,
) -> Result<(usize, String), Box<dyn std::error::Error>> {
    debug!("Encrypting plaintext");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_armored_output() {
        let mut input = b"some secrets" as &[u8];
        let passphrase = SecretString::from("snakeoil".to_owned());
        let result = encrypt_plaintext(&mut input, passphrase);

        assert!(result.is_ok());

        let (plaintext_size, armored) = result.unwrap();
        assert_eq!(plaintext_size, 12);

        let first_line: String = armored.lines().take(1).collect();
        assert_eq!(first_line, "-----BEGIN AGE ENCRYPTED FILE-----");

        let last_line: &str = armored.lines().last().unwrap();
        assert_eq!(last_line, "-----END AGE ENCRYPTED FILE-----")
    }
}
