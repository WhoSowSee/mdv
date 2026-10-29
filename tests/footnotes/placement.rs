use super::*;

#[test]
fn footnote_references_stay_inside_table_cells() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| Tool |\n| - |\n| mdv[^a] |\n\n[^a]: Terminal Markdown viewer.\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.args(["--color", "never"])
        .arg("--cols")
        .arg("40")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with table footnote references");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("mdv[^a]"),
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
    fs::write(
        &temp_file,
        "| Tool |\n| - |\n| mdv[^a] |\n\n[^a]: Terminal Markdown viewer.\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.args(["--color", "always", "--cols", "40"])
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with colored table footnote references");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let data_line = stdout
        .lines()
        .find(|line| line.contains("mdv") && line.contains("[^a]"))
        .expect("table data line present");

    let clean_line = strip_ansi(data_line);
    assert!(
        clean_line.contains("mdv[^a]"),
        "stripped table line should keep marker next to text: {}",
        clean_line
    );
    assert!(
        !data_line.contains("mdv[^a]"),
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
    cmd.args(["--color", "never"])
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

#[test]
fn code_footnote_highlighting_uses_unicode_offsets() {
    let output = mdv_cmd()
        .args([
            "--no-config",
            "--theme=catppuccin",
            "--color=always",
            "--color-depth=truecolor",
            "--cols=80",
            "--code-block-style=basic",
            "--custom-theme=link=#010203",
            "--custom-code-theme=comment=#112233",
        ])
        .write_stdin("```lua\n-- тест [^a] foo\n```\n\n[^a]: note\n")
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().find(|line| line.contains("тест")).unwrap();
    assert!(
        line.contains("\x1b[38;2;1;2;3m[^a]\x1b[0m"),
        "footnote color is not applied to the marker: {line:?}"
    );
    assert_eq!(strip_ansi(line).trim(), "-- тест [^a] foo");
    assert!(
        line.contains("\x1b[3m\x1b[38;2;17;34;51m foo"),
        "comment color and italics are not restored: {line:?}"
    );
}
