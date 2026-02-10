use super::*;

#[test]
fn test_link_truncation_tablecut_applies_to_inline_links_inside_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| ID | Link |\n| --- | --- |\n| 1 | [spec](https://example.com/service/alpha/beta/gamma/delta/epsilon/zzfinalzz) |\n",
    )
    .unwrap();

    let cut_output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("46")
        .arg("--link-style")
        .arg("inline")
        .arg("--link-truncation")
        .arg("cut")
        .arg(temp_file.path())
        .output()
        .expect("run mdv with cut truncation in table");
    assert!(cut_output.status.success());
    let cut_stdout = String::from_utf8(cut_output.stdout).expect("cut stdout utf8");

    let tablecut_output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("46")
        .arg("--link-style")
        .arg("inline")
        .arg("--link-truncation")
        .arg("tablecut")
        .arg(temp_file.path())
        .output()
        .expect("run mdv with tablecut truncation in table");
    assert!(tablecut_output.status.success());
    let tablecut_stdout = String::from_utf8(tablecut_output.stdout).expect("tablecut stdout utf8");

    assert!(
        cut_stdout.contains("zzfinalzz"),
        "cut mode should keep full URL in table cells, stdout:\n{}",
        cut_stdout
    );
    assert!(
        !tablecut_stdout.contains("zzfinalzz"),
        "tablecut should truncate URL tail inside table cells, stdout:\n{}",
        tablecut_stdout
    );
    assert!(
        tablecut_stdout.contains("..."),
        "tablecut should produce ellipsis for table inline URLs, stdout:\n{}",
        tablecut_stdout
    );
}
