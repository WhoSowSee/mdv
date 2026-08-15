use super::*;

#[test]
fn test_callout_simple_table_stays_inside_callout_gutter() {
    let stdout = render_callout_table("simple", false);

    let header_separator = stdout
        .lines()
        .find(|line| line.contains('┼'))
        .expect("table header separator present");
    let table_row = stdout
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table row present");

    assert!(
        header_separator.starts_with("┃ "),
        "expected table header separator to keep callout gutter, stdout:\n{}",
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

    let separator_without = without_smart
        .lines()
        .find(|line| line.contains('┼'))
        .expect("table header separator without smart indent");
    let separator_with = with_smart
        .lines()
        .find(|line| line.contains('┼'))
        .expect("table header separator with smart indent");
    let row_without = without_smart
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table row without smart indent");
    let row_with = with_smart
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table row with smart indent");

    assert_eq!(
        separator_without, separator_with,
        "expected the same table gutter regardless of --table-smart-indent"
    );
    assert_eq!(
        row_without, row_with,
        "expected the same table row gutter regardless of --table-smart-indent"
    );
}

#[test]
fn test_callout_pretty_table_keeps_frame_and_column_separator() {
    let stdout = render_callout_table("pretty", false);

    let header_line = stdout
        .lines()
        .find(|line| line.contains("A") && line.contains("B") && line.contains('│'))
        .expect("table header line present");
    let data_line = stdout
        .lines()
        .find(|line| line.contains("one") && line.contains("two"))
        .expect("table data line present");

    assert!(
        header_line.trim_start().starts_with("│  A") && header_line.matches('│').count() == 3,
        "expected table header to preserve the callout frame and column separator, stdout:\n{}",
        stdout
    );
    assert!(
        data_line.trim_start().starts_with("│  one") && data_line.matches('│').count() == 3,
        "expected table row to preserve the callout frame and column separator, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_table_references_render_outside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> See [README](README.md)\n").unwrap();

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
