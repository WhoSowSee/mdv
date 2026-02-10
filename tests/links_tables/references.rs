use super::*;

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
fn test_inline_table_references_stay_inside_blockquote_table() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> Quote intro before links table.\n>\n> | Provider | Console |\n> | --- | --- |\n> | AWS | [console](https://console.aws.amazon.com/) |\n> | Azure | [portal](https://portal.azure.com/) |\n>\n> Quote outro after links table.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--link-style")
        .arg("inlinetable")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for blockquote table inline references");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_reference = lines
        .iter()
        .find(|line| line.contains("[1] https://console.aws.amazon.com/"))
        .expect("first reference line present");
    let first_reference_idx = lines
        .iter()
        .position(|line| line.contains("[1] https://console.aws.amazon.com/"))
        .expect("first reference line index present");
    let second_reference = lines
        .iter()
        .find(|line| line.contains("[2] https://portal.azure.com/"))
        .expect("second reference line present");
    let second_reference_idx = lines
        .iter()
        .position(|line| line.contains("[2] https://portal.azure.com/"))
        .expect("second reference line index present");
    let outro_line = lines
        .iter()
        .find(|line| line.contains("Quote outro after links table."))
        .expect("blockquote outro line present");
    let outro_line_idx = lines
        .iter()
        .position(|line| line.contains("Quote outro after links table."))
        .expect("blockquote outro line index present");

    assert!(
        first_reference
            .trim_start()
            .starts_with("│ [1] https://console.aws.amazon.com/"),
        "expected first reference to keep blockquote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        second_reference
            .trim_start()
            .starts_with("│ [2] https://portal.azure.com/"),
        "expected second reference to keep blockquote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        outro_line
            .trim_start()
            .starts_with("│ Quote outro after links table."),
        "expected quote outro to remain inside blockquote, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line
            .trim_start()
            .starts_with("[1] https://console.aws.amazon.com/")),
        "reference must not break out of blockquote, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line
            .trim_start()
            .starts_with("[2] https://portal.azure.com/")),
        "reference must not break out of blockquote, stdout:\n{}",
        stdout
    );
    assert!(
        first_reference_idx < second_reference_idx
            && second_reference_idx + 1 < outro_line_idx
            && lines[second_reference_idx + 1].trim() == "│",
        "expected an empty blockquote line between references and quote outro, stdout:\n{}",
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
