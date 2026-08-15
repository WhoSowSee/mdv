use super::*;

#[test]
fn test_callout_inline_table_references_increment_and_compact() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note]\n> Paragraph [one](https://example.com/one)\n>\n> - list [two](https://example.com/two)\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout inline table numbering");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        stdout.contains("one[1]"),
        "expected first callout link to be [1], stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("two[2]"),
        "expected second callout link to be [2], stdout:\n{}",
        stdout
    );

    let first_idx = lines
        .iter()
        .position(|line| *line == "[1] https://example.com/one")
        .expect("first reference line present");
    let second_idx = lines
        .iter()
        .position(|line| *line == "[2] https://example.com/two")
        .expect("second reference line present");

    assert_eq!(
        second_idx,
        first_idx + 1,
        "expected reference lines to be consecutive, stdout:\n{}",
        stdout
    );

    assert!(
        !lines.contains(&"┃ [1] https://example.com/one"),
        "expected callout body to not contain [1] reference table line, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.contains(&"┃ [2] https://example.com/two"),
        "expected callout body to not contain [2] reference table line, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_table_inline_table_references_stay_inside_callout() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info]\n> | Field | Value |\n> | --- | --- |\n> | docs | [guide](https://example.com/guide) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout table inline table references");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        stdout.contains("guide[1]"),
        "expected table cell marker inside callout, stdout:\n{}",
        stdout
    );
    assert!(
        lines.iter().any(|line| line
            .trim_start()
            .starts_with("┃ [1] https://example.com/guide")),
        "expected table reference line inside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line
            .trim_start()
            .starts_with("[1] https://example.com/guide")),
        "expected no table reference line outside callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_table_reference_block_has_no_trailing_blank_when_last() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info] Table\n> | Field | Value |\n> | --- | --- |\n> | docs | [guide](https://example.com/guide) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout trailing spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let reference_idx = lines
        .iter()
        .position(|line| line.contains("│ [1] https://example.com/guide"))
        .expect("reference line inside pretty callout");
    let next_line = lines
        .get(reference_idx + 1)
        .copied()
        .unwrap_or_default()
        .trim();

    assert!(
        !next_line.starts_with('│'),
        "expected no empty content row after the last reference line, stdout:\n{}",
        stdout
    );
    assert!(
        next_line.starts_with('╰'),
        "expected callout frame to close right after the last reference line, stdout:\n{}",
        stdout
    );
}
