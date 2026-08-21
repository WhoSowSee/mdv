use super::*;

#[test]
fn test_callout_renders_label_and_body() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
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
fn test_callout_simple_icons_render_portable_alert_markers() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note]\n> Note body\n\n> [!tip]\n> Tip body\n\n> [!important]\n> Important body\n\n> [!warning]\n> Warning body\n\n> [!caution]\n> Caution body\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-simple-icons")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with portable callout icons");

    assert!(
        output.status.success(),
        "expected show-simple-icons to be accepted, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for expected in [
        "┃ [i] Note",
        "┃ [*] Tip",
        "┃ [!] Important",
        "┃ [!] Warning",
        "┃ [x] Caution",
    ] {
        assert!(
            stdout.contains(expected),
            "expected portable callout header {expected:?}, stdout:\n{stdout}"
        );
    }
}

#[test]
fn test_callout_simple_icons_cover_all_builtin_categories() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!abstract]\n> Body\n\n> [!info]\n> Body\n\n> [!todo]\n> Body\n\n> [!success]\n> Body\n\n> [!question]\n> Body\n\n> [!failure]\n> Body\n\n> [!danger]\n> Body\n\n> [!bug]\n> Body\n\n> [!example]\n> Body\n\n> [!quote]\n> Body\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-simple-icons")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with portable callout icons");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for expected in [
        "┃ [i] Abstract",
        "┃ [i] Info",
        "┃ [ ] Todo",
        "┃ [+] Success",
        "┃ [?] Question",
        "┃ [x] Failure",
        "┃ [!] Danger",
        "┃ [x] Bug",
        "┃ [*] Example",
        "┃ [>] Quote",
    ] {
        assert!(
            stdout.contains(expected),
            "expected portable callout header {expected:?}, stdout:\n{stdout}"
        );
    }
}

#[test]
fn test_callout_backslash_keeps_blockquote_context() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> Арбуз\\\n> Арбуз\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
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
        .arg("--no-colors")
        .arg("-w")
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
