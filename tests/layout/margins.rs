use super::*;
use mdv::utils::{display_width, strip_ansi};
use tempfile::TempDir;

#[test]
fn horizontal_margins_indent_and_constrain_wrapped_output() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "abcdefghijklmnop\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("12")
        .arg("-m")
        .arg("left:2;right:3")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with horizontal margins");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let content_lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    assert_eq!(content_lines.len(), 3, "stdout:\n{}", stdout);
    assert!(
        content_lines.iter().all(|line| line.starts_with("  ")),
        "stdout:\n{}",
        stdout
    );
    assert!(
        content_lines
            .iter()
            .all(|line| display_width(&strip_ansi(line)) <= 9),
        "stdout:\n{}",
        stdout
    );
    assert_eq!(
        content_lines
            .iter()
            .map(|line| line.trim_start())
            .collect::<String>(),
        "abcdefghijklmnop"
    );
}

#[test]
fn horizontal_margin_accepts_left_only() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "abcdefghijk\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("10")
        .arg("--margin")
        .arg("left:2")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with only a left margin");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let content_lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    assert_eq!(content_lines.len(), 2, "stdout:\n{}", stdout);
    assert!(
        content_lines.iter().all(|line| line.starts_with("  ")),
        "stdout:\n{}",
        stdout
    );
    assert!(
        content_lines
            .iter()
            .all(|line| display_width(&strip_ansi(line)) <= 10),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn horizontal_margins_load_from_config() {
    let config_dir = TempDir::new().unwrap();
    fs::write(
        config_dir.path().join("config.yaml"),
        "margin: \"right:4\"\n",
    )
    .unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "abcdefghijk\n").unwrap();

    let output = mdv_cmd()
        .arg("--config-file")
        .arg(config_dir.path())
        .arg("--no-colors")
        .arg("--cols")
        .arg("10")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with configured horizontal margins");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let content_lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    assert_eq!(content_lines.len(), 2, "stdout:\n{}", stdout);
    assert!(
        content_lines.iter().all(|line| !line.starts_with(' ')),
        "stdout:\n{}",
        stdout
    );
    assert!(
        content_lines
            .iter()
            .all(|line| display_width(&strip_ansi(line)) <= 6),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn horizontal_margins_bound_tables_code_and_callouts() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| Header | Details |\n| --- | --- |\n| value | a deliberately long table cell |\n\n```rust\nfn deliberately_long_name() {}\n```\n\n> [!note] Margin check\n> A deliberately long callout body.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("28")
        .arg("--margin")
        .arg("left:3;right:4")
        .arg("--table-wrap")
        .arg("wrap")
        .arg(temp_file.path())
        .output()
        .expect("mdv renders width-sensitive blocks with horizontal margins");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let content_lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    assert!(
        content_lines.iter().all(|line| line.starts_with("   ")),
        "stdout:\n{}",
        stdout
    );
    assert!(
        content_lines
            .iter()
            .all(|line| display_width(&strip_ansi(line)) <= 24),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn horizontal_margins_reject_an_exhausted_content_width() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "content\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-config")
        .arg("--cols")
        .arg("8")
        .arg("--margin")
        .arg("left:4;right:4")
        .arg(temp_file.path());

    cmd.assert().failure().stderr(predicate::str::contains(
        "Horizontal margins (4 + 4) must be smaller than the output width (8)",
    ));
}
