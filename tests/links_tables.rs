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
fn test_table_link_underlines_only_link_text_fragment() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| Col1 | Col2 |\n|------|------|\n| Before [link](https://example.com) after | Plain cell |\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-config")
        .arg("--cols")
        .arg("80")
        .arg("--link-style")
        .arg("clickable")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv for table link underline fragment");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid utf-8");
    let data_line = stdout
        .lines()
        .find(|line| line.contains("Before") && line.contains("Plain cell"))
        .expect("table data line with mixed link text present");

    assert!(
        data_line.contains("Before \u{1b}[4mlink\u{1b}[24m after"),
        "only link fragment should be underlined, got: {}",
        data_line
    );
    assert!(
        !data_line.contains("\u{1b}[4m Before")
            && !data_line.contains("\u{1b}[4mBefore")
            && !data_line.contains("\u{1b}[4m Plain")
            && !data_line.contains("\u{1b}[4mPlain")
            && !data_line.contains("after\u{1b}[24m"),
        "underline should not leak to non-link text, got: {}",
        data_line
    );
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
    cmd.arg("--code-block-style")
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
    cmd.arg("--code-block-style")
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

#[test]
fn test_inline_table_nested_list_uses_single_reference_block_and_monotonic_indices() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- [one](https://example.com/one)\n  - [two](https://example.com/two)\n    - [three](https://example.com/three)\n- [four](https://example.com/four)\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-u")
        .arg("it")
        .arg("--no-colors")
        .arg("--cols")
        .arg("100")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with inline table nested list");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid utf-8");
    assert!(
        stdout.contains("one[1]"),
        "first nested link should be [1], stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("two[2]"),
        "second nested link should be [2], stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("three[3]"),
        "third nested link should be [3], stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("four[4]"),
        "fourth nested link should be [4], stdout:\n{}",
        stdout
    );

    let lines: Vec<&str> = stdout.lines().collect();
    let reference_lines: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('[') {
                return None;
            }

            let rest = &trimmed[1..];
            let close = rest.find(']')?;
            let _index = rest[..close].parse::<usize>().ok()?;
            let suffix = &rest[close + 1..];
            if !suffix.starts_with(" https://example.com/") {
                return None;
            }

            Some((idx, trimmed))
        })
        .collect();

    assert_eq!(
        reference_lines.len(),
        4,
        "expected exactly one 4-line reference block, stdout:\n{}",
        stdout
    );

    let expected = [
        "[1] https://example.com/one",
        "[2] https://example.com/two",
        "[3] https://example.com/three",
        "[4] https://example.com/four",
    ];
    for ((_, actual), expected_line) in reference_lines.iter().zip(expected.iter()) {
        assert_eq!(
            *actual, *expected_line,
            "reference lines should preserve order and numbering, stdout:\n{}",
            stdout
        );
    }

    for pair in reference_lines.windows(2) {
        assert_eq!(
            pair[1].0,
            pair[0].0 + 1,
            "reference lines should be contiguous without split blocks, stdout:\n{}",
            stdout
        );
    }
}

#[test]
fn test_table_smart_indent_uses_heading_content_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n### Section\n\n| Col1 | Col2 |\n|------|------|\n| alpha beta gamma delta | one two three |\n",
    )
    .unwrap();

    let output_without_indent = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("60")
        .arg(temp_file.path())
        .output()
        .expect("run mdv without table smart indent");
    assert!(output_without_indent.status.success());
    let stdout_without_indent =
        String::from_utf8(output_without_indent.stdout).expect("stdout without indent");

    let top_border_without_indent = stdout_without_indent
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border without indent");
    assert!(
        top_border_without_indent.starts_with('╭'),
        "expected flush-left table without flag, stdout:\n{}",
        stdout_without_indent
    );

    let output_with_indent = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("60")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv with table smart indent");
    assert!(output_with_indent.status.success());
    let stdout_with_indent = String::from_utf8(output_with_indent.stdout).expect("stdout utf8");

    let top_border_with_indent = stdout_with_indent
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border with indent");
    assert!(
        top_border_with_indent.starts_with("   ╭"),
        "expected table to use content indent from H3 (3 spaces), stdout:\n{}",
        stdout_with_indent
    );
}

#[test]
fn test_table_smart_indent_reduces_indent_on_narrow_width() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n#### Deep Section\n\n| Col1 | Col2 |\n|------|------|\n| long content in first cell | another long value |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("18")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv with narrow width and table smart indent");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let top_border = stdout
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border present");

    assert!(
        top_border.starts_with("  ╭"),
        "expected adaptive indent to shrink to 2 spaces at 18 cols, stdout:\n{}",
        stdout
    );
    assert!(
        !top_border.starts_with("    ╭"),
        "expected indent to be reduced from base 4 spaces, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_inline_table_references_follow_table_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n### Links\n\n| Col1 | Col2 |\n|---|---|\n| [link-1](https://example.com/one) | [link-2](https://example.com/two) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("70")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for inline table references with smart indent");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let top_border = stdout
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border present");
    let reference_line = stdout
        .lines()
        .find(|line| line.trim_start().starts_with("[1] https://example.com/one"))
        .expect("first reference line present");

    let table_indent = top_border.chars().take_while(|ch| *ch == ' ').count();
    let reference_indent = reference_line.chars().take_while(|ch| *ch == ' ').count();

    assert_eq!(
        reference_indent, table_indent,
        "expected inline table reference block to align with smart-indented table, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_inline_table_reference_marker_keeps_brackets_together_when_wrapped() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| ID | Name | Link | Command | Status |\n|----|------|------|---------|--------|\n| 101 | ingest | [spec](https://example.com/1) | `cargo test -p ingest` | active |\n| 102 | transform | [runbook](https://example.com/2) | `cargo test -p transform` | active |\n| 103 | export | [dashboard](https://example.com/3) | `cargo test -p export` | paused |\n| 104 | notify | [alerts](https://example.com/4) | `cargo test -p notify` | active |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("52")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for wrapped inline-table references");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let dangling_bracket_line = lines
        .iter()
        .any(|line| line.contains("┆ ]") || line.contains("│ ]"));
    assert!(
        !dangling_bracket_line,
        "reference marker must not be split into a dangling `]`, stdout:\n{}",
        stdout
    );

    assert!(
        stdout.contains("┆ [2]") || stdout.contains("runbook[2]"),
        "expected wrapped or inline marker for runbook link, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("┆ [3]") || stdout.contains("dashboard[3]"),
        "expected wrapped or inline marker for dashboard link, stdout:\n{}",
        stdout
    );

    let table_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| {
            line.starts_with('╭')
                || line.starts_with('╰')
                || line.starts_with('│')
                || line.starts_with('├')
                || line.starts_with('╞')
        })
        .collect();

    assert!(!table_lines.is_empty(), "expected rendered table lines");

    let expected_width = table_lines[0].chars().count();
    for line in table_lines {
        assert_eq!(
            line.chars().count(),
            expected_width,
            "table line width changed, layout is broken:\n{}",
            stdout
        );
    }
}
