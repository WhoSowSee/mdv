use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

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

    let mut cmd = mdv_cmd();
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
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("it").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test document-level table style
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("et").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test inline style
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("i").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test hide style
    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

#[test]
fn test_end_table_link_style_collects_references_at_document_end() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\nParagraph with [one](https://example.com/one) link.\n\nSecond [two](https://example.com/two) link here.\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-u")
        .arg("et")
        .arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with endtable style");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid utf-8");
    assert!(
        stdout.contains("one[1]"),
        "link text should include reference number: {}",
        stdout
    );
    assert!(
        stdout.contains("two[2]"),
        "second link should include reference number: {}",
        stdout
    );
    let tail = stdout.trim_end();
    let lines: Vec<&str> = tail.lines().collect();
    assert!(
        lines.len() >= 2,
        "output should contain at least two lines for references: {}",
        tail
    );
    let last_two = &lines[lines.len().saturating_sub(2)..];
    assert_eq!(
        last_two[0].trim_start(),
        "[1] https://example.com/one",
        "first reference line mismatch: {}",
        tail
    );
    assert_eq!(
        last_two[1].trim_start(),
        "[2] https://example.com/two",
        "second reference line mismatch: {}",
        tail
    );
}





