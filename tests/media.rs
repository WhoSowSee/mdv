use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

#[test]
fn test_media_markers_match_file_extensions() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "![clip](animation.mkv)\n\n![song](https://example.com/media/track.M4A?raw=1)\n\n![photo](image.avif)\n\n![gif](animation.GIF?raw=1)\n\n![unknown](archive.bin)\n\n![noext](README)\n",
    )
    .unwrap();

    let output = mdv_cmd().arg("-A").arg(temp_file.path()).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("[VIDEO] clip"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[AUDIO] song"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[IMAGE] photo"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[GIF] gif"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[MEDIA] unknown"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[MEDIA] noext"), "stdout:\n{}", stdout);
}

#[test]
fn test_indented_media_in_list_starts_on_next_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "- Audio\n\n\t![track](animation.mp3)\n").unwrap();

    let output = mdv_cmd().arg("-A").arg(temp_file.path()).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("- Audio\n  [AUDIO] track"),
        "stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("- Audio[AUDIO] track"),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_data_uri_media_markers() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "![img](data:image/png;base64,AAAA)\n\n![gif](data:image/gif;base64,AAAA)\n\n![vid](data:video/mp4;base64,AAAA)\n\n![aud](data:audio/ogg;base64,AAAA)\n\n![bin](data:application/octet-stream;base64,AAAA)\n\n![upper](DATA:IMAGE/SVG+XML;BASE64,AAAA)\n",
    )
    .unwrap();

    let output = mdv_cmd().arg("-A").arg(temp_file.path()).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("[IMAGE] img"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[GIF] gif"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[VIDEO] vid"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[AUDIO] aud"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[MEDIA] bin"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[IMAGE] upper"), "stdout:\n{}", stdout);
}

#[test]
fn test_render_html_adjacent_media_markers_use_contextual_spacing() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<p>
<picture>
  <source srcset="photo.avif" type="image/avif">
  <source srcset="photo.webp" type="image/webp">
  <img src="fallback.png" alt="Picture fallback">
</picture>
</p>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("120")
        .arg("--render-html")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("[IMAGE] photo.avif [IMAGE] photo.webp [IMAGE] Picture fallback"),
        "stdout:\n{}",
        stdout
    );
    assert!(!stdout.contains("photo.avif[IMAGE]"), "stdout:\n{}", stdout);
    assert!(!stdout.contains("photo.webp[IMAGE]"), "stdout:\n{}", stdout);
}

#[test]
fn test_render_html_centers_media_blocks_as_single_span() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<p align="center">
  <img src="logo.png" alt="Logo">
</p>

<p align="center">
  <a href="https://example.com/one"><img src="one.svg" alt="ONE"></a>
  <a href="https://example.com/two"><img src="two.svg" alt="TWO"></a>
  <a href="https://example.com/three"><img src="three.svg" alt="THREE"></a>
</p>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("100")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let media_lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.contains("[IMAGE]"))
        .collect();
    assert_eq!(media_lines.len(), 2, "stdout:\n{}", stdout);

    let logo_line = media_lines
        .iter()
        .find(|line| line.contains("[IMAGE] Logo"))
        .expect("centered logo line");
    let badges_line = media_lines
        .iter()
        .find(|line| {
            line.contains("[IMAGE] ONE")
                && line.contains("[IMAGE] TWO")
                && line.contains("[IMAGE] THREE")
        })
        .expect("centered badges line");

    assert!(
        logo_line.starts_with("                              "),
        "stdout:\n{}",
        stdout
    );
    assert!(
        badges_line.starts_with("                  "),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_render_html_inline_table_references_for_html_links_and_media() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<p>
  <a href="https://example.com/docs">Docs</a>
  <a href="https://example.com/badge"><img src="badge.svg" alt="Badge"></a>
</p>
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
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("Docs[1]"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[IMAGE] Badge[2]"), "stdout:\n{}", stdout);
    assert!(
        stdout.contains("[1] https://example.com/docs"),
        "stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("[2] https://example.com/badge"),
        "stdout:\n{}",
        stdout
    );
}

#[test]
fn test_render_html_inline_table_references_for_split_inline_html_link() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"Before <a href="https://example.com/docs">Docs</a> after."#,
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
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("Before Docs[1] after."),
        "stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("[1] https://example.com/docs"),
        "stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Before [1]Docs after."),
        "stdout:\n{}",
        stdout
    );
}
