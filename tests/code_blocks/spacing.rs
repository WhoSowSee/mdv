use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_pretty_style_consecutive_code_blocks_have_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```python\nprint(\"hello\")\n```\n\n```python\nprint(\"world\")\n```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╯\n\n╭"))
        .stdout(predicate::str::contains("╯\n\n\n╭").not());
}

#[test]
fn test_code_block_followed_by_heading_has_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hi\")\n```\n\n# Heading\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n\nHeading"),
        "expected single blank line before heading; stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("\n\n\nHeading"),
        "unexpected double blank line before heading; stdout:\n{}",
        normalized
    );
}

#[test]
fn test_code_block_followed_by_rule_has_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hi\")\n```\n\n---\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n\n◈"),
        "expected single blank line before rule; stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("\n\n\n◈"),
        "unexpected double blank line before rule; stdout:\n{}",
        normalized
    );
}
