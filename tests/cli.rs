#![cfg(feature = "cli")]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_happy_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let input = temp.child("sample.txt");
    input.write_str("Hello")?;
    let output = temp.child("output.pdf");
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--output")
        .arg(output.path())
        .arg("--grid")
        .arg(input.path())
        .env("PAPERAGE_PASSPHRASE", "secret");
    cmd.assert().success();

    output.assert(predicate::path::is_file());

    Ok(())
}

#[test]
fn test_custom_footer() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let input = temp.child("sample.txt");
    input.write_str("Hello")?;
    let output = temp.child("footer.pdf");
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--output")
        .arg(output.path())
        .arg("--footer")
        .arg("Custom footer")
        .arg(input.path())
        .env("PAPERAGE_PASSPHRASE", "secret");
    cmd.assert().success();

    output.assert(predicate::path::is_file());

    Ok(())
}

#[test]
fn test_letter_support() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let input = temp.child("sample.txt");
    input.write_str("Hello")?;
    let output = temp.child("letter.pdf");
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--output")
        .arg(output.path())
        .arg("--page-size")
        .arg("letter")
        .arg(input.path())
        .env("PAPERAGE_PASSPHRASE", "secret");
    cmd.assert().success();

    output.assert(predicate::path::is_file());

    Ok(())
}

#[test]
fn test_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let input = temp.child("sample.txt");
    input.write_str("STDOUT")?;

    let mut cmd = cargo_bin_cmd!("paper-age");

    // Check that a complete PDF lands on stdout. The exact byte count depends on
    // how printpdf compresses its streams, so assert on the PDF structure plus a
    // conservative size floor rather than on a precise length.
    let min_size = 16 * 1024; // 16 KiB
    let pdf_predicate_fn = predicate::function(move |x: &[u8]| {
        x.starts_with(b"%PDF-") && x.ends_with(b"%%EOF") && x.len() > min_size
    });

    cmd.arg("--output")
        .arg("-")
        .arg(input.path())
        .env("PAPERAGE_PASSPHRASE", "secret");
    cmd.assert().stdout(pdf_predicate_fn).success();

    Ok(())
}

#[test]
fn test_file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let output = temp.child("output.pdf");
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--output")
        .arg(output.path())
        .arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(predicate::str::contains(
        "File not found: test/file/doesnt/exist",
    ));

    output.assert(predicate::path::missing());

    Ok(())
}

#[test]
fn test_too_much_data() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let input = temp.child("sample.txt");
    input.write_str("x".repeat(2048).as_str())?;
    let output = temp.child("output.pdf");
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--output")
        .arg(output.path())
        .arg(input.path())
        .env("PAPERAGE_PASSPHRASE", "secret");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Too much data after encryption"));

    output.assert(predicate::path::missing());

    Ok(())
}

#[test]
fn test_fonts_license() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--fonts-license");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SIL OPEN FONT LICENSE"));

    Ok(())
}

#[test]
fn test_title_too_long() -> Result<(), Box<dyn std::error::Error>> {
    let input = assert_fs::NamedTempFile::new("sample.txt")?;
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--title").arg("x".repeat(80)).arg(input.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("The title cannot be longer than"));

    Ok(())
}

#[test]
fn test_footer_too_long() -> Result<(), Box<dyn std::error::Error>> {
    let input = assert_fs::NamedTempFile::new("sample.txt")?;
    let mut cmd = cargo_bin_cmd!("paper-age");

    cmd.arg("--footer").arg("x".repeat(80)).arg(input.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("The footer cannot be longer than"));

    Ok(())
}
