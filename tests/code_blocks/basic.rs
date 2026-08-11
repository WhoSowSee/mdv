use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

use mdv::utils::display_width;

const RUST_OVERRIDE_SYNTAX: &str = r#"%YAML 1.2
---
name: Rust Override
scope: source.rust
file_extensions:
  - rs
contexts:
  main:
    - match: '\bfn\b'
      scope: keyword.control.rust
"#;

#[test]
fn test_default_code_block_style_has_no_frame_or_label() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn demo() {}\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let content_line = stdout
        .lines()
        .find(|line| line.contains("fn demo"))
        .expect("code line rendered");

    assert_eq!(content_line, "  fn demo() {}");
    assert!(!stdout.contains("Rust"), "stdout:\n{}", stdout);
    assert!(!stdout.contains('│'), "stdout:\n{}", stdout);
    assert!(!stdout.contains('╭'), "stdout:\n{}", stdout);
}

#[test]
fn test_basic_code_block_label_options_are_independent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn demo() {}\n```\n").unwrap();

    let cases = [
        ("basic:show-name", true, false),
        ("basic:show-icon", false, true),
        ("basic:show-name;show-icon", true, true),
    ];

    for (style, has_name, has_icon) in cases {
        let output = mdv_cmd()
            .arg("--no-config")
            .arg("--code-block-style")
            .arg(style)
            .arg("-A")
            .arg(temp_file.path())
            .output()
            .expect("mdv executed");

        assert!(output.status.success(), "style: {}", style);
        let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
        assert_eq!(stdout.contains("Rust"), has_name, "style: {}", style);
        assert_eq!(stdout.contains(''), has_icon, "style: {}", style);
    }
}

#[test]
fn test_basic_code_block_wrap_reserves_indent_width() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```text\nabcdefghijklmnop\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--code-block-style")
        .arg("basic")
        .arg("--wrap")
        .arg("char")
        .arg("--cols")
        .arg("10")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let code_lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    assert_eq!(code_lines.len(), 2, "stdout:\n{}", stdout);
    assert!(code_lines.iter().all(|line| line.starts_with("  ")));
    assert!(
        code_lines.iter().all(|line| display_width(line) <= 10),
        "stdout:\n{}",
        stdout
    );
}

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
        .arg("simple:show-name")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Unknownlang"))
        .stdout(predicate::str::contains("Rust").not());
}

#[test]
fn test_configured_custom_syntax_overrides_embedded_set() {
    let temp_dir = TempDir::new().unwrap();
    let syntaxes_dir = temp_dir.path().join("syntaxes");
    fs::create_dir(&syntaxes_dir).unwrap();
    fs::write(
        syntaxes_dir.join("RustOverride.sublime-syntax"),
        RUST_OVERRIDE_SYNTAX,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("config.yaml"),
        "syntaxes_dir: syntaxes\ncode_block_style: simple:show-name\nno_colors: true\ncode_guessing: false\n",
    )
    .unwrap();
    let markdown_path = temp_dir.path().join("custom-syntax.md");
    fs::write(
        &markdown_path,
        "```rs\nfn main() {}\n```\n\n```json\n{}\n```\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--config-file")
        .arg(temp_dir.path())
        .arg(&markdown_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Rust Override"))
        .stdout(predicate::str::contains("│ JSON"));
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
        .arg("simple:show-name")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Rust"))
        .stdout(predicate::str::contains("│ Rust\n│ \n│ fn badge()"));
}

#[test]
fn test_code_language_simple_style_plain_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```\nplain text output\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("simple:show-name")
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
