use assert_cmd::Command;
use mdv::utils::{display_width, strip_ansi};
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

fn markdown_file(contents: &str) -> NamedTempFile {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, contents).unwrap();
    file
}

fn successful_stdout(output: std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_backticks_cover_every_inline_style_element() {
    let file = markdown_file(
        "*emphasis* **strong** ***strong emphasis*** `code` ~~strike~~ ==highlight==\n\n| value |\n| --- |\n| *table* |\n",
    );
    let output = mdv_cmd()
        .args([
            "--no-config",
            "--no-colors",
            "--inline-style",
            "emphasis:backticks=true;strong:backticks=true;strong_emphasis:backticks=true;code:backticks=false;strikethrough:backticks=true;highlight:backticks=true",
        ])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(stdout.contains("`emphasis` `strong` `strong emphasis` code `strike` `highlight`"));
    assert!(stdout.contains("`table`"));
}

#[test]
fn backticks_are_counted_when_wrapping() {
    let file = markdown_file("1234 *abcdef*\n");
    let output = mdv_cmd()
        .args([
            "--no-config",
            "--no-colors",
            "--cols",
            "10",
            "--wrap",
            "word",
            "--inline-style",
            "emphasis:backticks=true",
        ])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(stdout.lines().all(|line| display_width(line) <= 10));
    assert!(!stdout.lines().any(|line| line.trim() == "`"));
}

#[test]
fn cli_inline_style_controls_ansi_attributes() {
    let file = markdown_file("*styled*\n");
    let output = mdv_cmd()
        .env("MDV_NO_COLOR", "false")
        .args([
            "--no-config",
            "-Z",
            "emphasis:bold=true,italic=false,underline=true,strikethrough=true",
        ])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(stdout.contains("\x1b[1m"));
    assert!(stdout.contains("\x1b[4m"));
    assert!(stdout.contains("\x1b[9m"));
    assert!(!stdout.contains("\x1b[3m"));
}

#[test]
fn cli_inline_style_partially_overrides_config() {
    let config_dir = TempDir::new().unwrap();
    fs::write(
        config_dir.path().join("config.yaml"),
        "inline_style:\n  code:\n    backticks: false\n    bold: true\n",
    )
    .unwrap();
    let file = markdown_file("`value`\n");
    let output = mdv_cmd()
        .env("MDV_NO_COLOR", "false")
        .arg("--config-file")
        .arg(config_dir.path())
        .args(["--inline-style", "code:backticks=true"])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(strip_ansi(&stdout).contains("`value`"));
    assert!(stdout.contains("\x1b[1m"));
}

#[test]
fn preset_inline_style_partially_overrides_config() {
    let config_dir = TempDir::new().unwrap();
    let presets_dir = config_dir.path().join("presets");
    fs::create_dir(&presets_dir).unwrap();
    fs::write(
        config_dir.path().join("config.yaml"),
        "inline_style:\n  code:\n    bold: true\n",
    )
    .unwrap();
    fs::write(
        presets_dir.join("inline.yaml"),
        "name: inline\ninline_style:\n  code:\n    backticks: false\n",
    )
    .unwrap();
    let file = markdown_file("`value`\n");
    let output = mdv_cmd()
        .env("MDV_NO_COLOR", "false")
        .arg("--config-file")
        .arg(config_dir.path())
        .args(["--preset", "inline"])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(strip_ansi(&stdout).contains("value"));
    assert!(!strip_ansi(&stdout).contains("`value`"));
    assert!(stdout.contains("\x1b[1m"));
}

#[test]
fn strong_emphasis_uses_its_own_attributes() {
    let file = markdown_file("***combined***\n");
    let output = mdv_cmd()
        .env("MDV_NO_COLOR", "false")
        .args([
            "--no-config",
            "--inline-style",
            "strong:backticks=true,bold=true;emphasis:italic=true;strong_emphasis:backticks=false,bold=false,italic=false,underline=true",
        ])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(stdout.contains("\x1b[4m"));
    assert!(!stdout.contains("\x1b[1m"));
    assert!(!stdout.contains("\x1b[3m"));
    assert!(!strip_ansi(&stdout).contains("`combined`"));
}

#[test]
fn user_theme_can_set_inline_style_and_colors() {
    let config_dir = TempDir::new().unwrap();
    let themes_dir = config_dir.path().join("themes");
    fs::create_dir(&themes_dir).unwrap();
    fs::write(
        themes_dir.join("inline.yaml"),
        "name: inline\nextends: terminal\ncode_background: \"#010203\"\nhighlight: \"#040506\"\nstrong_emphasis: \"#070809\"\ninline_style:\n  code:\n    backticks: false\n    underline: true\n",
    )
    .unwrap();
    let file = markdown_file("`code` ==mark== ***combined***\n");
    let output = mdv_cmd()
        .env("MDV_NO_COLOR", "false")
        .arg("--config-file")
        .arg(config_dir.path())
        .args(["--theme", "inline"])
        .arg(file.path())
        .output()
        .unwrap();

    let stdout = successful_stdout(output);
    assert!(strip_ansi(&stdout).contains("code mark"));
    assert!(!strip_ansi(&stdout).contains("`code`"));
    assert!(stdout.contains("\x1b[48;2;1;2;3m"));
    assert!(stdout.contains("\x1b[4m"));
    assert!(stdout.contains("\x1b[38;2;4;5;6m"));
    assert!(stdout.contains("\x1b[38;2;7;8;9m"));
}

#[test]
fn invalid_inline_style_property_is_rejected() {
    let file = markdown_file("*text*\n");
    mdv_cmd()
        .args(["--no-config", "--inline-style", "emphasis:blink=true"])
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unknown inline style property 'blink'",
        ));
}
