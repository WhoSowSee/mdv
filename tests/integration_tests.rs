use assert_cmd::Command;
use mdv::utils::display_width;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("terminal-based markdown viewer"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("mdv"));
}

#[test]
fn test_basic_markdown_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Hello World\n\nThis is **bold** text.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_stdin_input() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-");
    cmd.write_stdin("# Test\n\nFrom stdin");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test"));
}

#[test]
fn test_stdin_input_with_bom() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
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

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-H").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<h1>"))
        .stdout(predicate::str::contains("HTML Test"));
}

#[test]
fn test_no_colors_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Test\n\n**Bold text**").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert().success();
    // Note: We can't easily test for absence of ANSI codes in integration tests
}

#[test]
fn test_theme_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Theme Test\n\nTesting themes.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-t").arg("monokai").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Theme Test"));
}

#[test]
fn test_comments_rendered_by_default() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "<!-- note -->\n\nVisible text\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<!-- note -->"))
        .stdout(predicate::str::contains("Visible text"));
}

#[test]
fn test_hide_comments_option_hides_comments() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "<!-- secret -->\n\nVisible text\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--hide-comments").arg("-A").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<!-- secret -->").not())
        .stdout(predicate::str::contains("Visible text"));
}

#[test]
fn test_column_width_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Width Test\n\nThis is a long line that should be wrapped according to the specified column width.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-c").arg("40").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Width Test"));
}

#[test]
fn test_word_wrap_list_inline_code_does_not_hang() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "- Original: `C:\\Users\\VeryLongFolderNameThatExceedsLimit\\Documents\\Projects\\MyProject`",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A")
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

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("-A")
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
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("nonexistent_file.md");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_code_highlighting() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Code Test\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Code Test"));
}

#[test]
fn test_no_code_guessing_disables_detection_for_unknown_language() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```dasdasdas\nfn main() {}\n```").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--no-code-guessing")
        .arg("--show-code-language")
        .arg("--style-code-block")
        .arg("simple")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dasdasdas"))
        .stdout(predicate::str::contains("Rust").not());
}

#[test]
fn test_code_language_simple_style_named_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```rust\nfn badge() {\n    println!(\"label\");\n}\n```",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--show-code-language")
        .arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Rust"))
        .stdout(predicate::str::contains("│ Rust\n│ \n│ fn badge()"));
}

#[test]
fn test_code_language_simple_style_plain_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```\nplain text output\n```\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--show-code-language")
        .arg("--style-code-block")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Text\n│ \n│ plain text output"));
}

#[test]
fn test_code_language_pretty_style_named_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hello\")\n```\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--show-code-language")
        .arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(
        output.status.success(),
        "mdv exited with status {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(
        stdout.contains("│ print(\"hello\") │"),
        "expected symmetric padding, got:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\n\n│ print(\"hello\")"),
        "should not duplicate blank lines:\n{}",
        stdout
    );

    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        lines.len() >= 3,
        "expected pretty block lines, got {:?}",
        lines
    );

    let top = lines[0];
    let middle = lines[1];
    let bottom = lines[2];

    assert!(top.starts_with("╭─ Python"), "top line: {}", top);
    assert!(top.ends_with('╮'), "top line should end with ╮: {}", top);
    assert!(middle.starts_with("│ "), "middle line: {}", middle);
    assert!(
        middle.ends_with(" │"),
        "middle line should end with space+│: {}",
        middle
    );
    assert!(bottom.starts_with('╰'), "bottom line: {}", bottom);
    assert!(
        bottom.ends_with('╯'),
        "bottom line should end with ╯: {}",
        bottom
    );

    let top_width = display_width(top);
    let middle_width = display_width(middle);
    let bottom_width = display_width(bottom);

    assert_eq!(
        top_width, middle_width,
        "top and middle widths should match ({} vs {}); stdout:\n{}",
        top_width, middle_width, stdout
    );
    assert_eq!(
        top_width, bottom_width,
        "top and bottom widths should match ({} vs {}); stdout:\n{}",
        top_width, bottom_width, stdout
    );
}

