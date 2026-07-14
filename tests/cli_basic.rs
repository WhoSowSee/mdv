use assert_cmd::Command;
use mdv::utils::strip_ansi;
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

#[test]
fn test_help_command() {
    let mut cmd = mdv_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("terminal-based markdown viewer"));
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

    let mut cmd = mdv_cmd();
    cmd.arg("-A").arg(temp_file.path());
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
    cmd.arg("-A").arg(temp_file.path());
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
    cmd.arg("-A").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "<div align=\"center\">Centered</div>",
        ))
        .stdout(predicate::str::contains(
            "<span class=\"raw\">inline</span>",
        ));
}

#[test]
fn test_render_html_option_formats_raw_html() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "<head><title>Hidden</title></head><div align=\"center\"><strong>Centered</strong></div>\n\n<p>Clip <img src=\"photo.png\" alt=\"photo\"><img src=\"animation.gif\" alt=\"demo gif\"><video src=\"movie.mp4\" title=\"demo video\"></video><video><source src=\"trailer.webm\" title=\"source video\"></video></p>\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("40")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("                Centered"),
        "stdout:\n{}",
        stdout
    );
    assert!(stdout.contains("Clip [IMAGE] photo"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[GIF] demo gif"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[VIDEO] demo video"), "stdout:\n{}", stdout);
    assert!(
        stdout.contains("[VIDEO] source video"),
        "stdout:\n{}",
        stdout
    );
    assert!(!stdout.contains("Hidden"), "stdout:\n{}", stdout);
    assert!(!stdout.contains("<div"), "stdout:\n{}", stdout);
    assert!(!stdout.contains("align=\"center\""), "stdout:\n{}", stdout);
}

#[test]
fn test_render_html_buffers_centered_semantic_blocks() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<section align="center">
  <img src="logo.png" alt="Logo">
</section>

<figure style="text-align:center">
  <a href="https://example.com/one"><img src="one.svg" alt="ONE"></a>
  <a href="https://example.com/two"><img src="two.svg" alt="TWO"></a>
</figure>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("80")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let image_lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.contains("[IMAGE]") || line.contains("[SVG]"))
        .collect();
    assert_eq!(image_lines.len(), 2, "stdout:\n{}", stdout);
    assert!(
        image_lines[0].starts_with("                              "),
        "stdout:\n{}",
        stdout
    );
    assert!(
        image_lines[1].starts_with("                    "),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_render_html_right_aligns_regular_blocks() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<div align="right">Right edge</div>
<section style="text-align:right">
  <span>CSS right</span>
</section>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("40")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout
            .lines()
            .any(|line| line == "                              Right edge"),
        "stdout:\n{}",
        stdout
    );
    assert!(
        stdout
            .lines()
            .any(|line| line == "                               CSS right"),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_render_html_formats_inline_semantic_tags() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<p><code>cargo test</code> <kbd>Ctrl+C</kbd> <samp>ok</samp> <mark>marked</mark> <small>tiny</small> H<sub>2</sub> x<sup>2</sup> <abbr title="HyperText Markup Language">HTML</abbr></p>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("120")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        strip_ansi(&stdout).contains(
            "`cargo test` [Ctrl+C] `ok` marked tiny H_2 x^2 HTML (HyperText Markup Language)"
        ),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_render_html_details_summary_static_output() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<details>
  <summary>Install</summary>
  <p>Run <code>cargo install mdv</code>.</p>
</details>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);
    let summary_pos = clean.find("Install").expect("summary missing");
    let body_pos = clean
        .find("Run `cargo install mdv`.")
        .expect("details body missing");
    assert!(summary_pos < body_pos, "stdout:\n{}", stdout);
}

#[test]
fn test_render_html_preserves_pre_and_textarea_whitespace() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<pre>
pre keeps spaces:
    indented line
        deeper line
</pre>
<textarea>
textarea keeps spaces:
    typed content
</textarea>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg("-c")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(
        clean.contains("pre keeps spaces:\n    indented line\n        deeper line"),
        "stdout:\n{}",
        clean
    );
    assert!(
        clean.contains("textarea keeps spaces:\n    typed content"),
        "stdout:\n{}",
        clean
    );
}

#[test]
fn test_render_html_blockquote_uses_quote_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<blockquote>
  <p>Quote <strong>body</strong></p>
  <p>Second line</p>
</blockquote>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(clean.contains("│ Quote body"), "stdout:\n{}", clean);
    assert!(clean.contains("│ Second line"), "stdout:\n{}", clean);
}

#[test]
fn test_render_html_definition_lists() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<dl>
  <dt>Config</dt>
  <dd>Path to the config file.</dd>
  <dt>Theme</dt>
  <dd><a href="https://example.com/theme">Theme docs</a></dd>
