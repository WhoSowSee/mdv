use super::*;

#[test]
fn attached_footnotes_follow_paragraphs() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "First paragraph with note[^first].\n\nSecond paragraph with note[^second].\n\n[^first]: One\n[^second]: Two\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--footnote-style")
        .arg("attached")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with attached footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_count = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .count();
    assert_eq!(
        separator_count, 4,
        "each paragraph footnote block should have opening and closing separators: {}",
        stdout
    );
    let first_pos = stdout
        .find("First paragraph with note[^first].")
        .expect("first paragraph present");
    let separator_after_first = stdout[first_pos..]
        .find('◇')
        .map(|offset| offset + first_pos)
        .expect("separator after first paragraph");
    let second_pos = stdout
        .find("Second paragraph with note[^second].")
        .expect("second paragraph present");
    assert!(
        separator_after_first < second_pos,
        "first separator should appear before second paragraph: {}",
        stdout
    );
    assert!(
        !stdout.contains("[^first]: One"),
        "raw definitions should be stripped"
    );
}

#[test]
fn footnotes_have_single_blank_line_after_block_elements() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Intro\n\n```\ncode\n```\n\ntext with note[^a]\n\n[^a]: One\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with block before footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let lines: Vec<&str> = stdout.lines().collect();
    let separator_idx = lines
        .iter()
        .position(|line| line.trim_start().starts_with('◇'))
        .expect("separator line present");
    let prev_content_idx = (0..separator_idx)
        .rfind(|idx| !lines[*idx].trim().is_empty())
        .expect("content before separator");
    let blank_lines = separator_idx.saturating_sub(prev_content_idx + 1);
    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before footnotes, got {}: {}",
        blank_lines, stdout
    );
}

#[test]
fn attached_footnotes_render_inside_list_items() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "- item with note[^a]\n\n[^a]: one\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg("--footnote-style")
        .arg("attached")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with attached footnotes inside list");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .collect();
    assert_eq!(
        separator_lines.len(),
        2,
        "expected opening and closing separators: {}",
        stdout
    );
    assert!(
        stdout.contains("[^a] one"),
        "footnote content should render inline: {}",
        stdout
    );
}

#[test]
fn attached_footnotes_render_after_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| A | B |\n| - | - |\n| foo[^a] | bar |\n\n[^a]: alpha\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("40")
        .arg("--footnote-style")
        .arg("attached")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with table footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .collect();
    assert!(
        !separator_lines.is_empty(),
        "attached table footnotes should render with separators: {}",
        stdout
    );
    assert!(
        stdout.contains("[^a] alpha"),
        "footnote body should render after table: {}",
        stdout
    );
}

#[test]
fn footnotes_leave_single_blank_line_after_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Line with note[^a]\n\n[^a]: A\n\nNext block\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("40")
        .arg("--footnote-style")
        .arg("attached")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with spacing check");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let lines: Vec<&str> = stdout.lines().collect();
    let separator_idx = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with('◇'))
        .expect("separator present");
    let next_content_idx = lines
        .iter()
        .skip(separator_idx + 1)
        .position(|line| !line.trim().is_empty())
        .map(|offset| separator_idx + 1 + offset)
        .expect("next content present");

    let blank_lines = next_content_idx.saturating_sub(separator_idx + 1);
    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line after footnotes: {}",
        stdout
    );
}
