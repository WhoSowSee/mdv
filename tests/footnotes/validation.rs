use super::*;

#[test]
fn missing_footnote_show_renders_placeholder() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```md\nIntro[^missing]\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("show")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with missing footnote");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_count = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .count();
    assert_eq!(
        separator_count, 1,
        "missing footnote should render a block: {}",
        stdout
    );
    assert!(
        stdout.contains("[^missing] Missing footnote definition"),
        "missing footnote placeholder should be rendered: {}",
        stdout
    );
}

#[test]
fn missing_footnote_in_plain_text_renders_placeholder() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Intro[^missing]\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("show")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with missing footnote in plain text");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("[^missing] Missing footnote definition"),
        "missing footnote placeholder should be rendered: {}",
        stdout
    );
}

#[test]
fn missing_footnote_hide_omits_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```md\nIntro[^missing]\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("hide")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with hidden missing footnote");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_count = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .count();
    assert_eq!(
        separator_count, 0,
        "missing footnote block should be omitted: {}",
        stdout
    );
    assert!(
        !stdout.contains("Missing footnote definition"),
        "placeholder should not be rendered when hidden: {}",
        stdout
    );
}

#[test]
fn missing_footnote_hide_omits_all_placeholder_messages() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Intro[^missing]\n\n[^empty]:\n[^invalid]\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("hide")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with hidden placeholders");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_count = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .count();
    assert_eq!(
        separator_count, 0,
        "placeholder footnotes should not render any block: {}",
        stdout
    );
    assert!(
        !stdout.contains("Missing footnote definition"),
        "missing placeholder should be hidden: {}",
        stdout
    );
    assert!(
        !stdout.contains("Invalid footnote syntax"),
        "invalid syntax placeholder should be hidden: {}",
        stdout
    );
    assert!(
        !stdout.contains("Empty footnote content"),
        "empty content placeholder should be hidden: {}",
        stdout
    );
}

#[test]
fn bare_footnote_definition_without_colon_reports_invalid_syntax() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Intro[^a]\n\n[^a]\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("show")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with bare footnote definition");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("Intro[^a]"),
        "reference should remain in text: {}",
        stdout
    );
    assert!(
        !stdout.lines().any(|line| line.trim().eq("[^a]")),
        "bare definition line should be removed from output: {}",
        stdout
    );
    assert!(
        stdout.contains("[^a] Invalid footnote syntax"),
        "invalid syntax message should be rendered: {}",
        stdout
    );
}

#[test]
fn empty_footnote_definition_reports_empty_body() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Intro[^a]\n\n[^a]:\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("show")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with empty footnote definition");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("[^a] Empty footnote content"),
        "empty body message should be rendered: {}",
        stdout
    );
}

#[test]
fn markdown_code_block_definitions_match_normal_behavior() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```md\nIntro[^a]\nIntro2[^b]\n[^a]\n[^b]:\n```\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--missing-footnote-style")
        .arg("show")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with markdown code block footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    assert!(
        stdout.contains("[^a] Invalid footnote syntax"),
        "invalid syntax in markdown code block should be reported: {}",
        stdout
    );
    assert!(
        stdout.contains("[^b] Empty footnote content"),
        "empty body in markdown code block should be reported: {}",
        stdout
    );
    assert!(
        !stdout.contains("Missing footnote definition"),
        "markdown code block definitions should not be treated as missing: {}",
        stdout
    );
}
