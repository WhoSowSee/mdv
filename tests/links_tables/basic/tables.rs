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
