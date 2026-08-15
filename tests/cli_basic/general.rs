use super::*;

#[test]
fn test_help_command() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("terminal-based markdown viewer"));
}

#[test]
fn test_help_subcommand_prints_long_help_when_output_is_not_a_terminal() {
    let subcommand = mdv_cmd().arg("help").output().unwrap();
    let help_flag = mdv_cmd().arg("--help").output().unwrap();

    assert!(subcommand.status.success());
    assert!(help_flag.status.success());
    assert_eq!(subcommand.stdout, help_flag.stdout);

    let stdout = String::from_utf8(subcommand.stdout).unwrap();
    assert!(stdout.contains("terminal-based markdown viewer"));
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
            "--pretty-list 'type:nerd-font;size:large'",
        ))
        .stdout(predicate::str::contains(
            "--pretty-list 'type:nerd-font;size:small'",
        ))
        .stdout(predicate::str::contains(
            "--pretty-list 'type:unicode;size:large'",
        ))
        .stdout(predicate::str::contains("--pretty-list 'size:large'"))
        .stdout(predicate::str::contains("--pretty-list 'type:unicode'"))
        .stdout(predicate::str::contains("JetBrainsMono Nerd Font"))
        .stdout(predicate::str::contains("--uniform-list-marker"))
        .stdout(predicate::str::contains("-D, --pretty-definition <STYLE>"))
        .stdout(predicate::str::contains(
            "Unicode definition marker spacing may vary by font",
        ))
        .stdout(predicate::str::contains(
            "Nerd Font definition marker requires a Nerd Font terminal",
        ))
        .stdout(predicate::str::contains("U+F444").not());
}

#[test]
fn custom_checkbox_help_uses_real_icons_in_examples() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Override:  --custom-checkbox ' :󰀦'         replaces the unchecked icon",
        ))
        .stdout(predicate::str::contains(
            "Add:       --custom-checkbox '*:󰞋'         adds a new '[*]' checkbox state",
        ))
        .stdout(predicate::str::contains(
            "Color:     --custom-checkbox ' :󰀦:yellow'  accepts '#ffffff', '128,1,1', 'ansi(200)'",
        ))
        .stdout(predicate::str::contains(
            "Iconless:  --custom-checkbox '?:red'       keeps the [?] icon and applies red",
        ))
        .stdout(predicate::str::contains(
            "           --custom-checkbox '*:yellow'    uses the unchecked icon and applies yellow",
        ))
        .stdout(predicate::str::contains("--custom-checkbox ' :icon'").not())
        .stdout(predicate::str::contains("--custom-checkbox '*:icon'").not());
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
    cmd.arg("--html").arg(temp_file.path());
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
    cmd.arg("--no-colors").arg(temp_file.path());
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
    cmd.arg("--no-colors").arg(temp_file.path());
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
    cmd.arg("--no-colors").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "<div align=\"center\">Centered</div>",
        ))
        .stdout(predicate::str::contains(
            "<span class=\"raw\">inline</span>",
        ));
}
