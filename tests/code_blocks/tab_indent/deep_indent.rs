use super::*;

#[test]
fn fully_double_tab_indented_fence_dedents_code_content() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "\t\t```\n\t\tprint(\"x\")\n\t\t```\n").expect("write markdown");

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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
        .arg("--no-colors")
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
