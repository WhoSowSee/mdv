use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("terminal-based markdown viewer"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("mdv"));
}

#[test]
fn test_basic_markdown_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Hello World\n\nThis is **bold** text.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_stdin_input() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-");
    cmd.write_stdin("# Test\n\nFrom stdin");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test"));
}

#[test]
fn test_stdin_input_with_bom() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-");
    cmd.write_stdin("\u{feff}# Heading\n\nBody text");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("# Heading").not())
        .stdout(predicate::str::contains("Heading"))
        .stdout(predicate::str::contains("\u{feff}").not());
}

#[test]
fn test_html_output() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# HTML Test\n\nThis is a test.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-H").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<h1>"))
        .stdout(predicate::str::contains("HTML Test"));
}

#[test]
fn test_no_colors_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Test\n\n**Bold text**").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert().success();
    // Note: We can't easily test for absence of ANSI codes in integration tests
}

#[test]
fn test_theme_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Theme Test\n\nTesting themes.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-t").arg("monokai").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Theme Test"));
}

#[test]
fn test_comments_rendered_by_default() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "<!-- note -->\n\nVisible text\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<!-- note -->"))
        .stdout(predicate::str::contains("Visible text"));
}

#[test]
fn test_hide_comments_option_hides_comments() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "<!-- secret -->\n\nVisible text\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--hide-comments").arg("-A").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<!-- secret -->").not())
        .stdout(predicate::str::contains("Visible text"));
}

#[test]
fn test_column_width_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Width Test\n\nThis is a long line that should be wrapped according to the specified column width.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-c").arg("40").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Width Test"));
}

#[test]
fn test_word_wrap_list_inline_code_does_not_hang() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- Original: `C:\\Users\\VeryLongFolderNameThatExceedsLimit\\Documents\\Projects\\MyProject`",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A")
        .arg("--wrap")
        .arg("word")
        .arg("-c")
        .arg("75")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("- Original:"))
        .stdout(predicate::str::contains(
            "VeryLongFolderNameThatExceedsLimit",
        ))
        .stdout(predicate::str::contains("\n  `"));
}

#[test]
fn test_reverse_option_preserves_block_layout() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# First Heading\n\nParagraph one continues here.\n\n## Second Heading\n\nParagraph two comes last.",
    )
    .unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("-A")
        .arg("-r")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let second_pos = stdout
        .find("Second Heading")
        .expect("second heading missing in output");
    let first_pos = stdout
        .find("First Heading")
        .expect("first heading missing in output");

    assert!(
        second_pos < first_pos,
        "expected second heading to appear before first heading in reverse output"
    );
    assert!(
        stdout.contains("Paragraph two comes last."),
        "expected concluding paragraph to appear intact"
    );
}

#[test]
fn test_nonexistent_file() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("nonexistent_file.md");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_from_text_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Start\n\nSome content.\n\n## Target Section\n\nThis is the target.\n\n## End\n\nMore content.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-f").arg("Target Section").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Target Section"));
}

#[test]
fn test_tab_length_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Tab Test\n\n\tIndented with tab").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-b").arg("8").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tab Test"));
}

#[test]
fn test_theme_info_without_file_lists_available_themes() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--theme").arg("terminal");
    cmd.arg("--theme-info");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\nCurrent theme: terminal"))
        .stdout(predicate::str::contains("\nCurrent code theme: terminal"))
        .stdout(predicate::str::contains("Available themes:"));
}

#[test]
fn test_theme_info_with_file_outputs_file_contents() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "custom theme info").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--theme").arg("terminal");
    cmd.arg("--theme-info").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\nCurrent theme: terminal"))
        .stdout(predicate::str::contains("\nCurrent code theme: terminal"))
        .stdout(predicate::str::contains("custom theme info"))
        .stdout(predicate::str::contains("Available themes").not());
}

#[test]
fn test_theme_info_from_config_prints_current_theme() {
    let config_file = NamedTempFile::new().unwrap();
    fs::write(config_file.path(), "theme_info: true\n").unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Config Theme Info\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--config-file").arg(config_file.path());
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\nCurrent theme: terminal"))
        .stdout(predicate::str::contains("\nCurrent code theme: terminal"))
        .stdout(predicate::str::contains("Available themes").not());
}
