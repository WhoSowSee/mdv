use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_show_empty_elements_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> \n\n- \n\n```\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--code-block-style")
        .arg("simple")
        .arg(temp_file.path());
    let output = cmd.output().expect("mdv executed without flag");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        visible.is_empty(),
        "expected no visible empty elements, got: {}",
        stdout
    );

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--code-block-style")
        .arg("simple")
        .arg("--show-empty-elements")
        .arg(temp_file.path());
    let output = cmd.output().expect("mdv executed with flag");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        visible.contains(&"│ "),
        "expected blockquote marker, stdout: {}",
        stdout
    );
    assert!(
        visible.contains(&"- "),
        "expected list marker, stdout: {}",
        stdout
    );
    assert!(
        visible.len() >= 2,
        "expected visible lines for empty elements, stdout: {}",
        stdout
    );
    let pipe_lines = visible.iter().filter(|line| line.contains('│')).count();
    assert!(
        pipe_lines >= 2,
        "expected blockquote and code block pipes, stdout: {}",
        stdout
    );
}

#[test]
fn test_empty_table_respects_show_empty_elements_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "| |\n|-|\n| |\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭").not())
        .stdout(predicate::str::contains("╞").not());

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--show-empty-elements")
        .arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭"))
        .stdout(predicate::str::contains("╞"));
}

#[test]
fn test_empty_headings_respect_show_empty_elements_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "#\n\n##\n").unwrap();

    let output_without_flag = mdv_cmd()
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs without flag");
    assert!(output_without_flag.status.success());
    let stdout_without_flag =
        String::from_utf8(output_without_flag.stdout).expect("stdout utf8 without flag");
    let has_visible_markers = stdout_without_flag
        .lines()
        .any(|line| line.trim().starts_with('#'));
    assert!(
        !has_visible_markers,
        "expected empty headings hidden without flag, stdout: {}",
        stdout_without_flag
    );

    let output_with_flag = mdv_cmd()
        .arg("--no-colors")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with flag");
    assert!(output_with_flag.status.success());
    let stdout_with_flag =
        String::from_utf8(output_with_flag.stdout).expect("stdout utf8 with flag");
    let visible_lines: Vec<&str> = stdout_with_flag
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        visible_lines.iter().any(|line| line.trim() == "#"),
        "expected H1 marker visible, stdout: {}",
        stdout_with_flag
    );
    assert!(
        visible_lines.iter().any(|line| line.trim() == "##"),
        "expected H2 marker visible, stdout: {}",
        stdout_with_flag
    );
}

#[test]
fn test_empty_heading_with_content_shows_placeholder_without_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "#\n\nParagraph\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs without flag");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible_lines: Vec<&str> = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    assert!(
        visible_lines.contains(&"#"),
        "expected placeholder heading, stdout: {}",
        stdout
    );
    assert!(
        visible_lines.contains(&"Paragraph"),
        "expected paragraph content, stdout: {}",
        stdout
    );
}

#[test]
fn test_empty_subheading_with_list_content_shows_placeholder() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "##\n- item\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for list content");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible_lines: Vec<&str> = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    assert!(
        visible_lines.contains(&"##"),
        "expected subheading placeholder, stdout: {}",
        stdout
    );
    assert!(
        visible_lines.iter().any(|line| line.starts_with('-')),
        "expected list entry, stdout: {}",
        stdout
    );
}
