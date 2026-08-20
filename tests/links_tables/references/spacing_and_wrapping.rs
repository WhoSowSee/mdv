use super::*;

const INLINE_REFERENCE_WRAPPING_TABLE: &str = concat!(
    "| Platform | Protocol | Support |\n",
    "| --- | --- | --- |\n",
    "| [kitty](https://example.com/kitty) (>= 0.28.0) | [Kitty unicode placeholders](https://example.com/kgp) | Built-in |\n",
    "| [Windows Terminal](https://example.com/wt) (>= v1.22.10352.0) | [Sixel graphics format](https://example.com/sixel) | Built-in |\n",
    "| Fallback | [ASCII art (Unicode block)](https://example.com/ascii) | [Chafa](https://example.com/chafa) required (>= 1.16.0) |\n",
    "| [alpha](https://example.com/alpha) | [beta](https://example.com/beta) | [gamma](https://example.com/gamma) |\n",
    "| [delta](https://example.com/delta) | [epsilon](https://example.com/epsilon) | [zeta](https://example.com/zeta) |\n",
);

#[test]
fn test_inline_table_reference_blocks_leave_one_blank_line_before_following_content() {
    let cases = [
        (
            "markdown table",
            "| Link |\n| --- |\n| [table](https://example.com/table) |\n\nafter markdown table\n",
            "https://example.com/table",
            "after markdown table",
            false,
            &[] as &[&str],
        ),
        (
            "html table",
            "<table><tr><th>Link</th></tr><tr><td><a href=\"https://example.com/html\">table</a></td></tr></table>\n\nafter html table\n",
            "https://example.com/html",
            "after html table",
            true,
            &[],
        ),
        (
            "top-level list",
            "- [list](https://example.com/list)\n\nafter list\n",
            "https://example.com/list",
            "after list",
            false,
            &[],
        ),
        (
            "callout",
            "> [!note] Callout\n> [link](https://example.com/callout)\n\nafter callout\n",
            "https://example.com/callout",
            "after callout",
            false,
            &["--callout-style", "pretty"],
        ),
        (
            "plaintext code block",
            "```text\n[link](https://example.com/plaintext)\n```\n\nafter plaintext code block\n",
            "https://example.com/plaintext",
            "after plaintext code block",
            false,
            &["--code-block-style", "pretty"],
        ),
        (
            "mixed markdown code block",
            "```markdown\n[block](https://example.com/block)\n\n| Link |\n| --- |\n| [nested](https://example.com/nested) |\n```\n\nafter mixed markdown code block\n",
            "https://example.com/block",
            "after mixed markdown code block",
            false,
            &["--code-block-style", "pretty"],
        ),
    ];

    for (name, markdown, url, following_text, render_html, extra_args) in cases {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, markdown).unwrap();

        let mut cmd = mdv_cmd();
        cmd.arg("--no-config")
            .arg("--no-colors")
            .arg("--link-style")
            .arg("inlinetable");
        if render_html {
            cmd.arg("--render-html");
        }
        cmd.args(extra_args);
        cmd.arg(temp_file.path());

        let output = cmd.output().expect("run mdv with inline table links");
        assert!(output.status.success(), "{name}: mdv execution failed");

        let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
        let lines: Vec<&str> = stdout.lines().collect();
        let expected_reference = format!("[1] {url}");
        let reference_idx = lines
            .iter()
            .position(|line| line.trim() == expected_reference)
            .unwrap_or_else(|| panic!("{name}: reference line missing, stdout:\n{stdout}"));
        let following_idx = lines
            .iter()
            .position(|line| line.contains(following_text))
            .unwrap_or_else(|| panic!("{name}: following content missing, stdout:\n{stdout}"));

        assert_eq!(
            following_idx,
            reference_idx + 2,
            "{name}: expected one blank line after references, stdout:\n{stdout}"
        );
        assert!(
            lines[reference_idx + 1].trim().is_empty(),
            "{name}: separator line should be empty, stdout:\n{stdout}"
        );
    }
}

#[test]
fn test_inline_table_reference_marker_keeps_brackets_together_when_wrapped() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| ID | Name | Link | Command | Status |\n|----|------|------|---------|--------|\n| 101 | ingest | [spec](https://example.com/1) | `cargo test -p ingest` | active |\n| 102 | transform | [runbook](https://example.com/2) | `cargo test -p transform` | active |\n| 103 | export | [dashboard](https://example.com/3) | `cargo test -p export` | paused |\n| 104 | notify | [alerts](https://example.com/4) | `cargo test -p notify` | active |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("52")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--table-smart-indent")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for wrapped inline-table references");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let dangling_bracket_line = lines
        .iter()
        .any(|line| line.contains("┆ ]") || line.contains("│ ]"));
    assert!(
        !dangling_bracket_line,
        "reference marker must not be split into a dangling `]`, stdout:\n{}",
        stdout
    );

    assert!(
        stdout.contains("┆ [2]") || stdout.contains("runbook[2]"),
        "expected wrapped or inline marker for runbook link, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("┆ [3]") || stdout.contains("│ [3]") || stdout.contains("dashboard[3]"),
        "expected wrapped or inline marker for dashboard link, stdout:\n{}",
        stdout
    );

    let table_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| line.contains('│') || line.contains('┼'))
        .collect();

    assert!(!table_lines.is_empty(), "expected rendered table lines");

    let expected_width = table_lines[0].chars().count();
    for line in table_lines {
        assert_eq!(
            line.chars().count(),
            expected_width,
            "table line width changed, layout is broken:\n{}",
            stdout
        );
    }
}

#[test]
fn test_inline_table_reference_marker_stays_with_link_when_trailing_text_wraps() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, INLINE_REFERENCE_WRAPPING_TABLE).unwrap();

    let output = mdv_cmd()
        .args([
            "--no-config",
            "--cols",
            "64",
            "--link-style",
            "inlinetable",
            "--pretty-table",
        ])
        .arg(temp_file.path())
        .output()
        .expect("run mdv for inline table link with trailing text");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let clean = mdv::utils::strip_ansi(&stdout);
    assert!(
        clean.contains("│ kitty[1]"),
        "reference marker should stay with link text when trailing cell text wraps, stdout:\n{stdout}"
    );
    assert!(
        clean
            .lines()
            .filter(|line| matches!(line.chars().next(), Some('╭' | '│' | '╞' | '├' | '╰')))
            .all(|line| mdv::utils::display_width(line) == 64),
        "table width changed after reference marker cleanup, stdout:\n{stdout}"
    );
}

#[test]
fn test_inline_table_reference_constraints_do_not_overflow_narrow_table() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, INLINE_REFERENCE_WRAPPING_TABLE).unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .arg("--cols")
        .arg("30")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--pretty-table")
        .arg(temp_file.path())
        .output()
        .expect("run mdv for narrow inline table references");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let table_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| matches!(line.chars().next(), Some('╭' | '│' | '╞' | '├' | '╰')))
        .collect();
    assert!(
        table_lines
            .iter()
            .all(|line| mdv::utils::display_width(line) <= 30),
        "narrow table must not exceed configured terminal width, stdout:\n{stdout}"
    );
    assert!(
        table_lines.iter().any(|line| line.contains("[10]")),
        "multi-digit reference marker must stay intact in a narrow table, stdout:\n{stdout}"
    );
}
