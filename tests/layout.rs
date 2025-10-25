use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_blockquote_list_preserves_marker_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> - Ensure blockquoted list items maintain bullet prefixes even when the text spans multiple wrapped lines within the quote.\n> - Confirm subsequent bullet entries keep the same indentation so the rendered output never drops the quote marker.\n",
    )
    .unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("-A")
        .arg("-c")
        .arg("20")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed for blockquoted list");

    assert!(
        output.status.success(),
        "mdv exited with {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_line_ok = lines
        .iter()
        .any(|line| line.starts_with("│ - Ensure"));
    let first_wrap_ok = lines
        .iter()
        .any(|line| line.starts_with("│   ") && line.contains("bullet"));
    let second_line_ok = lines
        .iter()
        .any(|line| line.starts_with("│ - Confirm"));
    let second_wrap_ok = lines
        .iter()
        .any(|line| line.starts_with("│   ") && line.contains("quote"));

    assert!(
        first_line_ok && first_wrap_ok,
        "expected wrapped first bullet with quote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        second_line_ok && second_wrap_ok,
        "expected wrapped second bullet with quote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\n- Ensure"),
        "expected quote prefix to remain on first bullet, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\n- Confirm"),
        "expected quote prefix to remain on second bullet, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_smart_indent_promotes_first_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "## Heading Two\n\nContent\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Heading Two\n"))
        .stdout(predicate::str::contains("\n Content\n"));
}

#[test]
fn test_smart_indent_limits_growth_per_step() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# H1\n\n## H2\n\n###### H6\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n H2\n"))
        .stdout(predicate::str::contains("\n  H6\n"));
}

#[test]
fn test_smart_indent_handles_mixed_levels() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n## Level 2\n\n###### Level 6\n\n#### Level 4\n\n## Level 2 second\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n Level 2\n"))
        .stdout(predicate::str::contains("\n   Level 6\n"))
        .stdout(predicate::str::contains("\n  Level 4\n"))
        .stdout(predicate::str::contains("\n Level 2 second\n"));
}

#[test]
fn test_center_heading_layout_adds_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Centered\n## Another\n\n---\n\nParagraph body\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--heading-layout")
        .arg("center")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n\n◈"))
        .stdout(predicate::str::contains("\nParagraph body"))
        .stdout(predicate::str::contains("\n\n\n").not());
}

#[test]
fn test_single_blank_line_before_heading_after_empty_pretty_code_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```\n```\n\n##\n").unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg("--style-code-block")
        .arg("pretty")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for empty code block");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "##")
        .expect("heading present");

    let mut blank_lines = 0usize;
    let mut idx = heading_idx;
    while idx > 0 {
        idx -= 1;
        if lines[idx].trim().is_empty() {
            blank_lines += 1;
        } else {
            break;
        }
    }

    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before heading, stdout: {}",
        stdout
    );
}

#[test]
fn test_single_blank_line_before_heading_with_surrounding_elements() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "- Item\n-\n\n```\n```\n>\n>\n\n##\n").unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg("--style-code-block")
        .arg("pretty")
        .arg("--wrap")
        .arg("char")
        .arg("-c")
        .arg("74")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with surrounding elements");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "##")
        .expect("heading present");

    let mut blank_lines = 0usize;
    let mut idx = heading_idx;
    while idx > 0 {
        idx -= 1;
        if lines[idx].trim().is_empty() {
            blank_lines += 1;
        } else {
            break;
        }
    }

    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before heading, stdout: {}",
        stdout
    );
}
