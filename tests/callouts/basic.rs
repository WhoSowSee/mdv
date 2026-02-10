use super::*;

#[test]
fn test_callout_renders_label_and_body() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Info]\n┃ \n┃ Example text\n"),
        "expected callout header and body, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("[!info]"),
        "expected callout marker to be hidden, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_backslash_keeps_blockquote_context() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> Арбуз\\\n> Арбуз\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout backslash");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Important]"),
        "expected callout header, stdout:\n{}",
        stdout
    );

    let arbuz_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("Арбуз"))
        .collect();

    assert!(
        !arbuz_lines.is_empty(),
        "expected callout body lines, stdout:\n{}",
        stdout
    );
    assert!(
        arbuz_lines.iter().all(|line| line.starts_with("┃ ")),
        "expected backslash content to stay inside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("│ Арбуз"),
        "expected no plain blockquote after backslash, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_adds_blank_lines_around() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Alpha\n> [!info]\n> Example text\nOmega\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let alpha_idx = lines
        .iter()
        .position(|line| *line == "Alpha")
        .expect("alpha line present");
    let callout_idx = lines
        .iter()
        .position(|line| *line == "┃ [Info]")
        .expect("callout header present");
    let omega_idx = lines
        .iter()
        .position(|line| *line == "Omega")
        .expect("omega line present");

    let before_callout = &lines[alpha_idx + 1..callout_idx];
    let after_callout = &lines[callout_idx + 3..omega_idx];

    assert_eq!(
        before_callout
            .iter()
            .filter(|line| line.trim().is_empty())
            .count(),
        1,
        "expected one blank line before callout, stdout:\n{}",
        stdout
    );
    assert_eq!(
        after_callout
            .iter()
            .filter(|line| line.trim().is_empty())
            .count(),
        1,
        "expected one blank line after callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_alias_uses_label() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tldr]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout alias");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Tldr]\n┃ \n┃ Example text\n"),
        "expected alias label to render, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_admonition_syntaxes_render() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        ":::note\nAlpha\n:::\n\n:::{note} Title\nBeta\n:::\n\n!!! note Арбуз\nГамма\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for admonition callout syntaxes");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Note]"),
        "expected note callout header, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("┃ [Title]"),
        "expected custom callout label to render, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("┃ [Арбуз]"),
        "expected custom callout label in bang syntax, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Alpha") && stdout.contains("Beta") && stdout.contains("Гамма"),
        "expected callout bodies to render, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains(":::note") && !stdout.contains("!!! note"),
        "expected raw admonition markers to be hidden, stdout:\n{}",
        stdout
    );
}
