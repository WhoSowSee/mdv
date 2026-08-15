use super::*;

#[test]
fn test_inline_html_in_heading_indented_table_keeps_single_top_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Section\n\nComplex cell content:\n\n| Feature | Supports |\n| --- | --- |\n| Lists | one<br>two |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for table inline HTML spacing");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let paragraph_idx = lines
        .iter()
        .position(|line| line.trim() == "Complex cell content:")
        .expect("paragraph present");
    let table_idx = lines
        .iter()
        .position(|line| line.contains("Feature") && line.contains("Supports"))
        .expect("table present");

    assert_eq!(
        table_idx - paragraph_idx - 1,
        1,
        "expected one blank line before table, stdout:\n{stdout}"
    );
}