</dl>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(
        clean.contains("Config\n  Path to the config file."),
        "stdout:\n{}",
        clean
    );
    assert!(clean.contains("Theme\n  Theme docs"), "stdout:\n{}", clean);
}

#[test]
fn test_render_html_figure_caption_is_rendered_after_content() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<figure align="center">
  <img src="overview.png" alt="Overview">
  <figcaption>Overview caption</figcaption>
</figure>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg("-c")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    let image_pos = clean
        .find("[IMAGE] Overview")
        .expect("figure image missing");
    let caption_pos = clean.find("Overview caption").expect("figcaption missing");
    assert!(image_pos < caption_pos, "stdout:\n{}", clean);
    assert!(
        clean
            .lines()
            .any(|line| line.trim() == "Overview caption" && line.starts_with(" ")),
        "stdout:\n{}",
        clean
    );
}

#[test]
fn test_render_html_basic_inline_css_styles() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<p><span style="font-weight:bold">bold css</span> <span style="font-style:italic">italic css</span> <span style="text-decoration:line-through">strike css</span> <span style="text-decoration:underline">underlined css</span></p>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-E")
        .arg("-c")
        .arg("120")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);
    assert!(
        clean.contains("bold css italic css strike css underlined css"),
        "stdout:\n{}",
        stdout
    );
    assert!(stdout.contains("\u{1b}[1m"), "stdout:\n{}", stdout);
    assert!(stdout.contains("\u{1b}[3m"), "stdout:\n{}", stdout);
    assert!(stdout.contains("\u{1b}[9m"), "stdout:\n{}", stdout);
    assert!(stdout.contains("\u{1b}[4m"), "stdout:\n{}", stdout);
}

#[test]
fn test_render_html_inline_table_references_inside_html_containers() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<details>
  <summary><a href="https://example.com/details">Details docs</a></summary>
  <p><img src="details.png" alt="Details image"></p>
</details>

<figure>
  <a href="https://example.com/figure"><img src="figure.png" alt="Figure image"></a>
  <figcaption><a href="https://example.com/caption">Caption docs</a></figcaption>
</figure>

<blockquote>
  <p><a href="https://example.com/quote">Quote link</a> <a href="https://example.com/quote-image"><img src="quote.png" alt="Quote image"></a></p>
</blockquote>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg("-u")
        .arg("inlinetable")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    for expected in [
        "Details docs[",
        "[IMAGE] Details image",
        "[IMAGE] Figure image[",
        "Caption docs[",
        "Quote link[",
        "[IMAGE] Quote image[",
        "https://example.com/details",
        "https://example.com/figure",
        "https://example.com/caption",
        "https://example.com/quote",
        "https://example.com/quote-image",
    ] {
        assert!(
            clean.contains(expected),
            "missing {expected}; stdout:\n{}",
            clean
        );
    }
}

#[test]
fn test_render_html_inline_table_references_reset_across_blocks() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"Markdown para with [a link](https://example.com/md-link).

<p align="center">
  © 2026-present <a href="https://example.com/user">User</a>
</p>

<div align="center">
  <a href="https://example.com/license"><img src="badge.svg" alt="LICENSE"></a>
</div>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg("-u")
        .arg("inlinetable")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(
        clean.contains("a link[1]"),
        "expected markdown paragraph link [1]; stdout:\n{}",
        clean
    );
    assert!(
        clean.contains("User[1]"),
        "expected HTML block link [1] after markdown paragraph; stdout:\n{}",
        clean
    );
    assert!(
        clean.contains("[SVG] LICENSE[1]"),
        "expected second HTML block link [1], not sequential [2]; stdout:\n{}",
        clean
    );
    assert!(
        !clean.contains("[2]"),
        "sequential numbering across blocks is a bug; stdout:\n{}",
        clean
    );
}

#[test]
fn test_render_html_ordered_list_attributes() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<ol start="5">
  <li>Five</li>
  <li>Six</li>
</ol>
<ol start="3" reversed>
  <li>Three</li>
  <li>Two</li>
  <li>One</li>
</ol>
<ol type="a">
  <li>Alpha</li>
  <li>Beta</li>
</ol>
<ol type="A" start="27">
  <li>Upper alpha</li>
</ol>
<ol type="i" start="4">
  <li>Lower roman</li>
</ol>
<ol type="I" start="9">
  <li>Upper roman</li>
</ol>
<ol>
  <li value="4">Explicit value</li>
  <li>After explicit value</li>
