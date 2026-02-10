use super::*;

#[test]
fn test_table_smart_indent_uses_heading_content_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n### Section\n\n| Col1 | Col2 |\n|------|------|\n| alpha beta gamma delta | one two three |\n",
    )
    .unwrap();

    let output_without_indent = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("60")
        .arg(temp_file.path())
        .output()
        .expect("run mdv without table smart indent");
    assert!(output_without_indent.status.success());
    let stdout_without_indent =
        String::from_utf8(output_without_indent.stdout).expect("stdout without indent");

    let top_border_without_indent = stdout_without_indent
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border without indent");
    assert!(
        top_border_without_indent.starts_with('╭'),
        "expected flush-left table without flag, stdout:\n{}",
        stdout_without_indent
    );

    let output_with_indent = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("60")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv with table smart indent");
    assert!(output_with_indent.status.success());
    let stdout_with_indent = String::from_utf8(output_with_indent.stdout).expect("stdout utf8");

    let top_border_with_indent = stdout_with_indent
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border with indent");
    assert!(
        top_border_with_indent.starts_with("   ╭"),
        "expected table to use content indent from H3 (3 spaces), stdout:\n{}",
        stdout_with_indent
    );
}

#[test]
fn test_table_smart_indent_reduces_indent_on_narrow_width() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n#### Deep Section\n\n| Col1 | Col2 |\n|------|------|\n| long content in first cell | another long value |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("18")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv with narrow width and table smart indent");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let top_border = stdout
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border present");

    assert!(
        top_border.starts_with("  ╭"),
        "expected adaptive indent to shrink to 2 spaces at 18 cols, stdout:\n{}",
        stdout
    );
    assert!(
        !top_border.starts_with("    ╭"),
        "expected indent to be reduced from base 4 spaces, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_inline_table_references_follow_table_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n### Links\n\n| Col1 | Col2 |\n|---|---|\n| [link-1](https://example.com/one) | [link-2](https://example.com/two) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("70")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for inline table references with smart indent");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let top_border = stdout
        .lines()
        .find(|line| line.contains('╭'))
        .expect("table top border present");
    let reference_line = stdout
        .lines()
        .find(|line| line.trim_start().starts_with("[1] https://example.com/one"))
        .expect("first reference line present");

    let table_indent = top_border.chars().take_while(|ch| *ch == ' ').count();
    let reference_indent = reference_line.chars().take_while(|ch| *ch == ' ').count();

    assert_eq!(
        reference_indent, table_indent,
        "expected inline table reference block to align with smart-indented table, stdout:\n{}",
        stdout
    );
}
