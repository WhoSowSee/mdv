use super::*;

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
            .arg("--no-colors")
            .arg("-c")
            .arg("40")
            .arg("-w")
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
    cmd.arg("--hide-comments")
        .arg("--no-colors")
        .arg(temp_file.path());
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
fn test_word_wrap_splits_unbroken_text_to_column_width() {
    let token = format!("{}.{}", "a".repeat(40), "b".repeat(40));
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, &token).unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--wrap")
        .arg("word")
        .arg("--cols")
        .arg("40")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(
        clean.lines().all(|line| display_width(line) <= 40),
        "word-wrapped line exceeds width: {clean:?}"
    );
    assert_eq!(clean.lines().collect::<String>(), token);
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
    cmd.arg("--no-colors")
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
        .arg("--no-colors")
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
    cmd.arg("--from")
        .arg("Target Section")
        .arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Target Section"));
}

#[test]
fn test_tab_length_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Tab Test\n\n\tIndented with tab").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--tab-length").arg("8").arg(temp_file.path());
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
fn test_preset_info_lists_builtin_and_user_presets() {
    let config_dir = TempDir::new().unwrap();
    let presets_dir = config_dir.path().join("presets");
    fs::create_dir(&presets_dir).unwrap();
    fs::write(
        presets_dir.join("custom.yaml"),
        "name: custom\ntheme: monokai\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd_with_config(&config_dir);
    cmd.arg("--preset-info");
    cmd.assert().success().stdout(concat!(
        "Available presets:\n\n",
        "  compact              - built-in\n",
        "  custom               - custom\n",
        "  reader               - built-in\n",
        "  showcase             - built-in\n",
    ));
}

#[test]
fn test_preset_info_with_file_shows_active_preset() {
    let config_dir = TempDir::new().unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "preset info content").unwrap();

    let mut cmd = mdv_cmd_with_config(&config_dir);
    cmd.arg("--preset-info")
        .arg(temp_file.path())
        .arg("-P")
        .arg("reader");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Current preset: reader"))
        .stdout(predicate::str::contains("preset info content"))
        .stdout(predicate::str::contains("Available presets:").not());
}

#[test]
fn test_preset_info_with_file_without_preset_is_ignored() {
    let config_dir = TempDir::new().unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "plain preset info content").unwrap();

    let mut cmd = mdv_cmd_with_config(&config_dir);
    cmd.arg("--preset-info").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("plain preset info content"))
        .stdout(predicate::str::contains("Current preset:").not())
        .stdout(predicate::str::contains("Available presets:").not());
}
