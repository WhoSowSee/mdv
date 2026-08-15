use super::*;

#[test]
fn heading_before_tab_indented_code_does_not_insert_empty_first_line() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(
        &temp_file,
        "# Test\n```\n\tstring.format(\"scale=-1:'min(%d,ih)':flags=fast_bilinear\", rt.preview.max_height / 2)\n```\n",
    )
    .expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty")
        .arg("--wrap")
        .arg("word")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    let top_idx = lines
        .iter()
        .position(|line| line.contains('╭'))
        .expect("expected pretty code block top border");
    let first_content = *lines
        .get(top_idx + 1)
        .expect("expected first content line after top border");

    let without_frame = first_content
        .trim_start_matches(' ')
        .trim_start_matches('│')
        .trim_start_matches(' ');
    assert!(
        !without_frame.trim_end_matches('│').trim().is_empty(),
        "unexpected empty first content line in pretty code block, stdout:\n{}",
        normalized
    );
}

#[test]
fn top_level_tab_indented_text_renders_as_paragraph() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\tTest text\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");
    let visible: Vec<&str> = normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    assert_eq!(
        visible,
        vec!["Test text"],
        "expected paragraph output, stdout:\n{}",
        normalized
    );
}

#[test]
fn top_level_space_indented_text_renders_as_paragraph() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "    Test text\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");
    let visible: Vec<&str> = normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    assert_eq!(
        visible,
        vec!["Test text"],
        "expected paragraph output, stdout:\n{}",
        normalized
    );
}
