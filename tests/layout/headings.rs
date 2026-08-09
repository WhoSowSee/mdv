use super::*;
use tempfile::tempdir;

#[test]
fn test_smart_indent_promotes_first_heading() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "## Heading Two\n\nContent\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Heading Two\n"))
        .stdout(predicate::str::contains("\n Content\n"));
}

#[test]
fn test_smart_indent_limits_growth_per_step() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# H1\n\n## H2\n\n###### H6\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n H2\n"))
        .stdout(predicate::str::contains("\n  H6\n"));
}

#[test]
fn test_smart_indent_handles_mixed_levels() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Root\n\n## Level 2\n\n###### Level 6\n\n#### Level 4\n\n## Level 2 second\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--smart-indent")
        .arg("--heading-layout")
        .arg("level")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n Level 2\n"))
        .stdout(predicate::str::contains("\n   Level 6\n"))
        .stdout(predicate::str::contains("\n  Level 4\n"))
        .stdout(predicate::str::contains("\n Level 2 second\n"));
}

#[test]
fn test_center_heading_layout_adds_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "# Centered\n## Another\n\n---\n\nParagraph body\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--heading-layout")
        .arg("center")
        .arg("-A")
        .arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n\n◈"))
        .stdout(predicate::str::contains("\nParagraph body"))
        .stdout(predicate::str::contains("\n\n\n").not());
}

#[test]
fn test_single_blank_line_before_heading_after_empty_pretty_code_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```\n```\n\n##\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--code-block-style")
        .arg("pretty")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for empty code block");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "##")
        .expect("heading present");

    let mut blank_lines = 0usize;
    let mut idx = heading_idx;
    while idx > 0 {
        idx -= 1;
        if lines[idx].trim().is_empty() {
            blank_lines += 1;
        } else {
            break;
        }
    }

    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before heading, stdout: {}",
        stdout
    );
}

#[test]
fn test_single_blank_line_before_heading_with_surrounding_elements() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "- Item\n-\n\n```\n```\n>\n>\n\n##\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--code-block-style")
        .arg("pretty")
        .arg("--wrap")
        .arg("char")
        .arg("-c")
        .arg("74")
        .arg("--show-empty-elements")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with surrounding elements");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "##")
        .expect("heading present");

    let mut blank_lines = 0usize;
    let mut idx = heading_idx;
    while idx > 0 {
        idx -= 1;
        if lines[idx].trim().is_empty() {
            blank_lines += 1;
        } else {
            break;
        }
    }

    assert_eq!(
        blank_lines, 1,
        "expected exactly one blank line before heading, stdout: {}",
        stdout
    );
}

#[test]
fn test_heading_markers_support_levels_short_alias_and_center_layout() {
    let markdown = "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n";
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--heading-layout")
        .arg("none")
        .arg("--show-heading-markers")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with heading markers");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(stdout, markdown);

    let alias_file = NamedTempFile::new().unwrap();
    fs::write(&alias_file, "## Alias\n").unwrap();
    let alias_output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("40")
        .arg("--heading-layout")
        .arg("center")
        .arg("-k")
        .arg(alias_file.path())
        .output()
        .expect("mdv runs with the heading marker alias");
    assert!(alias_output.status.success());
    let alias_stdout = String::from_utf8(alias_output.stdout).expect("stdout utf8");
    assert_eq!(alias_stdout.trim(), "## Alias");
}

#[test]
fn test_heading_markers_load_from_config() {
    let config_dir = tempdir().unwrap();
    fs::write(
        config_dir.path().join("config.yaml"),
        "no_colors: true\nheading_layout: none\nshow_heading_markers: true\n",
    )
    .unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "### Configured\n").unwrap();

    let output = mdv_cmd()
        .arg("--config-file")
        .arg(config_dir.path())
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with configured heading markers");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(stdout, "### Configured\n");
}

#[test]
fn test_heading_markers_do_not_duplicate_empty_placeholders() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "#\n\n##\n").unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--heading-layout")
        .arg("none")
        .arg("--show-empty-elements")
        .arg("--show-heading-markers")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with empty heading markers");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let visible_lines: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(visible_lines, ["#", "##"]);
}
