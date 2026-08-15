use super::*;

#[test]
fn test_inline_table_references_stay_inside_blockquote_table() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> Quote intro before links table.\n>\n> | Provider | Console |\n> | --- | --- |\n> | AWS | [console](https://console.aws.amazon.com/) |\n> | Azure | [portal](https://portal.azure.com/) |\n>\n> Quote outro after links table.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--link-style")
        .arg("inlinetable")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for blockquote table inline references");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_reference = lines
        .iter()
        .find(|line| line.contains("[1] https://console.aws.amazon.com/"))
        .expect("first reference line present");
    let first_reference_idx = lines
        .iter()
        .position(|line| line.contains("[1] https://console.aws.amazon.com/"))
        .expect("first reference line index present");
    let second_reference = lines
        .iter()
        .find(|line| line.contains("[2] https://portal.azure.com/"))
        .expect("second reference line present");
    let second_reference_idx = lines
        .iter()
        .position(|line| line.contains("[2] https://portal.azure.com/"))
        .expect("second reference line index present");
    let outro_line = lines
        .iter()
        .find(|line| line.contains("Quote outro after links table."))
        .expect("blockquote outro line present");
    let outro_line_idx = lines
        .iter()
        .position(|line| line.contains("Quote outro after links table."))
        .expect("blockquote outro line index present");

    assert!(
        first_reference
            .trim_start()
            .starts_with("│ [1] https://console.aws.amazon.com/"),
        "expected first reference to keep blockquote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        second_reference
            .trim_start()
            .starts_with("│ [2] https://portal.azure.com/"),
        "expected second reference to keep blockquote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        outro_line
            .trim_start()
            .starts_with("│ Quote outro after links table."),
        "expected quote outro to remain inside blockquote, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line
            .trim_start()
            .starts_with("[1] https://console.aws.amazon.com/")),
        "reference must not break out of blockquote, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line
            .trim_start()
            .starts_with("[2] https://portal.azure.com/")),
        "reference must not break out of blockquote, stdout:\n{}",
        stdout
    );
    assert!(
        first_reference_idx < second_reference_idx
            && second_reference_idx + 1 < outro_line_idx
            && lines[second_reference_idx + 1].trim() == "│",
        "expected an empty blockquote line between references and quote outro, stdout:\n{}",
        stdout
    );
}
