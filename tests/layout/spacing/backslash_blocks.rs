use super::*;

#[test]
fn test_task_list_following_text_is_not_indented_without_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- [ ] Draft the summary\n- [x] Approve the summary\nNext steps begin here.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--code-block-style")
        .arg("simple")
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
