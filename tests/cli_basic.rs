use assert_cmd::Command;
use mdv::utils::strip_ansi;
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

#[test]
fn test_help_command() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("terminal-based markdown viewer"));
}

#[test]
fn test_version_command() {
    let mut cmd = mdv_cmd();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("mdv"));
}

#[test]
fn test_basic_markdown_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Hello World\n\nThis is **bold** text.").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_stdin_input() {
    let mut cmd = mdv_cmd();
    cmd.arg("-");
    cmd.write_stdin("# Test\n\nFrom stdin");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test"));
}

#[test]
fn test_stdin_input_with_bom() {
    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert().success();
    // Note: We can't easily test for absence of ANSI codes in integration tests
}

#[test]
fn test_theme_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Theme Test\n\nTesting themes.").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-t").arg("monokai").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Theme Test"));
}

#[test]
fn test_comments_rendered_by_default() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "<!-- note -->\n\nVisible text\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<!-- note -->"))
        .stdout(predicate::str::contains("Visible text"));
}

#[test]
fn test_comments_wrap_to_column_width() {
    for wrap_mode in ["char", "word"] {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(
            &temp_file,
            "# Title\n<!-- This file demonstrates a wide variety of Markdown capabilities, including formatting, tables, links, media, and references. -->\n",
        )
        .unwrap();

        let output = mdv_cmd()
            .arg("-A")
            .arg("-c")
            .arg("40")
            .arg("-W")
            .arg(wrap_mode)
            .arg(temp_file.path())
            .output()
            .unwrap();
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();
        let clean = strip_ansi(&stdout);
        assert!(clean.contains("<!-- This file"), "stdout:\n{}", stdout);
        assert!(
            clean.lines().all(|line| line.chars().count() <= 40),
            "wrap_mode={wrap_mode}, stdout:\n{}",
            stdout
        );
    }
}

#[test]
fn test_hide_comments_option_hides_comments() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "<!-- secret -->\n\nVisible text\n").unwrap();

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let output = mdv_cmd()
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
    let mut cmd = mdv_cmd();
    cmd.arg("nonexistent_file.md");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_from_text_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Start\n\nSome content.\n\n## Target Section\n\nThis is the target.\n\n## End\n\nMore content.").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-f").arg("Target Section").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Target Section"));
}

#[test]
fn test_tab_length_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Tab Test\n\n\tIndented with tab").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-b").arg("8").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tab Test"));
}

#[test]
fn test_theme_info_without_file_lists_available_themes() {
    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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
    let config_dir = TempDir::new().unwrap();
    let config_path = config_dir.path().join("config.yaml");
    fs::write(&config_path, "theme_info: true\n").unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Config Theme Info\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--config-file").arg(config_dir.path());
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\nCurrent theme: terminal"))
        .stdout(predicate::str::contains("\nCurrent code theme: terminal"))
        .stdout(predicate::str::contains("Available themes").not());
}

#[test]
fn test_text_highlight_background() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Normal ==highlighted text== end.").unwrap();

    let output = mdv_cmd()
        .arg("-c")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let clean = strip_ansi(&stdout);
    assert!(clean.contains("highlighted text"));
    assert!(!clean.contains("==highlighted text=="));
    assert!(stdout.contains("\u{1b}[48;"));
}

#[test]
fn test_init_config_creates_config_file_from_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&config_path.display().to_string()));

    let config = fs::read_to_string(&config_path).unwrap();
    assert!(config.contains("theme: \"terminal\""));
    assert!(config.contains("link_style: \"clickable\""));
}

#[test]
fn test_init_config_creates_config_file_in_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(".")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}

#[test]
fn test_init_config_creates_config_file_from_config_file_arg() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nested").join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg("--config-file")
        .arg(temp_dir.path().join("nested"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}

#[test]
fn test_init_config_refuses_to_overwrite_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, "theme: \"monokai\"\n").unwrap();

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Config file already exists"),
        "stderr:\n{}",
        stderr
    );
    assert_eq!(
        fs::read_to_string(&config_path).unwrap(),
        "theme: \"monokai\"\n"
    );
}

#[test]
fn test_init_config_uses_env_config_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .env("MDV_CONFIG_PATH", temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&config_path.display().to_string()));
}

#[test]
fn test_init_config_positional_path_overrides_config_file_arg() {
    let temp_dir = TempDir::new().unwrap();
    let config_file_path = temp_dir.path().join("config-file");
    let positional_path = temp_dir.path().join("positional");
    let positional_config = positional_path.join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(&positional_path)
        .arg("--config-file")
        .arg(&config_file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(positional_config.exists());
    assert!(!config_file_path.join("config.yaml").exists());
}

#[test]
fn test_pager_mode_prints_to_stdout_when_output_is_not_a_terminal() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Pager output\n").unwrap();

    let output = mdv_cmd()
        .arg("--pager")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);
    assert!(clean.contains("Pager output"), "stdout:\n{}", stdout);
}

#[test]
fn test_interactive_flag_is_rejected() {
    mdv_cmd()
        .arg("--interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--interactive'",
        ));
}
