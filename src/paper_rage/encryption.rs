use std::io::{BufReader, Read, Write};

use age::armor::ArmoredWriter;
use age::armor::Format::AsciiArmor;
use age::secrecy::Secret;

pub fn encrypt_plaintext(
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