</ol>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    for expected in [
        "5. Five",
        "6. Six",
        "3. Three",
        "2. Two",
        "1. One",
        "a. Alpha",
        "b. Beta",
        "AA. Upper alpha",
        "iv. Lower roman",
        "IX. Upper roman",
        "4. Explicit value",
        "5. After explicit value",
    ] {
        assert!(
            clean.contains(expected),
            "missing {expected}; stdout:\n{}",
            clean
        );
    }
}

#[test]
fn test_render_html_unordered_list_type_markers() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<ul type="circle"><li>Circle</li></ul>
<ul type="square"><li>Square</li></ul>
<ul type="disc"><li>Disc</li></ul>
<ul><li>Default</li></ul>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(clean.contains("◦ Circle"), "stdout:\n{}", clean);
    assert!(clean.contains("▪ Square"), "stdout:\n{}", clean);
    assert!(clean.contains("• Disc"), "stdout:\n{}", clean);
    assert!(clean.contains("- Default"), "stdout:\n{}", clean);
}

#[test]
fn test_render_html_option_formats_html_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<table>
  <thead>
    <tr>
      <th align="left">Project</th>
      <th style="text-align:center">Status</th>
      <th align="right">Asset</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Alpha</strong></td>
      <td><a href="https://example.com/ready">Ready</a></td>
      <td><img src="logo.gif" alt="Logo"></td>
    </tr>
    <tr>
      <td>Beta</td>
      <td>Blocked</td>
      <td><video src="demo.mp4" title="Demo"></video></td>
    </tr>
  </tbody>
</table>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("80")
        .arg("--render-html")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Project"), "stdout:\n{}", stdout);
    assert!(stdout.contains("Status"), "stdout:\n{}", stdout);
    assert!(stdout.contains("Alpha"), "stdout:\n{}", stdout);
    assert!(stdout.contains("Ready"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[GIF] Logo"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[VIDEO] Demo"), "stdout:\n{}", stdout);
    assert!(stdout.contains("│ Project"), "stdout:\n{}", stdout);
    assert!(stdout.contains("│ Alpha"), "stdout:\n{}", stdout);
    assert!(!stdout.contains("<table"), "stdout:\n{}", stdout);
    assert!(!stdout.contains("<td"), "stdout:\n{}", stdout);
}

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
            .arg("-A")
            .arg("-c")
            .arg("40")
            .arg("-W")
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

    let mut cmd = mdv_cmd();
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

    let mut cmd = mdv_cmd();
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

    let output = mdv_cmd()
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
    cmd.arg("-f").arg("Target Section").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Target Section"));
}

#[test]
fn test_tab_length_option() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Tab Test\n\n\tIndented with tab").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-b").arg("8").arg(temp_file.path());
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
fn test_text_highlight_background() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Normal ==highlighted text== end.").unwrap();

    let output = mdv_cmd()
        .arg("-c")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let clean = strip_ansi(&stdout);
    assert!(clean.contains("highlighted text"));
    assert!(!clean.contains("==highlighted text=="));
    assert!(stdout.contains("\u{1b}[48;"));
}

#[test]
fn test_init_config_creates_config_file_from_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&config_path.display().to_string()));

    let config = fs::read_to_string(&config_path).unwrap();
    assert!(config.contains("theme: \"terminal\""));
    assert!(config.contains("link_style: \"clickable\""));
}

#[test]
fn test_init_config_creates_config_file_in_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(".")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}

#[test]
fn test_init_config_creates_config_file_from_config_file_arg() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nested").join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg("--config-file")
        .arg(temp_dir.path().join("nested"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}

#[test]
fn test_init_config_refuses_to_overwrite_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, "theme: \"monokai\"\n").unwrap();

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Config file already exists"),
        "stderr:\n{}",
        stderr
    );
    assert_eq!(
        fs::read_to_string(&config_path).unwrap(),
        "theme: \"monokai\"\n"
    );
}

#[test]
fn test_init_config_uses_env_config_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .env("MDV_CONFIG_PATH", temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&config_path.display().to_string()));
}

#[test]
fn test_init_config_positional_path_overrides_config_file_arg() {
    let temp_dir = TempDir::new().unwrap();
    let config_file_path = temp_dir.path().join("config-file");
    let positional_path = temp_dir.path().join("positional");
    let positional_config = positional_path.join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(&positional_path)
        .arg("--config-file")
        .arg(&config_file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(positional_config.exists());
    assert!(!config_file_path.join("config.yaml").exists());
}

#[test]
fn test_pager_mode_prints_to_stdout_when_output_is_not_a_terminal() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Pager output\n").unwrap();

    let output = mdv_cmd()
        .arg("--pager")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);
    assert!(clean.contains("Pager output"), "stdout:\n{}", stdout);
}

#[test]
fn test_interactive_flag_is_rejected() {
    mdv_cmd()
        .arg("--interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--interactive'",
        ));
}
