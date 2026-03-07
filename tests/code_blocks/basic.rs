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
    fs::write(&temp_file, "```unknownlang\nfn main() {}\n```").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-code-guessing")
        .arg("--code-block-style")
        .arg("simple")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Unknownlang"))
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
    cmd.arg("--code-block-style")
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
        .arg("--code-block-style")
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
    cmd.arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Text\n│ \n│ plain text output"));
}

#[test]
fn test_markdown_code_block_setext_heading_renders_as_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```markdown\nTitle\n---\nBody\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for markdown code block");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("Title") && stdout.contains("Body"),
        "expected setext heading inside markdown code block, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("◈") && !stdout.contains("│ ---"),
        "expected no horizontal rule for setext heading, stdout:\n{}",
        stdout
    );
}
