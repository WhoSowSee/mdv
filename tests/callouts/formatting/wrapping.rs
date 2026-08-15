use super::*;

#[test]
fn test_callout_pretty_style_renders_frame() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
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
        lines.contains(&"│ Example text │"),
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
        .arg("--no-colors")
        .arg("-c")
        .arg("40")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-c")
        .arg("30")
        .arg("-w")
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
fn test_callout_pretty_word_wrap_keeps_frame_for_long_unbroken_lines() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        ">[!info]- Информация\n>配置配置配置配置配置配置配置配置配置配置配置配置配置配置配置\n>terminalconfigurationterminalconfigurationterminalconfiguration\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-c")
        .arg("60")
        .arg("-w")
        .arg("word")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout word wrap");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.contains('╭') && line.contains("Информация")),
        "expected pretty top border with label, stdout:\n{}",
        stdout
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with('│') && line.ends_with('│') && line.contains("配置")),
        "expected wrapped body inside pretty frame, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("┃ [Информация]") && !stdout.contains("┃ [Info]"),
        "expected no fallback to raw blockquote callout rendering, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_char_wrap_avoids_single_character_tail_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!warning]\n> On environments without terminfo/`tput` (especially some Windows setups), `pipes` startup may fail\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-c")
        .arg("101")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout single-character tail");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout
        .lines()
        .filter(|line| line.starts_with('│') && line.ends_with('│'))
    {
        let inner = line.trim_start_matches('│').trim_end_matches('│').trim();
        if !inner.is_empty() {
            assert_ne!(
                inner.chars().count(),
                1,
                "expected no single-character tail line in pretty callout, stdout:\n{}",
                stdout
            );
        }
    }
}

#[test]
fn test_callout_pretty_char_wrap_avoids_single_character_tail_with_heading_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Bench\n\n> [!warning]\n> On environments without terminfo/`tput` (especially some Windows setups), `pipes` startup may fail\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-c")
        .arg("101")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout single-character tail with heading indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('│') && line.trim_end().ends_with('│'))
    {
        let trimmed = line.trim_start();
        let inner = trimmed.trim_start_matches('│').trim_end_matches('│').trim();
        if !inner.is_empty() {
            assert_ne!(
                inner.chars().count(),
                1,
                "expected no single-character tail line with heading indent, stdout:\n{}",
                stdout
            );
        }
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
        .arg("--no-colors")
        .arg("-w")
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
