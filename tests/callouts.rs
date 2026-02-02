use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

#[test]
fn test_callout_renders_label_and_body() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Info]\n┃ \n┃ Example text\n"),
        "expected callout header and body, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("[!info]"),
        "expected callout marker to be hidden, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_backslash_keeps_blockquote_context() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> Арбуз\\\n> Арбуз\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout backslash");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Important]"),
        "expected callout header, stdout:\n{}",
        stdout
    );

    let arbuz_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("Арбуз"))
        .collect();

    assert!(
        !arbuz_lines.is_empty(),
        "expected callout body lines, stdout:\n{}",
        stdout
    );
    assert!(
        arbuz_lines.iter().all(|line| line.starts_with("┃ ")),
        "expected backslash content to stay inside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("│ Арбуз"),
        "expected no plain blockquote after backslash, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_adds_blank_lines_around() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Alpha\n> [!info]\n> Example text\nOmega\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let alpha_idx = lines
        .iter()
        .position(|line| *line == "Alpha")
        .expect("alpha line present");
    let callout_idx = lines
        .iter()
        .position(|line| *line == "┃ [Info]")
        .expect("callout header present");
    let omega_idx = lines
        .iter()
        .position(|line| *line == "Omega")
        .expect("omega line present");

    let before_callout = &lines[alpha_idx + 1..callout_idx];
    let after_callout = &lines[callout_idx + 3..omega_idx];

    assert_eq!(
        before_callout
            .iter()
            .filter(|line| line.trim().is_empty())
            .count(),
        1,
        "expected one blank line before callout, stdout:\n{}",
        stdout
    );
    assert_eq!(
        after_callout
            .iter()
            .filter(|line| line.trim().is_empty())
            .count(),
        1,
        "expected one blank line after callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_alias_uses_label() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tldr]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout alias");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Tldr]\n┃ \n┃ Example text\n"),
        "expected alias label to render, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_admonition_syntaxes_render() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        ":::note\nAlpha\n:::\n\n:::{note} Title\nBeta\n:::\n\n!!! note Арбуз\nГамма\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for admonition callout syntaxes");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Note]"),
        "expected note callout header, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("┃ [Title]"),
        "expected custom callout label to render, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("┃ [Арбуз]"),
        "expected custom callout label in bang syntax, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Alpha") && stdout.contains("Beta") && stdout.contains("Гамма"),
        "expected callout bodies to render, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains(":::note") && !stdout.contains("!!! note"),
        "expected raw admonition markers to be hidden, stdout:\n{}",
        stdout
    );
}

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

#[test]
fn test_custom_callout_icon_applies() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!custom]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg("--custom-callout")
        .arg("custom:icon=*")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for custom callout icon");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ *  Custom"),
        "expected custom callout icon, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_custom_callout_color_keeps_default_icon_for_builtin() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg("--custom-callout")
        .arg("tip:color=red")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for custom callout color");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃   Tip"),
        "expected default tip icon to remain, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_label_override_keeps_type_icon_over_custom_label() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note] custom\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg("--custom-callout")
        .arg("custom:icon=*")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout label override");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃   custom"),
        "expected type icon to remain for label override, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_label_override_requires_space() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]Myname\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout label spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Info]"),
        "expected default callout label, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("[Myname]"),
        "expected no custom label without space, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Myname"),
        "expected inline text to be ignored, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_label_without_space_does_not_add_extra_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        ">[!info]Информация\n>dsadasasasasasasasasasasasasasasasasasasasasas\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for inline callout label spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let header_idx = lines
        .iter()
        .position(|line| *line == "┃ [Info]")
        .expect("callout header present");
    let spacer_line = lines.get(header_idx + 1).copied().unwrap_or_default();
    let body_line = lines.get(header_idx + 2).copied().unwrap_or_default();

    assert_eq!(
        spacer_line, "┃ ",
        "expected single spacer line after header, stdout:\n{}",
        stdout
    );
    assert!(
        body_line.contains("dsadasa"),
        "expected body to follow spacer line, stdout:\n{}",
        stdout
    );
    let extra_spacers = lines
        .iter()
        .skip(header_idx + 2)
        .take_while(|line| **line == "┃ ")
        .count();
    assert_eq!(
        extra_spacers, 0,
        "expected no extra spacer lines, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Информация"),
        "expected inline label to be ignored, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_fold_icons_show_when_enabled() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info]+\n> Expanded\n\n> [!info]-\n> Collapsed\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons;fold-icons")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout fold icons");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("Info "),
        "expected expanded fold icon, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Info "),
        "expected collapsed fold icon, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_fold_icons_hidden_without_show_icons() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]+\n> Example\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout fold icon visibility");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        !stdout.contains("") && !stdout.contains(""),
        "expected no fold icons without show-icons, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_style_respects_heading_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Heading\n\n> [!note]\n> Base informational callout.\n",
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
        .expect("mdv runs for callout heading indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| *line == "  │ Base informational callout. │"),
        "expected heading indent on callout frame, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_simple_heading_keeps_pipe_alignment() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> ### Heading\n> Body line\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout heading alignment");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let heading_line = lines
        .iter()
        .find(|line| line.contains("Heading"))
        .expect("heading line present");
    let body_line = lines
        .iter()
        .find(|line| line.contains("Body line"))
        .expect("body line present");

    assert!(
        heading_line.starts_with("┃ "),
        "expected callout pipe to be left-aligned for heading, stdout:\n{}",
        stdout
    );
    assert!(
        body_line.starts_with("┃ "),
        "expected callout pipe to be left-aligned for body, stdout:\n{}",
        stdout
    );

    let header_idx = lines
        .iter()
        .position(|line| *line == "┃ [Important]")
        .expect("callout header present");
    let spacer_line = lines
        .get(header_idx + 1)
        .expect("spacer line after callout header");
    let heading_after_spacer = lines
        .get(header_idx + 2)
        .expect("heading line after spacer");

    assert_eq!(
        *spacer_line, "┃ ",
        "expected single prefixed spacer line after header, stdout:\n{}",
        stdout
    );
    assert!(
        heading_after_spacer.contains("Heading"),
        "expected heading immediately after spacer, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_table_references_render_outside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> See [README](README.md)\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout inline table references");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ See README[1]"),
        "expected inline table link text to stay inside callout, stdout:\n{}",
        stdout
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.iter().any(|line| *line == "[1] README.md"),
        "expected reference list to render outside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| *line == "┃ [1] README.md"),
        "expected reference list to render without callout prefix, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_table_references_increment_and_compact() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note]\n> Paragraph [one](https://example.com/one)\n>\n> - list [two](https://example.com/two)\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout inline table numbering");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        stdout.contains("one[1]"),
        "expected first callout link to be [1], stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("two[2]"),
        "expected second callout link to be [2], stdout:\n{}",
        stdout
    );

    let first_idx = lines
        .iter()
        .position(|line| *line == "[1] https://example.com/one")
        .expect("first reference line present");
    let second_idx = lines
        .iter()
        .position(|line| *line == "[2] https://example.com/two")
        .expect("second reference line present");

    assert_eq!(
        second_idx,
        first_idx + 1,
        "expected reference lines to be consecutive, stdout:\n{}",
        stdout
    );
}

