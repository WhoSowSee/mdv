use super::*;

#[test]
fn test_help_command() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Render Markdown in the terminal"));
}

#[test]
fn test_help_subcommand_prints_long_help_when_output_is_not_a_terminal() {
    let subcommand = mdv_cmd().arg("help").output().unwrap();
    let help_flag = mdv_cmd().arg("--help").output().unwrap();

    assert!(subcommand.status.success());
    assert!(help_flag.status.success());
    assert_eq!(subcommand.stdout, help_flag.stdout);

    let stdout = String::from_utf8(subcommand.stdout).unwrap();
    assert!(stdout.contains("Render Markdown in the terminal"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--block-spacing <SPACING>"));
}

#[test]
fn test_pretty_marker_help_documents_font_behavior() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("type:<nerd-font|unicode>"))
        .stdout(predicate::str::contains(
            "size option only changes Nerd Font icons",
        ))
        .stdout(predicate::str::contains(
            "--list-style 'type:nerd-font;size:large'",
        ))
        .stdout(predicate::str::contains(
            "--list-style 'type:nerd-font;size:small'",
        ))
        .stdout(predicate::str::contains(
            "--list-style 'type:unicode;size:large'",
        ))
        .stdout(predicate::str::contains("--list-style 'size:large'"))
        .stdout(predicate::str::contains("--list-style 'type:unicode'"))
        .stdout(predicate::str::contains("--uniform-list-marker"))
        .stdout(predicate::str::contains(
            "-D, --definition-marker-style <STYLE>",
        ))
        .stdout(predicate::str::contains(
            "Unicode definition marker spacing may vary by font",
        ))
        .stdout(predicate::str::contains(
            "The nerd-font marker requires a Nerd Font",
        ))
        .stdout(predicate::str::contains("U+F444").not());
}

#[test]
fn custom_checkbox_help_uses_real_icons_in_examples() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--custom-checkbox ' :󰀦'"))
        .stdout(predicate::str::contains("--custom-checkbox '*:󰞋'"))
        .stdout(predicate::str::contains("--custom-checkbox ' :󰀦:yellow'"))
        .stdout(predicate::str::contains("--custom-checkbox '?:red'"))
        .stdout(predicate::str::contains("--custom-checkbox '*:yellow'"))
        .stdout(predicate::str::contains("--custom-checkbox ' :icon'").not())
        .stdout(predicate::str::contains("--custom-checkbox '*:icon'").not());
}

#[test]
fn test_version_command() {
    let directory = TempDir::new().unwrap();
    let expected = format!(
        "mdv\n    Version: {}\n    Debug  : {}\n    Triple : {} ({}-{})\n    Rustc  : {}\n",
        env!("MDV_BUILD_VERSION"),
        cfg!(debug_assertions),
        env!("MDV_BUILD_TARGET"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("MDV_BUILD_RUSTC"),
    );

    for flag in ["--version", "-V"] {
        mdv_cmd()
            .current_dir(directory.path())
            .env("PATH", "")
            .arg(flag)
            .assert()
            .success()
            .stdout(expected.clone())
            .stderr("");
    }
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
    cmd.arg("--html").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<h1>"))
        .stdout(predicate::str::contains("HTML Test"));
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
    cmd.args(["--color", "never"]).arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<!-- note -->"))
        .stdout(predicate::str::contains("Visible text"));
}

#[test]
fn test_raw_html_rendered_as_literal_text() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "<div align=\"center\">Centered</div>\n\nText with <span class=\"raw\">inline</span> HTML.\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.args(["--color", "never"]).arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "<div align=\"center\">Centered</div>",
        ))
        .stdout(predicate::str::contains(
            "<span class=\"raw\">inline</span>",
        ));
}