#[test]
fn test_pretty_style_empty_code_block_has_right_padding() {
    let temp_file = NamedTempFile::new().unwrap();
    // Empty fenced block; language shown to match the reported case
    fs::write(&temp_file, "# T\n\n```\n```\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("--show-code-language")
        .arg("--show-empty-elements")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    // Find the first pretty frame after the header
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
    // Skip the header line
    let _ = lines.next();
    // Expect pretty top border line with a dash after label
    let top = lines.next().expect("top border");
    assert!(top.contains("╭─ Text ─╮"), "top border: {}", top);
    // Middle line must end with space + right border
    let middle = lines.next().expect("middle line");
    assert!(middle.ends_with(" │"), "middle: {}", middle);
}

#[test]
fn test_pretty_style_empty_block_falls_back_when_too_narrow() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```text\n```\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("--show-code-language")
        .arg("--show-empty-elements")
        .arg("--wrap")
        .arg("word")
        .arg("-c")
        .arg("9")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(stdout.contains("│ Text"), "stdout: {}", stdout);
    assert!(!stdout.contains("╭─ Text"), "stdout: {}", stdout);
}

#[test]
fn test_simple_language_label_wraps_under_char_width() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```elixir\nIO.puts(\"Hello\")\n```\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("--show-code-language")
        .arg("--wrap")
        .arg("char")
        .arg("-c")
        .arg("6")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines() {
        if line.trim_start().starts_with('│') {
            let width = display_width(line);
            assert!(
                width <= 6,
                "code-block line exceeds width: {} ({} cols)",
                line,
                width
            );
        }
    }
}

#[test]
fn test_pretty_style_consecutive_code_blocks_have_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```python\nprint(\"hello\")\n```\n\n```python\nprint(\"world\")\n```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╯\n\n╭"))
        .stdout(predicate::str::contains("╯\n\n\n╭").not());
}

#[test]
fn test_blockquote_code_block_preserves_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> \n> ```python\n> print(\"Hello word\")\n> ```\n>\n>> \n>> ```python\n>> print(\"Hello word\")\n>> ```\n",
    )
    .unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(
        output.status.success(),
        "mdv finished with failure status: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let mut lines = stdout.lines();

    assert_eq!(
        lines.next(),
        Some("│ │ print(\"Hello word\")"),
        "expected first code block to keep blockquote and border prefixes"
    );
    assert_eq!(
        lines.next(),
        Some("│ "),
        "expected blank line within blockquote to retain blockquote prefix"
    );
    assert_eq!(
        lines.next(),
        Some("││ "),
        "expected nested blockquote spacer line"
    );
    assert_eq!(
        lines.next(),
        Some("││ │ print(\"Hello word\")"),
        "expected nested blockquote code line to keep prefixes"
    );
    assert_eq!(
        lines.next(),
        Some("││"),
        "expected trailing blank line for nested blockquote to keep prefix"
    );
}

#[test]
fn test_markdown_code_block_in_blockquote_has_no_leading_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> ```markdown\n> > dsadas\n> ```\n",
    )
    .unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(
        output.status.success(),
        "mdv finished with failure status: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        !stdout.starts_with('\n'),
        "expected no leading blank line, stdout: {}",
        stdout
    );

    let first_line = stdout
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    assert!(
        first_line.contains("│ │ │ dsadas"),
        "expected blockquote and code block prefixes with content, first line: {}",
        first_line
    );
}

#[test]
fn test_pretty_style_consecutive_code_blocks_in_blockquote_have_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> ```python\n> print(\"hello\")\n> ```\n>\n> ```python\n> print(\"world\")\n> ```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n│ \n│ ╭"))
        .stdout(predicate::str::contains("\n│ \n│ \n│ ╭").not());
}

#[test]
fn test_code_block_followed_by_heading_has_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hi\")\n```\n\n# Heading\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n\nHeading"),
        "expected single blank line before heading; stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("\n\n\nHeading"),
        "unexpected double blank line before heading; stdout:\n{}",
        normalized
    );
}

#[test]
fn test_code_block_followed_by_rule_has_single_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hi\")\n```\n\n---\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--style-code-block")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n\n◈"),
        "expected single blank line before rule; stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("\n\n\n◈"),
        "unexpected double blank line before rule; stdout:\n{}",
        normalized
    );
}

