use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_code_highlighting() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Code Test\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Code Test"));
}

#[test]
fn test_no_code_guessing_disables_detection_for_unknown_language() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```dasdasdas\nfn main() {}\n```").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-code-guessing")
        .arg("--style-code-block")
        .arg("simple")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dasdasdas"))
        .stdout(predicate::str::contains("Rust").not());
}

#[test]
fn test_code_language_simple_style_named_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```rust\nfn badge() {\n    println!(\"label\");\n}\n```",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Rust"))
        .stdout(predicate::str::contains("│ Rust\n│ \n│ fn badge()"));
}

#[test]
fn test_no_code_language_flag_hides_label() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```rust\nfn badge() {\n    println!(\"label\");\n}\n```",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-code-language")
        .arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Rust").not())
        .stdout(predicate::str::contains("│ fn badge()"));
}

#[test]
fn test_code_language_simple_style_plain_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```\nplain text output\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Text\n│ \n│ plain text output"));
}



