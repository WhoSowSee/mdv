use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

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

    let output = mdv_cmd()
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

    let first_line_ok = lines.iter().any(|line| line.starts_with("│ - Ensure"));
    let first_wrap_ok = lines
        .iter()
        .any(|line| line.starts_with("│   ") && line.contains("bullet"));
    let second_line_ok = lines.iter().any(|line| line.starts_with("│ - Confirm"));
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
fn test_blockquote_respects_heading_indent_and_single_space() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "## Heading\n\n> Quote text\n").unwrap();

    let output = mdv_cmd()
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for blockquote heading indent");

    assert!(
        output.status.success(),
        "mdv exited with {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let quote_line = lines
        .iter()
        .find(|line| line.contains("Quote text"))
        .expect("quote line present");

    assert!(
        quote_line.starts_with("  │ "),
        "expected heading indent before quote, stdout:\n{}",
        stdout
    );

    let after_prefix = &quote_line["  │ ".len()..];
    assert!(
        !after_prefix.starts_with(' '),
        "expected single space after quote marker, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_smart_indent_promotes_first_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "## Heading Two\n\nContent\n").unwrap();

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--code-block-style")
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

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--code-block-style")
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

#[test]
fn test_backslash_line_creates_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "First line explains the plan.\n\\\nSecond line continues the plan.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for backslash blank line");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| *line == "First line explains the plan.")
        .expect("first line present");
    let second_idx = lines
        .iter()
        .position(|line| *line == "Second line continues the plan.")
        .expect("second line present");

    let gap = &lines[first_idx + 1..second_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();
    let non_blank_lines = gap.iter().filter(|line| !line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 1,
        "expected one blank line between lines, stdout:\n{}",
        stdout
    );
    assert_eq!(
        non_blank_lines, 0,
        "expected no extra content between lines, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_backslash_after_paragraph_gap_keeps_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "The summary ends here.\n\n\\\nThe follow-up starts on the next line.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for backslash after paragraph gap");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| *line == "The summary ends here.")
        .expect("summary line present");
    let second_idx = lines
        .iter()
        .position(|line| *line == "The follow-up starts on the next line.")
        .expect("follow-up line present");

    let gap = &lines[first_idx + 1..second_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();
    let non_blank_lines = gap.iter().filter(|line| !line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 1,
        "expected one blank line between paragraphs, stdout:\n{}",
        stdout
    );
    assert_eq!(
        non_blank_lines, 0,
        "expected no extra content between paragraphs, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_backslash_after_code_block_does_not_stack_blank_lines() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Summary\n\nThis section introduces the example.\n\n```\nExample output line.\n```\n\\\nAfter the snippet, the note continues.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--code-block-style")
        .arg("simple")
        .arg("--no-code-language")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for code block with backslash");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let code_idx = lines
        .iter()
        .position(|line| line.contains("Example output line."))
        .expect("code line present");
    let text_idx = lines
        .iter()
        .position(|line| line.trim() == "After the snippet, the note continues.")
        .expect("follow-up text present");

    let gap = &lines[code_idx + 1..text_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();
    let non_blank_lines = gap.iter().filter(|line| !line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 1,
        "expected one blank line after code block, stdout:\n{}",
        stdout
    );
    assert_eq!(
        non_blank_lines, 0,
        "expected no extra content after code block, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_backslash_after_task_list_resets_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Phase one\n- [ ] Draft the outline\n- [x] Confirm the draft\n- [?] Review the draft\n\\\n\nPhase two\n- [ ] Gather feedback\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for task list with backslash");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let phase_two_idx = lines
        .iter()
        .position(|line| *line == "Phase two")
        .expect("phase two present");
    let last_item_idx = lines
        .iter()
        .rposition(|line| line.contains("Review the draft"))
        .expect("last list item present");

    let gap = &lines[last_item_idx + 1..phase_two_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();
    let non_blank_lines = gap.iter().filter(|line| !line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 1,
        "expected one blank line between list and phase two, stdout:\n{}",
        stdout
    );
    assert_eq!(
        non_blank_lines, 0,
        "expected no extra content between list and phase two, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\\"),
        "expected backslash marker to be removed, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_task_list_following_text_is_not_indented_without_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- [ ] Draft the summary\n- [x] Approve the summary\nNext steps begin here.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for task list termination");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("\nNext steps begin here.\n"),
        "expected next section to be outside the list, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_backslash_end_of_line_before_list_adds_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Section overview\\\n- [ ] Capture requirements\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for trailing backslash before list");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let section_idx = lines
        .iter()
        .position(|line| *line == "Section overview")
        .expect("section line present");
    let list_idx = lines
        .iter()
        .position(|line| line.contains("Capture requirements"))
        .expect("list item present");

    let gap = &lines[section_idx + 1..list_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 1,
        "expected one blank line between section and list, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\\"),
        "expected backslash marker to be removed, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_multiple_backslash_lines_create_multiple_blank_lines() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Alpha line\n\\\n\\\n\\\nBeta line\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for repeated backslash lines");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let alpha_idx = lines
        .iter()
        .position(|line| *line == "Alpha line")
        .expect("alpha line present");
    let beta_idx = lines
        .iter()
        .position(|line| *line == "Beta line")
        .expect("beta line present");

    let gap = &lines[alpha_idx + 1..beta_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 3,
        "expected three blank lines between lines, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_backslash_end_of_line_before_code_block_adds_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Status update\\\n```\nSample output\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--code-block-style")
        .arg("simple")
        .arg("--no-code-language")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for trailing backslash before code block");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let status_idx = lines
        .iter()
        .position(|line| line.trim() == "Status update")
        .expect("status line present");
    let code_idx = lines
        .iter()
        .position(|line| line.contains("Sample output"))
        .expect("code line present");

    let gap = &lines[status_idx + 1..code_idx];
    let blank_lines = gap.iter().filter(|line| line.trim().is_empty()).count();

    assert_eq!(
        blank_lines, 1,
        "expected one blank line before code block, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\\"),
        "expected backslash marker to be removed, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_blockquote_backslash_keeps_prefix_on_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> First\\\n> Second\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for blockquote backslash");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| *line == "│ First")
        .expect("first quote line present");
    let second_idx = lines
        .iter()
        .position(|line| *line == "│ Second")
        .expect("second quote line present");

    let gap = &lines[first_idx + 1..second_idx];
    assert!(
        gap.iter().any(|line| line.trim() == "│"),
        "expected blank line to keep blockquote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        gap.iter().all(|line| !line.trim().is_empty()),
        "expected no unprefixed blank lines inside blockquote, stdout:\n{}",
        stdout
    );
}
