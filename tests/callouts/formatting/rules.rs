use super::*;

#[test]
fn test_callout_simple_horizontal_rule_stays_inside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> Before\n> ***\n> After\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("-c")
        .arg("20")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout rule simple");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let rule_line = lines
        .iter()
        .find(|line| line.contains('◈'))
        .expect("rule line present");
    let trimmed = rule_line.trim_start_matches('┃').trim_start();
    assert!(
        rule_line.starts_with('┃') && trimmed.starts_with('◈') && trimmed.ends_with('◈'),
        "expected horizontal rule inside simple callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_horizontal_rule_keeps_padding() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> Before\n> ***\n> After\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("-c")
        .arg("20")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout rule pretty");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let rule_line = lines
        .iter()
        .find(|line| line.contains('◈'))
        .expect("rule line present");

    assert!(
        rule_line.starts_with('│') && rule_line.ends_with('│'),
        "expected rule line inside pretty frame, stdout:\n{}",
        stdout
    );

    let inner = rule_line
        .trim_start_matches('│')
        .trim_end_matches('│')
        .trim();
    assert!(
        inner.starts_with('◈') && inner.ends_with('◈'),
        "expected padded rule inside pretty callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_rule_ignores_heading_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!tip]\n> # Требования\n> dadas\n> ***\n> ## Тест\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("-c")
        .arg("30")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout rule heading indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let rule_line = lines
        .iter()
        .find(|line| line.contains('◈'))
        .expect("rule line present");
    let indent = spaces_after_prefix(rule_line, '│');

    assert_eq!(
        indent, 1,
        "expected rule to have only callout padding, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_setext_heading_does_not_render_rule() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note]\n> Title\n> ---\n> Body\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout setext heading");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("Title") && stdout.contains("Body"),
        "expected setext heading and body to render, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("◈") && !stdout.contains("---"),
        "expected no horizontal rule line for setext heading, stdout:\n{}",
        stdout
    );
}
