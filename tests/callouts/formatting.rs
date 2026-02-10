use super::*;

#[test]
fn test_callout_pretty_style_renders_frame() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout pretty style");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.contains("╭") && line.contains("Info")),
        "expected pretty top border with label, stdout:\n{}",
        stdout
    );
    assert!(
        lines.iter().any(|line| *line == "│ Example text │"),
        "expected callout body inside frame, stdout:\n{}",
        stdout
    );
    assert!(
        lines.iter().any(|line| line.contains("╰")),
        "expected pretty bottom border, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("[!info]"),
        "expected callout marker to be hidden, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_style_keeps_padding_for_plain_text() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info]\n> This is a long line that should wrap inside the callout box.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("40")
        .arg("-W")
        .arg("word")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout padding");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let content_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| line.starts_with('│') && line.ends_with('│'))
        .collect();

    assert!(
        !content_lines.is_empty(),
        "expected content lines inside pretty callout, stdout:\n{}",
        stdout
    );

    for line in content_lines {
        assert!(
            line.starts_with("│ "),
            "expected left padding inside pretty callout, stdout:\n{}",
            stdout
        );
        assert!(
            line.ends_with(" │"),
            "expected right padding inside pretty callout, stdout:\n{}",
            stdout
        );
    }
}

#[test]
fn test_callout_pretty_style_keeps_padding_when_wrapping_for_frame() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info]\n> ThisIsAVeryLongUnbrokenLineThatShouldWrapInsideTheCalloutFrame\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("30")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout frame wrapping");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let content_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| line.starts_with('│') && line.ends_with('│'))
        .collect();

    assert!(
        !content_lines.is_empty(),
        "expected content lines inside pretty callout, stdout:\n{}",
        stdout
    );

    for line in content_lines {
        assert!(
            line.starts_with("│ "),
            "expected left padding inside pretty callout, stdout:\n{}",
            stdout
        );
        assert!(
            line.ends_with(" │"),
            "expected right padding inside pretty callout, stdout:\n{}",
            stdout
        );
    }
}

#[test]
fn test_callout_pretty_style_preserves_heading_content_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note]\n>\n> ### Требования\n> - Установленный Rust\n> - Терминал с поддержкой ANSI-цветов\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout heading content indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let heading_line = lines
        .iter()
        .find(|line| line.contains("Требования"))
        .expect("heading line present");
    let first_item_line = lines
        .iter()
        .find(|line| line.contains("- Установленный Rust"))
        .expect("list item line present");

    let heading_indent = spaces_after_prefix(heading_line, '│');
    let item_indent = spaces_after_prefix(first_item_line, '│');

    assert!(
        item_indent == heading_indent + 1,
        "expected list content to be indented relative to heading, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_simple_horizontal_rule_stays_inside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> Before\n> ***\n> After\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
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
        .arg("-A")
        .arg("-W")
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
        .arg("-A")
        .arg("-W")
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
        .arg("-A")
        .arg("-W")
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

#[test]
fn test_callout_setext_heading_simple_keeps_single_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> ---\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout setext heading prefix");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines() {
        if line.starts_with('│') {
            panic!(
                "unexpected non-callout prefix in simple setext heading, stdout:\n{}",
                stdout
            );
        }
    }
}

#[test]
fn test_callout_setext_h1_simple_keeps_single_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> =\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout setext h1 prefix");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines() {
        if line.starts_with('│') {
            panic!(
                "unexpected non-callout prefix in simple setext h1, stdout:\n{}",
                stdout
            );
        }
    }
}

#[test]
fn test_callout_setext_heading_pretty_has_no_inner_pipe() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> ---\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty setext heading pipe check");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines().filter(|line| line.starts_with('│')) {
        assert!(
            !line.contains('┃'),
            "expected no callout pipe inside pretty frame, stdout:\n{}",
            stdout
        );
    }

    let heading_line = stdout
        .lines()
        .find(|line| line.contains("dadas"))
        .expect("heading line present");
    let indent = spaces_after_prefix(heading_line, '│');
    assert_eq!(
        indent, 1,
        "expected single left padding for setext heading, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_setext_h1_pretty_has_no_inner_pipe() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> =\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty setext h1 pipe check");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines().filter(|line| line.starts_with('│')) {
        assert!(
            !line.contains('┃'),
            "expected no callout pipe inside pretty frame, stdout:\n{}",
            stdout
        );
    }

    let heading_line = stdout
        .lines()
        .find(|line| line.contains("dadas"))
        .expect("heading line present");
    let indent = spaces_after_prefix(heading_line, '│');
    assert_eq!(
        indent, 1,
        "expected single left padding for setext h1, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_headings_do_not_affect_global_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> # Inside\n>\n## Outside\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--heading-layout")
        .arg("level")
        .arg("--smart-indent")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout smart indent isolation");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    let outside_line = stdout
        .lines()
        .find(|line| line.contains("Outside"))
        .expect("outside heading line present");
    assert!(
        !outside_line.starts_with(' '),
        "expected outside heading to be flush-left under smart-indent, stdout:\n{}",
        stdout
    );
}
