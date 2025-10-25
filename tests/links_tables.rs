use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_table_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Table Test\n\n| Col1 | Col2 |\n|------|------|\n| A    | B    |\n| C    | D    |",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Table Test"));
}

#[test]
fn test_link_styles() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Link Test\n\n[Example](https://example.com)").unwrap();

    // Test inline table style (default)
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-u").arg("it").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test inline style
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-u").arg("i").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test hide style
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-u").arg("h").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));
}

#[test]
fn test_inline_table_link_style_inside_text_code_block_pretty() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```text\nThis is a [link](https://example.com/example-path)\n```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭─ Text"))
        .stdout(predicate::str::contains("│ This is a link[1]"))
        .stdout(predicate::str::contains(
            "\n [1] https://example.com/example-path",
        ))
        .stdout(predicate::str::contains("\n│ [1]").not())
        .stdout(predicate::str::contains("[link](").not());
}

#[test]
fn test_inline_table_link_style_inside_text_code_block_simple() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```text\nThis is a [link](https://example.com/example-path)\n```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("simple")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Text"))
        .stdout(predicate::str::contains("│ This is a link[1]"))
        .stdout(predicate::str::contains(
            "\n [1] https://example.com/example-path",
        ))
        .stdout(predicate::str::contains("│ [1]").not())
        .stdout(predicate::str::contains("[link](").not());
}
