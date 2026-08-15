use super::*;

#[test]
fn test_backslash_line_creates_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "First line explains the plan.\n\\\nSecond line continues the plan.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--code-block-style")
        .arg("simple")
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
        .arg("--no-colors")
        .arg("-w")
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
