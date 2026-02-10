use super::*;

#[test]
fn test_table_rendering() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Table Test\n\n| Col1 | Col2 |\n|------|------|\n| A    | B    |\n| C    | D    |",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
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
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("it").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test document-level table style
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("et").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test inline style
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("i").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));

    // Test hide style
    let mut cmd = mdv_cmd();
    cmd.arg("-u").arg("h").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Link Test"));
}

#[test]
fn test_table_link_underlines_only_link_text_fragment() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| Col1 | Col2 |\n|------|------|\n| Before [link](https://example.com) after | Plain cell |\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-config")
        .arg("--cols")
        .arg("80")
        .arg("--link-style")
        .arg("clickable")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv for table link underline fragment");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid utf-8");
    let data_line = stdout
        .lines()
        .find(|line| line.contains("Before") && line.contains("Plain cell"))
        .expect("table data line with mixed link text present");

    assert!(
        data_line.contains("\u{1b}[4mlink\u{1b}[24m"),
        "link fragment should remain underlined, got: {}",
        data_line
    );
    assert!(
        !data_line.contains("\u{1b}[4m Before")
            && !data_line.contains("\u{1b}[4mBefore")
            && !data_line.contains("\u{1b}[4m Plain")
            && !data_line.contains("\u{1b}[4mPlain")
            && !data_line.contains("after\u{1b}[24m"),
        "underline should not leak to non-link text, got: {}",
        data_line
    );
    assert!(
        data_line.contains("\u{1b}]8;;https://example.com\u{1b}\\"),
        "clickable table link should include OSC8 hyperlink start, got: {}",
        data_line
    );
    assert!(
        data_line.contains("\u{1b}]8;;\u{1b}\\"),
        "clickable table link should include OSC8 hyperlink end, got: {}",
        data_line
    );
}

#[test]
fn test_table_fclickable_links_are_clickable() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| Col1 | Col2 |\n|------|------|\n| Before [link](https://example.com/forced) after | Plain cell |\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.env("MDV_NO_COLOR", "false")
        .arg("--no-config")
        .arg("--cols")
        .arg("80")
        .arg("--link-style")
        .arg("fclickable")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv for fclickable table hyperlink");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid utf-8");
    let data_line = stdout
        .lines()
        .find(|line| line.contains("Before") && line.contains("Plain cell"))
        .expect("table data line with mixed link text present");

    assert!(
        data_line.contains("\u{1b}]8;;https://example.com/forced\u{1b}\\"),
        "fclickable table link should include OSC8 hyperlink start, got: {}",
        data_line
    );
    assert!(
        data_line.contains("\u{1b}]8;;\u{1b}\\"),
        "fclickable table link should include OSC8 hyperlink end, got: {}",
        data_line
    );
}