#[test]
fn test_blockquote_list_preserves_marker_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> - dsadasddddddddddddsadasasdsah jdashjd hsajd hsajdsahjkdhsajkdhsajh djashdkjsahdjhsadjhas\n> - dsadjkasjdkasdjkasjdklsadjlksajdlksaj kldasjdlksajldkjs\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-A").arg("-c").arg("20").arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n│ - dsadas"))
        .stdout(predicate::str::contains("\n│   ddsadas"))
        .stdout(predicate::str::contains("\n│ - dsadjka"))
        .stdout(predicate::str::contains("\n│   asjdkls"))
        .stdout(predicate::str::contains("\n- dsadas").not())
        .stdout(predicate::str::contains("\n- dsadjka").not());
}
#[test]
fn test_smart_indent_promotes_first_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "## Heading Two\n\nContent\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Heading Two\n"))
        .stdout(predicate::str::contains("\n Content\n"));
}

#[test]
fn test_smart_indent_limits_growth_per_step() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# H1\n\n## H2\n\n###### H6\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n H2\n"))
        .stdout(predicate::str::contains("\n  H6\n"));
}

#[test]
fn test_smart_indent_handles_mixed_levels() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n## Level 2\n\n###### Level 6\n\n#### Level 4\n\n## Level 2 second\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n Level 2\n"))
        .stdout(predicate::str::contains("\n   Level 6\n"))
        .stdout(predicate::str::contains("\n  Level 4\n"))
        .stdout(predicate::str::contains("\n Level 2 second\n"));
}

#[test]
fn test_center_heading_layout_adds_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Centered\n## Another\n\n---\n\nParagraph body\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--heading-layout")
        .arg("center")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n\n◈"))
        .stdout(predicate::str::contains("\nParagraph body"))
        .stdout(predicate::str::contains("\n\n\n").not());
}

#[test]
fn test_table_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Table Test\n\n| Col1 | Col2 |\n|------|------|\n| A    | B    |\n| C    | D    |",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Table Test"));
}

#[test]
fn test_link_styles() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Link Test\n\n[Example](https://example.com)").unwrap();

    // Test inline table style (default)
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-u").arg("it").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test inline style
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-u").arg("i").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test hide style
    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-u").arg("h").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));
}

#[test]
fn test_inline_table_link_style_inside_text_code_block_pretty() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```text\nThis is a [link](https://example.com/example-path)\n```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--show-code-language")
        .arg("--style-code-block")
        .arg("pretty")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭─ Text"))
        .stdout(predicate::str::contains("│ This is a link[1]"))
        .stdout(predicate::str::contains(
            "\n [1] https://example.com/example-path",
        ))
        .stdout(predicate::str::contains("\n│ [1]").not())
        .stdout(predicate::str::contains("[link](").not());
}

#[test]
fn test_inline_table_link_style_inside_text_code_block_simple() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```text\nThis is a [link](https://example.com/example-path)\n```\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--show-code-language")
        .arg("--style-code-block")
        .arg("simple")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Text"))
        .stdout(predicate::str::contains("│ This is a link[1]"))
        .stdout(predicate::str::contains(
            "\n [1] https://example.com/example-path",
        ))
        .stdout(predicate::str::contains("│ [1]").not())
        .stdout(predicate::str::contains("[link](").not());
}

#[test]
fn test_from_text_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Start\n\nSome content.\n\n## Target Section\n\nThis is the target.\n\n## End\n\nMore content.").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-f").arg("Target Section").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Target Section"));
}

#[test]
fn test_tab_length_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Tab Test\n\n\tIndented with tab").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("-b").arg("8").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tab Test"));
}

#[test]
fn test_theme_info_without_file_lists_available_themes() {
    let mut cmd = Command::cargo_bin("mdv").unwrap();
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

    let mut cmd = Command::cargo_bin("mdv").unwrap();
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
    let config_file = NamedTempFile::new().unwrap();
    fs::write(config_file.path(), "theme_info: true\n").unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Config Theme Info\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--config-file").arg(config_file.path());
    cmd.arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\nCurrent theme: terminal"))
        .stdout(predicate::str::contains("\nCurrent code theme: terminal"))
        .stdout(predicate::str::contains("Available themes").not());
}

