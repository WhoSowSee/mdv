use super::*;

#[test]
fn test_table_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Table Test\n\n| Col1 | Col2 |\n|------|------|\n| A    | B    |\n| C    | D    |",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Table Test"));
}

#[test]
fn test_default_table_uses_compact_borders() {
    let stdout = render_basic_table(|cmd| {
        cmd.args(["--no-config", "--no-colors", "--cols", "80"]);
    });
    assert!(
        !stdout
            .chars()
            .any(|character| matches!(character, '╭' | '╮' | '╰' | '╯' | '╞' | '╡')),
        "default table must not have an outer border: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.contains('─') && line.contains('┼')),
        "default table must have a compact header separator: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .filter(|line| line.contains('A') || line.contains('C'))
            .all(|line| line.matches('│').count() == 1),
        "default table must only separate columns vertically: {stdout}"
    );
}

#[test]
fn test_pretty_table_flag_restores_full_borders() {
    let stdout = render_basic_table(|cmd| {
        cmd.args([
            "--no-config",
            "--no-colors",
            "--pretty-table",
            "--cols",
            "80",
        ]);
    });
    assert!(
        stdout.contains('╭')
            && stdout.contains('╮')
            && stdout.contains('╰')
            && stdout.contains('╯'),
        "pretty table must have a rounded outer border: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.contains('╞') && line.contains('╪') && line.contains('╡')),
        "pretty table must preserve the full header border: {stdout}"
    );
}

#[test]
fn test_pretty_table_config_restores_full_borders() {
    let config_dir = tempfile::TempDir::new().unwrap();
    fs::write(
        config_dir.path().join("config.yaml"),
        "pretty_table: true\nno_colors: true\ncols: 80\n",
    )
    .unwrap();

    let stdout = render_basic_table(|cmd| {
        cmd.arg("--config-file").arg(config_dir.path());
    });
    assert!(
        stdout.contains('╭')
            && stdout.contains('╮')
            && stdout.contains('╰')
            && stdout.contains('╯'),
        "pretty_table config must enable the rounded outer border: {stdout}"
    );
}

#[test]
fn test_header_only_table_does_not_render_empty_body_separator() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "| Col1 | Col2 |\n|------|------|\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Col1"))
        .stdout(predicate::str::contains("Col2"))
        .stdout(predicate::str::contains("╞").not());
}

fn render_wrapped_table(content: &str, width: &str, text_wrap: &str, table_wrap: &str) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, format!("| Value |\n| --- |\n| {content} |\n")).unwrap();

    let output = mdv_cmd()
        .args([
            "--no-config",
            "--no-colors",
            "--pretty-table",
            "--cols",
            width,
            "--wrap",
            text_wrap,
            "--table-wrap",
            table_wrap,
        ])
        .arg(temp_file.path())
        .output()
        .expect("render table wrap combination");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout utf8")
}

fn table_cell_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter_map(|line| line.strip_prefix("│ ")?.strip_suffix(" │"))
        .map(str::trim_end)
        .filter(|line| *line != "Value")
        .collect()
}

#[test]
fn table_cells_follow_text_wrap_mode_across_table_layout_modes() {
    let cases: [(&str, &str, &[&str]); 5] = [
        ("word", "fit", &["alpha", "beta", "gamma"]),
        ("char", "fit", &["alpha be", "ta gamma"]),
        ("none", "fit", &["alpha be", "ta gamma"]),
        ("char", "wrap", &["alpha be", "ta gamma"]),
        ("none", "none", &["alpha beta gamma"]),
    ];

    for (text_wrap, table_wrap, expected) in cases {
        let output = render_wrapped_table("alpha beta gamma", "12", text_wrap, table_wrap);
        assert_eq!(
            table_cell_lines(&output),
            expected,
            "wrap={text_wrap}, table-wrap={table_wrap}"
        );
    }
}

#[test]
fn character_wrapped_table_reflows_after_boundary_spaces() {
    let output = render_wrapped_table("1234567 abcdefgh", "11", "char", "fit");
    assert_eq!(table_cell_lines(&output), ["1234567", "abcdefg", "h"]);
    assert!(
        output
            .lines()
            .all(|line| mdv::utils::display_width(line) == 11),
        "table width changed after boundary-space reflow:\n{output}"
    );
}
