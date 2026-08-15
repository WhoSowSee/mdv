use super::*;

#[test]
fn tab_indented_fence_after_heading_renders_as_fence() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "# Test\n\n\t```\n\tprint(\"x\")\n\t```\n").expect("write markdown");

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
        .arg("simple:show-name")
        .arg("--no-colors")
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
        .arg("simple:show-name")
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
