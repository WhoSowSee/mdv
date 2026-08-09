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
fn test_paragraphs_preserve_source_lines_and_use_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Text 1\nLong Text 2\n\nLong text 3\n\n\nLong text 4\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .expect("mdv renders paragraphs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        stdout,
        "Text 1\nLong Text 2\n\nLong text 3\n\nLong text 4\n"
    );
}

#[test]
fn test_reflow_collapses_soft_break_that_would_otherwise_be_preserved() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Alpha beta\nGamma delta epsilon\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("--reflow")
        .arg("-c")
        .arg("26")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for reflow soft break");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("Alpha beta Gamma delta"),
        "expected reflow to collapse the soft break and refill the line, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Alpha beta\nGamma delta epsilon"),
        "expected reflow not to preserve the source soft break, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_reflow_preserves_hard_breaks() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Line one\\\nLine two\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("--reflow")
        .arg("-c")
        .arg("40")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for reflow hard break");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        !stdout.contains("Line one Line two"),
        "expected reflow to keep hard breaks on separate lines, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_block_elements_have_one_blank_line_on_both_sides_without_stacking() {
    let cases = [
        (
            "level heading",
            "BEFORE\n\n# HEADING\n\nAFTER\n",
            &["--heading-layout", "level"] as &[&str],
        ),
        (
            "flat heading",
            "BEFORE\n\n# HEADING\n\nAFTER\n",
            &["--heading-layout", "flat"],
        ),
        (
            "hidden-layout heading",
            "BEFORE\n\n# HEADING\n\nAFTER\n",
            &["--heading-layout", "none"],
        ),
        (
            "centered heading",
            "BEFORE\n\n# HEADING\n\nAFTER\n",
            &["--heading-layout", "center"],
        ),
        ("table", "BEFORE\n\n| H |\n| - |\n| CELL |\n\nAFTER\n", &[]),
        ("horizontal rule", "BEFORE\n\n---\n\nAFTER\n", &[]),
        (
            "unordered list",
            "BEFORE\n\n- UNORDERED\n- SECOND\n\nAFTER\n",
            &[],
        ),
        (
            "ordered list",
            "BEFORE\n\n1. ORDERED\n2. SECOND\n\nAFTER\n",
            &[],
        ),
        ("blockquote", "BEFORE\n\n> QUOTED\n\nAFTER\n", &[]),
    ];

    for (name, markdown, extra_args) in cases {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, markdown).unwrap();

        let output = mdv_cmd()
            .arg("--no-config")
            .arg("--no-colors")
            .args(extra_args)
            .arg(temp_file.path())
            .output()
            .expect("mdv runs for symmetric block spacing");
        assert!(output.status.success(), "{name}: mdv execution failed");

        let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
        let lines: Vec<&str> = stdout.lines().collect();
        let before_idx = lines
            .iter()
            .position(|line| line.trim() == "BEFORE")
            .unwrap_or_else(|| panic!("{name}: BEFORE missing, stdout:\n{stdout}"));
        let after_idx = lines
            .iter()
            .position(|line| line.trim() == "AFTER")
            .unwrap_or_else(|| panic!("{name}: AFTER missing, stdout:\n{stdout}"));
        let visible_lines: Vec<usize> = (before_idx + 1..after_idx)
            .filter(|idx| !lines[*idx].trim().is_empty())
            .collect();
        let first_visible = *visible_lines
            .first()
            .unwrap_or_else(|| panic!("{name}: block output missing, stdout:\n{stdout}"));
        let last_visible = *visible_lines
            .last()
            .unwrap_or_else(|| panic!("{name}: block output missing, stdout:\n{stdout}"));

        assert_eq!(
            first_visible,
            before_idx + 2,
            "{name}: expected one blank line before block, stdout:\n{stdout}"
        );
        assert_eq!(
            after_idx,
            last_visible + 2,
            "{name}: expected one blank line after block, stdout:\n{stdout}"
        );
    }

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "BEFORE\n\n# HEADING\n\n| H |\n| - |\n| CELL |\n\n---\n\n- LIST\n\n> QUOTED\n\nAFTER\n",
    )
    .unwrap();
    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--heading-layout")
        .arg("center")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for adjacent block spacing");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        !stdout.contains("\n\n\n"),
        "adjacent block spacing must not stack, stdout:\n{stdout}"
    );
}

#[test]
fn test_hidden_empty_list_does_not_leave_block_spacing() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "BEFORE\n\n-\n\nAFTER\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for hidden empty list");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        stdout, "BEFORE\n\nAFTER\n",
        "hidden empty list must not add spacing beyond the paragraph gap"
    );
}

#[test]
fn test_lists_at_document_start_do_not_add_leading_blank_line() {
    let cases = [
        ("unordered", "- FIRST\n- SECOND\n"),
        ("ordered", "1. FIRST\n2. SECOND\n"),
    ];

    for (name, markdown) in cases {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, markdown).unwrap();

        let output = mdv_cmd()
            .arg("--no-config")
            .arg("--no-colors")
            .arg(temp_file.path())
            .output()
            .expect("mdv runs for a list at document start");
        assert!(output.status.success(), "{name}: mdv execution failed");

        let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
        assert_eq!(
            stdout, markdown,
            "{name}: list at document start must not have a leading blank line"
        );
    }
}
