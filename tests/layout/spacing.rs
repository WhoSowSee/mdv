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
fn test_soft_break_inside_paragraph_collapses_when_next_line_fits() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Alpha beta\nGamma delta\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for wide soft break");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("Alpha beta Gamma delta\n"),
        "expected soft break to collapse into a space, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Alpha beta\nGamma delta"),
        "expected no preserved soft break when the next line fits, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_soft_break_inside_paragraph_preserves_when_next_line_does_not_fit() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Alpha beta\nGamma delta epsilon\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("26")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for narrow soft break");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("Alpha beta\nGamma delta epsilon\n"),
        "expected source soft break to stay before a non-fitting next line, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Alpha beta Gamma"),
        "expected renderer not to fill the previous line with part of the next line, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_soft_break_inside_paragraph_preserves_short_final_tail_on_full_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Alpha beta gamma delta epsilon zeta eta\ntheta iota kappa\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("58")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for balanced final soft break");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("Alpha beta gamma delta epsilon zeta eta\ntheta iota kappa\n"),
        "expected short final tail to remain on its source line, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Alpha beta gamma delta epsilon zeta eta theta"),
        "expected renderer not to overfill the previous line with a short final tail, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_soft_break_inside_paragraph_preserves_long_single_word_final_tail() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- Keep `docs/architecture.md` aligned with the actual crate graph and\n  responsibilities.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("110")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for single-word final tail");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("actual crate graph and\n  responsibilities."),
        "expected long single-word final tail to remain on its source line, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("actual crate graph and responsibilities."),
        "expected renderer not to glue a long single-word final tail, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_soft_break_inside_paragraph_collapses_single_word_tail_after_wrapped_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- Keep `docs/architecture.md` aligned with the actual crate graph and\n  responsibilities.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("60")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for wrapped single-word final tail");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("graph and responsibilities."),
        "expected single-word tail to join a short wrapped line, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("graph and\n  responsibilities."),
        "expected renderer not to leave a dangling single-word tail after a wrapped source line, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_soft_break_inside_paragraph_collapses_long_fragment_after_short_wrapped_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Alpha beta gamma delta epsilon zeta would\ngrow past that limit, split into focused submodules under a same-name directory.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("word")
        .arg("-c")
        .arg("37")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for long fragment after short wrapped line");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("would grow"),
        "expected long next fragment to join a short wrapped line, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("would\ngrow"),
        "expected renderer not to leave a short wrapped line before a long fragment, stdout:\n{}",
        stdout
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
