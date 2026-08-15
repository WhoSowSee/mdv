use super::*;

#[test]
fn test_render_html_option_formats_raw_html() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "<head><title>Hidden</title></head><div align=\"center\"><strong>Centered</strong></div>\n\n<p>Clip <img src=\"photo.png\" alt=\"photo\"><img src=\"animation.gif\" alt=\"demo gif\"><video src=\"movie.mp4\" title=\"demo video\"></video><video><source src=\"trailer.webm\" title=\"source video\"></video></p>\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        r#"<p><code>cargo test</code> <kbd>Ctrl+C</kbd> <samp>ok</samp> <mark>marked</mark> <small>tiny</small> H<sub>2</sub>O x<sup>2</sup> <abbr title="HyperText Markup Language">HTML</abbr></p>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
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
            "`cargo test` [Ctrl+C] `ok` marked tiny H₂O x² HTML (HyperText Markup Language)"
        ),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_render_html_formats_semantic_tags_inside_markdown_paragraph() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"Prefix <code>cargo test</code> <kbd>Ctrl+C</kbd> <samp>ok</samp> <mark>marked</mark> <small>tiny</small> H<sub>2</sub>O x<sup>2</sup> <abbr title="HyperText Markup Language">HTML</abbr> suffix.
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-c")
        .arg("160")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        strip_ansi(&stdout).contains(
            "Prefix `cargo test` [Ctrl+C] `ok` marked tiny H₂O x² HTML (HyperText Markup Language) suffix."
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(clean.contains("│ Quote body"), "stdout:\n{}", clean);
    assert!(clean.contains("│ Second line"), "stdout:\n{}", clean);
}
