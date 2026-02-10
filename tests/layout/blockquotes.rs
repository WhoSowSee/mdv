use super::*;

#[test]
fn test_blockquote_list_preserves_marker_prefix() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> - Ensure blockquoted list items maintain bullet prefixes even when the text spans multiple wrapped lines within the quote.\n> - Confirm subsequent bullet entries keep the same indentation so the rendered output never drops the quote marker.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-c")
        .arg("20")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed for blockquoted list");

    assert!(
        output.status.success(),
        "mdv exited with {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_line_ok = lines.iter().any(|line| line.starts_with("│ - Ensure"));
    let first_wrap_ok = lines
        .iter()
        .any(|line| line.starts_with("│   ") && line.contains("bullet"));
    let second_line_ok = lines.iter().any(|line| line.starts_with("│ - Confirm"));
    let second_wrap_ok = lines
        .iter()
        .any(|line| line.starts_with("│   ") && line.contains("quote"));

    assert!(
        first_line_ok && first_wrap_ok,
        "expected wrapped first bullet with quote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        second_line_ok && second_wrap_ok,
        "expected wrapped second bullet with quote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\n- Ensure"),
        "expected quote prefix to remain on first bullet, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\n- Confirm"),
        "expected quote prefix to remain on second bullet, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_blockquote_respects_heading_indent_and_single_space() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "## Heading\n\n> Quote text\n").unwrap();

    let output = mdv_cmd()
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for blockquote heading indent");

    assert!(
        output.status.success(),
        "mdv exited with {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let quote_line = lines
        .iter()
        .find(|line| line.contains("Quote text"))
        .expect("quote line present");

    assert!(
        quote_line.starts_with("  │ "),
        "expected heading indent before quote, stdout:\n{}",
        stdout
    );

    let after_prefix = &quote_line["  │ ".len()..];
    assert!(
        !after_prefix.starts_with(' '),
        "expected single space after quote marker, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_blockquote_backslash_keeps_prefix_on_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> First\\\n> Second\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for blockquote backslash");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| *line == "│ First")
        .expect("first quote line present");
    let second_idx = lines
        .iter()
        .position(|line| *line == "│ Second")
        .expect("second quote line present");

    let gap = &lines[first_idx + 1..second_idx];
    assert!(
        gap.iter().any(|line| line.trim() == "│"),
        "expected blank line to keep blockquote prefix, stdout:\n{}",
        stdout
    );
    assert!(
        gap.iter().all(|line| !line.trim().is_empty()),
        "expected no unprefixed blank lines inside blockquote, stdout:\n{}",
        stdout
    );
}
