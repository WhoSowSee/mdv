use super::*;

#[test]
fn test_callout_simple_table_stays_inside_callout_gutter() {
    let stdout = render_callout_table("simple", false);

    let table_top = stdout
        .lines()
        .find(|line| line.contains('╭') && line.contains('┬'))
        .expect("table top border present");
    let table_row = stdout
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table row present");

    assert!(
        table_top.starts_with("┃ "),
        "expected table top border to keep callout gutter, stdout:\n{}",
        stdout
    );
    assert!(
        table_row.starts_with("┃ "),
        "expected table row to keep callout gutter, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_table_gutter_does_not_depend_on_table_smart_indent() {
    let without_smart = render_callout_table("simple", false);
    let with_smart = render_callout_table("simple", true);

    let top_without = without_smart
        .lines()
        .find(|line| line.contains('╭') && line.contains('┬'))
        .expect("table top without smart indent");
    let top_with = with_smart
        .lines()
        .find(|line| line.contains('╭') && line.contains('┬'))
        .expect("table top with smart indent");
    let row_without = without_smart
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table row without smart indent");
    let row_with = with_smart
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table row with smart indent");

    assert_eq!(
        top_without, top_with,
        "expected the same table gutter regardless of --table-smart-indent"
    );
    assert_eq!(
        row_without, row_with,
        "expected the same table row gutter regardless of --table-smart-indent"
    );
}

#[test]
fn test_callout_pretty_table_keeps_inner_left_border() {
    let stdout = render_callout_table("pretty", false);

    let header_line = stdout
        .lines()
        .find(|line| line.contains("A") && line.contains("B") && line.contains('┆'))
        .expect("table header line present");
    let data_line = stdout
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table data line present");

    assert!(
        header_line.trim_start().starts_with("│ │"),
        "expected table header to preserve inner left border in pretty callout, stdout:\n{}",
        stdout
    );
    assert!(
        data_line.trim_start().starts_with("│ │"),
        "expected table row to preserve inner left border in pretty callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_table_references_render_outside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> See [README](README.md)\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout inline table references");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ See README[1]"),
        "expected inline table link text to stay inside callout, stdout:\n{}",
        stdout
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.contains(&"[1] README.md"),
        "expected reference list to render outside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.contains(&"┃ [1] README.md"),
        "expected no inline-table reference list inside callout body, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_table_references_increment_and_compact() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note]\n> Paragraph [one](https://example.com/one)\n>\n> - list [two](https://example.com/two)\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
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
        .arg("-A")
        .arg("-W")
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
        .arg("-A")
        .arg("-W")
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

#[test]
fn test_callout_pretty_reference_marker_is_not_split_from_url() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info] Dense links in callout\n> | Topic | Link Set |\n> | --- | --- |\n> | Terminal | [osc8](https://iterm2.com/feature-reporting/Hyperlinks_in_Terminal_Emulators.html) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("--cols")
        .arg("60")
        .arg("--wrap")
        .arg("word")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout long reference");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.contains("[1] https://iterm2.com")),
        "expected reference marker and URL start on the same line, stdout:\n{}",
        stdout
    );

    let marker_only_idx = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed == "│ [1] │" || trimmed == "│ [1]  │" || trimmed == "[1]"
    });
    if let Some(idx) = marker_only_idx {
        let next_trimmed = lines.get(idx + 1).copied().unwrap_or_default().trim();
        assert!(
            !next_trimmed.starts_with("https://"),
            "reference marker must not be split into its own line, stdout:\n{}",
            stdout
        );
    }
}

#[test]
fn test_callout_inline_links_render_outside_while_table_links_stay_inside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info] Dense links in callout\n> Testing smart indent together with callout gutters and inline [references](example.com).\n>\n> | Topic | Link Set |\n> | --- | --- |\n> | Terminal | [osc8](https://iterm2.com/feature-reporting/Hyperlinks_in_Terminal_Emulators.html) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("--cols")
        .arg("90")
        .arg("--wrap")
        .arg("word")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for mixed callout links");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        stdout.contains("references[1]"),
        "expected callout inline link marker in body text, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("osc8[1]"),
        "expected table link marker to use table-local numbering, stdout:\n{}",
        stdout
    );
    assert!(
        lines.contains(&"[1] example.com"),
        "expected callout-level reference block outside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line.contains("│ [1] example.com")),
        "callout-level reference must not be rendered inside pretty callout, stdout:\n{}",
        stdout
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("│ [1] https://iterm2.com/feature-reporting/")),
        "expected table reference line to stay inside callout box, stdout:\n{}",
        stdout
    );
}
