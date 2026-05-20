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
