use super::*;

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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
