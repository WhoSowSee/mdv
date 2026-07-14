use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

const BASH_SAMPLE: &str = "```bash\nomarchy-theme-install https://github.com/euandeas/omarchy-flexoki-light-theme.git\n```";
const LOCAL_BUILD_SAMPLE: &str =
    "```bash\ncargo build --release\n./target/release/mdv README.md\n```";

const TRUECOLOR_WHITE: &str = "\x1b[38;2;255;255;255m";
const BRIGHT_WHITE: &str = "\x1b[97m";
const ANSI_231: &str = "\x1b[38;5;231m";

#[test]
fn default_terminal_theme_uses_default_foreground_in_bash_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, BASH_SAMPLE).unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg(temp_file.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(!stdout.contains(TRUECOLOR_WHITE));
    assert!(!stdout.contains(BRIGHT_WHITE));
    assert!(!stdout.contains(ANSI_231));
    assert!(stdout.contains("\x1b[39m"));
}

#[test]
fn default_terminal_theme_uses_default_foreground_in_readme_build_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, LOCAL_BUILD_SAMPLE).unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg(temp_file.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(!stdout.contains(TRUECOLOR_WHITE));
    assert!(!stdout.contains(BRIGHT_WHITE));
    assert!(!stdout.contains(ANSI_231));
    assert!(stdout.contains("\x1b[39m"));
}

#[test]
fn code_theme_null_follows_terminal_theme_for_syntax_palette() {
    let config_dir = TempDir::new().unwrap();
    let config_path = config_dir.path().join("config.yaml");
    fs::write(&config_path, "theme: \"terminal\"\ncode_theme: null\n").unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, BASH_SAMPLE).unwrap();

    let output = mdv_cmd()
        .arg("--config-file")
        .arg(config_dir.path())
        .arg(temp_file.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(!stdout.contains(TRUECOLOR_WHITE));
    assert!(!stdout.contains(BRIGHT_WHITE));
    assert!(stdout.contains("\x1b[39m"));
}

#[test]
fn monokai_code_theme_still_uses_truecolor() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn main() {}\n```").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-config")
        .arg("--code-theme")
        .arg("monokai")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;"));
}
