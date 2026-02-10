use super::*;

#[test]
fn footnote_references_stay_inside_table_cells() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "| A |\n| - |\n| foo[^a] |\n\n[^a]: alpha\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("40")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with table footnote references");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("foo[^a]"),
        "footnote marker should remain in table cell: {}",
        stdout
    );
    assert!(
        !stdout.lines().any(|line| line.trim() == "[^a]"),
        "footnote marker should not be rendered as a standalone line before table: {}",
        stdout
    );
}

#[test]
fn table_footnote_marker_does_not_tint_cell_text() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "| A |\n| - |\n| foo[^a] |\n\n[^a]: alpha\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--cols").arg("40").arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with colored table footnote references");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let data_line = stdout
        .lines()
        .find(|line| line.contains("foo") && line.contains("[^a]"))
        .expect("table data line present");

    let clean_line = strip_ansi(data_line);
    assert!(
        clean_line.contains("foo[^a]"),
        "stripped table line should keep marker next to text: {}",
        clean_line
    );
    assert!(
        !data_line.contains("foo[^a]"),
        "raw colored line should insert style between text and marker: {:?}",
        data_line
    );
}

#[test]
fn footnotes_render_inside_code_blocks() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```rust\nlet x = 1; // note[^code]\n```\n\n[^code]: from code\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("60")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with code footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("[^code] from code"),
        "footnote from code block should render: {}",
        stdout
    );
}