#[test]
fn test_show_empty_elements_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> \n\n- \n\n```\n```\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--no-colors")
        .arg("--style-code-block")
        .arg("simple")
        .arg(temp_file.path());
    let output = cmd.output().expect("mdv executed without flag");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        visible.is_empty(),
        "expected no visible empty elements, got: {}",
        stdout
    );

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--no-colors")
        .arg("--style-code-block")
        .arg("simple")
        .arg("--show-empty-elements")
        .arg(temp_file.path());
    let output = cmd.output().expect("mdv executed with flag");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        visible.contains(&"│ "),
        "expected blockquote marker, stdout: {}",
        stdout
    );
    assert!(
        visible.contains(&"- "),
        "expected list marker, stdout: {}",
        stdout
    );
    assert!(
        visible.len() >= 2,
        "expected visible lines for empty elements, stdout: {}",
        stdout
    );
    let pipe_lines = visible.iter().filter(|line| line.contains('│')).count();
    assert!(
        pipe_lines >= 2,
        "expected blockquote and code block pipes, stdout: {}",
        stdout
    );
}

#[test]
fn test_empty_table_respects_show_empty_elements_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "| |\n|-|\n| |\n").unwrap();

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--no-colors").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭").not())
        .stdout(predicate::str::contains("╞").not());

    let mut cmd = Command::cargo_bin("mdv").unwrap();
    cmd.arg("--no-colors")
        .arg("--show-empty-elements")
        .arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭"))
        .stdout(predicate::str::contains("╞"));
}

#[test]
fn test_empty_headings_respect_show_empty_elements_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "#\n\n##\n").unwrap();

    let output_without_flag = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs without flag");
    assert!(output_without_flag.status.success());
    let stdout_without_flag =
        String::from_utf8(output_without_flag.stdout).expect("stdout utf8 without flag");
    let has_visible_markers = stdout_without_flag
        .lines()
        .any(|line| line.trim().starts_with('#'));
    assert!(
        !has_visible_markers,
        "expected empty headings hidden without flag, stdout: {}",
        stdout_without_flag
    );

    let output_with_flag = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with flag");
    assert!(output_with_flag.status.success());
    let stdout_with_flag =
        String::from_utf8(output_with_flag.stdout).expect("stdout utf8 with flag");
    let visible_lines: Vec<&str> = stdout_with_flag
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        visible_lines.iter().any(|line| line.trim() == "#"),
        "expected H1 marker visible, stdout: {}",
        stdout_with_flag
    );
    assert!(
        visible_lines.iter().any(|line| line.trim() == "##"),
        "expected H2 marker visible, stdout: {}",
        stdout_with_flag
    );
}

#[test]
fn test_empty_heading_with_content_shows_placeholder_without_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "#\n\nParagraph\n").unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs without flag");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible_lines: Vec<&str> = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    assert!(
        visible_lines.contains(&"#"),
        "expected placeholder heading, stdout: {}",
        stdout
    );
    assert!(
        visible_lines.contains(&"Paragraph"),
        "expected paragraph content, stdout: {}",
        stdout
    );
}

#[test]
fn test_empty_subheading_with_list_content_shows_placeholder() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "##\n- item\n").unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for list content");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible_lines: Vec<&str> = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    assert!(
        visible_lines.contains(&"##"),
        "expected subheading placeholder, stdout: {}",
        stdout
    );
    assert!(
        visible_lines.iter().any(|line| line.starts_with('-')),
        "expected list entry, stdout: {}",
        stdout
    );
}

#[test]
fn test_single_blank_line_before_heading_after_empty_pretty_code_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```\n```\n\n##\n").unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg("--style-code-block")
        .arg("pretty")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for empty code block");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "##")
        .expect("heading present");

    let mut blank_lines = 0usize;
    let mut idx = heading_idx;
    while idx > 0 {
        idx -= 1;
        if lines[idx].trim().is_empty() {
            blank_lines += 1;
        } else {
            break;
        }
    }

    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before heading, stdout: {}",
        stdout
    );
}

#[test]
fn test_single_blank_line_before_heading_with_surrounding_elements() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "- Item\n-\n\n```\n```\n>\n>\n\n##\n").unwrap();

    let output = Command::cargo_bin("mdv")
        .unwrap()
        .arg("--no-colors")
        .arg("--style-code-block")
        .arg("pretty")
        .arg("--wrap")
        .arg("char")
        .arg("-c")
        .arg("74")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with surrounding elements");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "##")
        .expect("heading present");

    let mut blank_lines = 0usize;
    let mut idx = heading_idx;
    while idx > 0 {
        idx -= 1;
        if lines[idx].trim().is_empty() {
            blank_lines += 1;
        } else {
            break;
        }
    }

    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before heading, stdout: {}",
        stdout
    );
}
