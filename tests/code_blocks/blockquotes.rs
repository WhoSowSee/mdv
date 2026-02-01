use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_blockquote_code_block_preserves_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> \n> ```python\n> print(\"Hello word\")\n> ```\n>\n>> \n>> ```python\n>> print(\"Hello word\")\n>> ```\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(
        output.status.success(),
        "mdv finished with failure status: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let mut lines = stdout.lines();

    assert_eq!(
        lines.next(),
        Some("│ │ Python"),
        "expected language label line to keep blockquote prefix"
    );
    assert_eq!(
        lines.next(),
        Some("│ │ "),
        "expected spacer between label and code to keep blockquote prefix"
    );
    assert_eq!(
        lines.next(),
        Some("│ │ print(\"Hello word\")"),
        "expected first code block to keep blockquote and border prefixes"
    );
    assert_eq!(
        lines.next(),
        Some("│ "),
        "expected blank line within blockquote to retain blockquote prefix"
    );
    assert_eq!(
        lines.next(),
        Some("││ "),
        "expected nested blockquote spacer line"
    );
    assert_eq!(
        lines.next(),
        Some("││ │ Python"),
        "expected nested language label to keep prefixes"
    );
    assert_eq!(
        lines.next(),
        Some("││ │ "),
        "expected nested spacer between label and code"
    );
    assert_eq!(
        lines.next(),
        Some("││ │ print(\"Hello word\")"),
        "expected nested blockquote code line to keep prefixes"
    );
    assert_eq!(
        lines.next(),
        Some("││"),
        "expected trailing blank line for nested blockquote to keep prefix"
    );
}

#[test]
fn test_markdown_code_block_in_blockquote_has_no_leading_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> ```markdown\n> > Nested reminder\n> ```\n").unwrap();

    let output = mdv_cmd()
        .arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(
        output.status.success(),
        "mdv finished with failure status: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        !stdout.starts_with('\n'),
        "expected no leading blank line, stdout: {}",
        stdout
    );

    let mut lines = stdout.lines();
    assert_eq!(
        lines.next(),
        Some("│ │ Markdown"),
        "expected language label to keep blockquote prefix"
    );
    assert_eq!(
        lines.next(),
        Some("│ │ "),
        "expected spacer between label and content"
    );
    assert_eq!(
        lines.next(),
        Some("│ │ │ Nested reminder"),
        "expected blockquote and code block prefixes with content"
    );
}

#[test]
fn test_pretty_style_consecutive_code_blocks_in_blockquote_have_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> ```python\n> print(\"hello\")\n> ```\n>\n> ```python\n> print(\"world\")\n> ```\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n│ \n│ ╭"))
        .stdout(predicate::str::contains("\n│ \n│ \n│ ╭").not());
}