fn is_empty_box_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('│') || !trimmed.ends_with('│') {
        return false;
    }

    let inner: String = trimmed
        .chars()
        .skip(1)
        .take(trimmed.chars().count().saturating_sub(2))
        .collect();

    inner.trim().is_empty()
}

#[test]
fn test_callout_heading_has_no_blank_edges_in_pretty_style() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> ### Heading\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout heading spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let top_idx = lines
        .iter()
        .position(|line| line.contains('╭'))
        .expect("top border present");
    let bottom_idx = lines
        .iter()
        .rposition(|line| line.contains('╰'))
        .expect("bottom border present");

    let first_content = lines
        .get(top_idx + 1)
        .expect("line after top border present");
    let last_content = lines
        .get(bottom_idx.saturating_sub(1))
        .expect("line before bottom border present");

    assert!(
        !is_empty_box_line(first_content),
        "expected no empty line after top border, stdout:\n{}",
        stdout
    );
    assert!(
        !is_empty_box_line(last_content),
        "expected no empty line before bottom border, stdout:\n{}",
        stdout
    );
}

fn spaces_after_prefix(line: &str, prefix: char) -> usize {
    let mut chars = line.chars();
    let first = chars.next().expect("line not empty");
    assert_eq!(
        first, prefix,
        "expected prefix '{}' in line: {}",
        prefix, line
    );
    let mut count = 0usize;
    for ch in chars {
        if ch == ' ' {
            count += 1;
        } else {
            break;
        }
    }
    count
}

#[test]
fn test_callout_simple_headings_respect_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note]\n> # H1\n> ### H3\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--heading-layout")
        .arg("level")
        .arg("--smart-indent")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout smart indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let h1_line = lines
        .iter()
        .find(|line| line.contains("H1"))
        .expect("H1 line present");
    let h3_line = lines
        .iter()
        .find(|line| line.contains("H3"))
        .expect("H3 line present");

    let h1_indent = spaces_after_prefix(h1_line, '┃');
    let h3_indent = spaces_after_prefix(h3_line, '┃');

    assert_eq!(
        h3_indent,
        h1_indent + 1,
        "expected smart-indent to compress H3 indent inside callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_headings_respect_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note]\n> # H1\n> ### H3\n").unwrap();

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
        .expect("mdv runs for callout pretty smart indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let h1_line = lines
        .iter()
        .find(|line| line.contains("H1"))
        .expect("H1 line present");
    let h3_line = lines
        .iter()
        .find(|line| line.contains("H3"))
        .expect("H3 line present");

    let h1_indent = spaces_after_prefix(h1_line, '│');
    let h3_indent = spaces_after_prefix(h3_line, '│');

    assert_eq!(
        h3_indent,
        h1_indent + 1,
        "expected smart-indent to compress H3 indent inside pretty callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_spacing_between_callouts_after_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Heading\n\n> [!info]\n> First\n\n> [!info]\n> Second\n",
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
        .expect("mdv runs for pretty callout spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_top = lines
        .iter()
        .position(|line| line.contains('╭'))
        .expect("first callout top border present");
    let first_bottom = lines
        .iter()
        .skip(first_top + 1)
        .position(|line| line.contains('╰'))
        .map(|idx| idx + first_top + 1)
        .expect("first callout bottom border present");
    let second_top = lines
        .iter()
        .skip(first_bottom + 1)
        .position(|line| line.contains('╭'))
        .map(|idx| idx + first_bottom + 1)
        .expect("second callout top border present");

    let between = &lines[first_bottom + 1..second_top];
    let blank_lines = between.iter().filter(|line| line.trim().is_empty()).count();
    let non_blank_lines = between
        .iter()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert_eq!(
        non_blank_lines, 0,
        "expected only blank lines between callouts, stdout:\n{}",
        stdout
    );
    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line between callouts, stdout:\n{}",
        stdout
    );
}
