use super::*;

#[test]
fn test_callout_setext_heading_simple_keeps_single_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> ---\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout setext heading prefix");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines() {
        if line.starts_with('│') {
            panic!(
                "unexpected non-callout prefix in simple setext heading, stdout:\n{}",
                stdout
            );
        }
    }
}

#[test]
fn test_callout_setext_h1_simple_keeps_single_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> =\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout setext h1 prefix");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines() {
        if line.starts_with('│') {
            panic!(
                "unexpected non-callout prefix in simple setext h1, stdout:\n{}",
                stdout
            );
        }
    }
}

#[test]
fn test_callout_setext_heading_pretty_has_no_inner_pipe() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> ---\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty setext heading pipe check");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines().filter(|line| line.starts_with('│')) {
        assert!(
            !line.contains('┃'),
            "expected no callout pipe inside pretty frame, stdout:\n{}",
            stdout
        );
    }

    let heading_line = stdout
        .lines()
        .find(|line| line.contains("dadas"))
        .expect("heading line present");
    let indent = spaces_after_prefix(heading_line, '│');
    assert_eq!(
        indent, 1,
        "expected single left padding for setext heading, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_setext_h1_pretty_has_no_inner_pipe() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> dadas\n> =\n> Как сделать так\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty setext h1 pipe check");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines().filter(|line| line.starts_with('│')) {
        assert!(
            !line.contains('┃'),
            "expected no callout pipe inside pretty frame, stdout:\n{}",
            stdout
        );
    }

    let heading_line = stdout
        .lines()
        .find(|line| line.contains("dadas"))
        .expect("heading line present");
    let indent = spaces_after_prefix(heading_line, '│');
    assert_eq!(
        indent, 1,
        "expected single left padding for setext h1, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_headings_do_not_affect_global_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> # Inside\n>\n## Outside\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--heading-layout")
        .arg("level")
        .arg("--smart-indent")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout smart indent isolation");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    let outside_line = stdout
        .lines()
        .find(|line| line.contains("Outside"))
        .expect("outside heading line present");
    assert!(
        !outside_line.starts_with(' '),
        "expected outside heading to be flush-left under smart-indent, stdout:\n{}",
        stdout
    );
}
