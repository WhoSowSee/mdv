use super::*;

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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
