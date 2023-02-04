use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new().unwrap();
    let output = temp.child("output.pdf");
    let mut cmd = Command::cargo_bin("paper-age")?;

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
fn test_fonts_license() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("paper-age")?;

    cmd.arg("--fonts-license");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SIL OPEN FONT LICENSE"));

    Ok(())
}

#[test]
fn test_title_too_long() -> Result<(), Box<dyn std::error::Error>> {
    let input = assert_fs::NamedTempFile::new("sample.txt")?;
    let mut cmd = Command::cargo_bin("paper-age")?;

    cmd.arg("--title").arg("x".repeat(80)).arg(input.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("The title cannot be longer than"));

    Ok(())
}
