use super::*;

#[test]
fn test_callout_pretty_style_respects_heading_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Heading\n\n> [!note]\n> Base informational callout.\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout heading indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines.contains(&"  │ Base informational callout. │"),
        "expected heading indent on callout frame, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_simple_heading_keeps_pipe_alignment() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> ### Heading\n> Body line\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout heading alignment");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let heading_line = lines
        .iter()
        .find(|line| line.contains("Heading"))
        .expect("heading line present");
    let body_line = lines
        .iter()
        .find(|line| line.contains("Body line"))
        .expect("body line present");

    assert!(
        heading_line.starts_with("┃ "),
        "expected callout pipe to be left-aligned for heading, stdout:\n{}",
        stdout
    );
    assert!(
        body_line.starts_with("┃ "),
        "expected callout pipe to be left-aligned for body, stdout:\n{}",
        stdout
    );

    let header_idx = lines
        .iter()
        .position(|line| *line == "┃ [Important]")
        .expect("callout header present");
    let spacer_line = lines
        .get(header_idx + 1)
        .expect("spacer line after callout header");
    let heading_after_spacer = lines
        .get(header_idx + 2)
        .expect("heading line after spacer");

    assert_eq!(
        *spacer_line, "┃ ",
        "expected single prefixed spacer line after header, stdout:\n{}",
        stdout
    );
    assert!(
        heading_after_spacer.contains("Heading"),
        "expected heading immediately after spacer, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_heading_has_no_blank_edges_in_pretty_style() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!important]\n> ### Heading\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout heading spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let top_idx = lines
        .iter()
        .position(|line| line.contains('╭'))
        .expect("top border present");
    let bottom_idx = lines
        .iter()
        .rposition(|line| line.contains('╰'))
        .expect("bottom border present");

    let first_content = lines
        .get(top_idx + 1)
        .expect("line after top border present");
    let last_content = lines
        .get(bottom_idx.saturating_sub(1))
        .expect("line before bottom border present");

    assert!(
        !is_empty_box_line(first_content),
        "expected no empty line after top border, stdout:\n{}",
        stdout
    );
    assert!(
        !is_empty_box_line(last_content),
        "expected no empty line before bottom border, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_simple_headings_respect_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note]\n> # H1\n> ### H3\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--heading-layout")
        .arg("level")
        .arg("--smart-indent")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout smart indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let h1_line = lines
        .iter()
        .find(|line| line.contains("H1"))
        .expect("H1 line present");
    let h3_line = lines
        .iter()
        .find(|line| line.contains("H3"))
        .expect("H3 line present");

    let h1_indent = spaces_after_prefix(h1_line, '┃');
    let h3_indent = spaces_after_prefix(h3_line, '┃');

    assert_eq!(
        h3_indent,
        h1_indent + 1,
        "expected smart-indent to compress H3 indent inside callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_headings_respect_smart_indent() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note]\n> # H1\n> ### H3\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--heading-layout")
        .arg("level")
        .arg("--smart-indent")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout pretty smart indent");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let h1_line = lines
        .iter()
        .find(|line| line.contains("H1"))
        .expect("H1 line present");
    let h3_line = lines
        .iter()
        .find(|line| line.contains("H3"))
        .expect("H3 line present");

    let h1_indent = spaces_after_prefix(h1_line, '│');
    let h3_indent = spaces_after_prefix(h3_line, '│');

    assert_eq!(
        h3_indent,
        h1_indent + 1,
        "expected smart-indent to compress H3 indent inside pretty callout, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_pretty_spacing_between_callouts_after_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "## Heading\n\n> [!info]\n> First\n\n> [!info]\n> Second\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let first_top = lines
        .iter()
        .position(|line| line.contains('╭'))
        .expect("first callout top border present");
    let first_bottom = lines
        .iter()
        .skip(first_top + 1)
        .position(|line| line.contains('╰'))
        .map(|idx| idx + first_top + 1)
        .expect("first callout bottom border present");
    let second_top = lines
        .iter()
        .skip(first_bottom + 1)
        .position(|line| line.contains('╭'))
        .map(|idx| idx + first_bottom + 1)
        .expect("second callout top border present");

    let between = &lines[first_bottom + 1..second_top];
    let blank_lines = between.iter().filter(|line| line.trim().is_empty()).count();
    let non_blank_lines = between
        .iter()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert_eq!(
        non_blank_lines, 0,
        "expected only blank lines between callouts, stdout:\n{}",
        stdout
    );
    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line between callouts, stdout:\n{}",
        stdout
    );
}
