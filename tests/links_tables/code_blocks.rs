use super::*;

#[test]
fn test_inline_table_link_style_inside_text_code_block_pretty() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```text\nThis is a [link](https://example.com/example-path)\n```\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("pretty")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭─ Text"))
        .stdout(predicate::str::contains("│ This is a link[1]"))
        .stdout(predicate::str::contains(
            "[1] https://example.com/example-path",
        ))
        .stdout(predicate::str::contains("│ [1] https://example.com/example-path").not())
        .stdout(predicate::str::contains("[link](").not());
}

#[test]
fn test_inline_table_link_style_inside_text_code_block_simple() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```text\nThis is a [link](https://example.com/example-path)\n```\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("simple")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("80")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("│ Text"))
        .stdout(predicate::str::contains("│ This is a link[1]"))
        .stdout(predicate::str::contains(
            "[1] https://example.com/example-path",
        ))
        .stdout(predicate::str::contains("│ [1] https://example.com/example-path").not())
        .stdout(predicate::str::contains("[link](").not());
}

#[test]
fn test_inline_table_table_reference_stays_inside_markdown_code_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```markdown\n| Field | Value |\n| --- | --- |\n| docs | [guide](https://example.com/guide) |\n```\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("pretty")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("90")
        .arg("--no-colors")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("guide[1]"))
        .stdout(predicate::str::contains("│ [1] https://example.com/guide"))
        .stdout(predicate::str::contains("\n [1] https://example.com/guide").not());
}

#[test]
fn test_inline_table_mixed_references_split_between_code_body_and_nested_table() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Demo\n\n```markdown\n[text](https://example.com/block)\n| Field | Value |\n| --- | --- |\n| docs | [guide](https://example.com/table) |\n```\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty")
        .arg("-u")
        .arg("it")
        .arg("--link-truncation")
        .arg("none")
        .arg("--cols")
        .arg("90")
        .arg("--no-colors")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for mixed code block references");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        stdout.contains("text[1]"),
        "expected block link marker, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("guide[1]"),
        "expected table link marker with table-local numbering, stdout:\n{}",
        stdout
    );

    let table_reference_idx = lines
        .iter()
        .position(|line| line.contains("│ [1] https://example.com/table"))
        .expect("table reference line inside code block");
    let block_reference_idx = lines
        .iter()
        .position(|line| {
            line.trim_start()
                .starts_with("[1] https://example.com/block")
        })
        .expect("block-level reference line outside code block");
    let code_block_bottom_idx = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with('╰'))
        .expect("code block bottom border present");

    assert!(
        !lines
            .iter()
            .any(|line| line.contains("│ [1] https://example.com/block")),
        "block-level references must not stay inside code block, stdout:\n{}",
        stdout
    );
    assert!(
        table_reference_idx < code_block_bottom_idx,
        "table reference should stay inside code block, stdout:\n{}",
        stdout
    );
    assert!(
        block_reference_idx > code_block_bottom_idx,
        "block-level reference should render after code block, stdout:\n{}",
        stdout
    );
}
