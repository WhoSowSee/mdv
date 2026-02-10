use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn tab_indented_fence_after_heading_renders_as_fence() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "# Test\n\n\t```\n\tprint(\"x\")\n\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│ print(\"x\")"),
        "expected code content line, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│ ```"),
        "tab-indented fence should not render literal backticks, stdout:\n{}",
        normalized
    );
}

#[test]
fn tab_indented_fence_after_list_is_not_nested_by_indent() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "- item\n\n\t```\n\tprint(\"x\")\n\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n│ Text\n"),
        "expected top-level code block label line, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("\n  │ Text\n"),
        "code block should not keep synthetic list indentation, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│ ```"),
        "tab-indented fence should not render literal backticks, stdout:\n{}",
        normalized
    );
}

#[test]
fn space_indented_fence_inside_list_renders_without_list_offset() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "- item\n  ```\n  print(\"x\")\n  ```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n│ Text\n"),
        "expected top-level code block label line, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("\n  │ Text\n"),
        "code block should not keep list indentation offset, stdout:\n{}",
        normalized
    );
}

#[test]
fn tab_inside_regular_fence_stays_as_code_indentation() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "```\n\tprint(\"x\")\n```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│     print(\"x\")"),
        "expected preserved code indentation from tab, stdout:\n{}",
        normalized
    );
}

#[test]
fn tab_inside_regular_fence_after_heading_stays_as_code_indentation() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "# Test\n\n```\n\tprint(\"x\")\n```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│     print(\"x\")"),
        "expected preserved code indentation from tab after heading, stdout:\n{}",
        normalized
    );
}

#[test]
fn fully_tab_indented_fence_dedents_code_content() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t```\n\tprint(\"x\")\n\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│ print(\"x\")"),
        "expected dedented code content for fully tab-indented fence, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│     print(\"x\")"),
        "fully tab-indented fence must not keep extra inner indentation, stdout:\n{}",
        normalized
    );
}

#[test]
fn fully_double_tab_indented_fence_dedents_code_content() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t\t```\n\t\tprint(\"x\")\n\t\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│ print(\"x\")"),
        "expected dedented code content for fully double-tab-indented fence, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│ ```"),
        "double-tab-indented fence should not render literal backticks, stdout:\n{}",
        normalized
    );
}

#[test]
fn fully_double_tab_indented_fence_preserves_extra_inner_tab() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t\t```\n\t\t\tprint(\"x\")\n\t\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│     print(\"x\")"),
        "expected one preserved inner tab after removing shared double-tab indent, stdout:\n{}",
        normalized
    );
}

#[test]
fn fully_double_tab_indented_fence_with_less_indented_content_dedents_to_plain_content() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t\t```\n\tprint(\"x\")\n\t\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("│ print(\"x\")"),
        "expected plain dedented content when inner line has fewer tabs than fences, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│ ```"),
        "fence markers must not leak into rendered code body, stdout:\n{}",
        normalized
    );
}

#[test]
fn fully_double_tab_open_with_triple_tab_close_renders_clean_block() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t\t```\n\tprint(\"x\")\n\t\t\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("print(\"x\")"),
        "expected content inside code block, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│     ```"),
        "closing fence should not render as code text, stdout:\n{}",
        normalized
    );
}

#[test]
fn fully_five_tab_open_with_four_tab_close_renders_clean_block() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t\t\t\t\t```\n\tprint(\"x\")\n\t\t\t\t```\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("print(\"x\")"),
        "expected content inside code block, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│                 ```"),
        "opening fence should not render as code text, stdout:\n{}",
        normalized
    );
    assert!(
        !normalized.contains("│             ```"),
        "closing fence should not render as code text, stdout:\n{}",
        normalized
    );
}

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
        .arg("-A")
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
        .arg("-A")
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
fn top_level_space_indented_text_stays_code_block() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "    Test text\n").expect("write markdown");

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("run mdv");

    assert!(output.status.success(), "mdv failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let normalized = stdout.replace("\r\n", "\n");

    assert!(
        normalized.contains("\n│ Text\n") || normalized.starts_with("│ Text\n"),
        "expected code block label for space-indented block, stdout:\n{}",
        normalized
    );
}
