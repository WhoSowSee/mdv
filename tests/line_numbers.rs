use assert_cmd::Command;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

fn render(markdown: &str, args: &[&str]) -> String {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, markdown).unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("-A")
        .arg(file.path())
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn assert_line_prefixes(output: &str, expected: &[&str]) {
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), expected.len(), "stdout:\n{output}");
    for (line, prefix) in lines.iter().zip(expected) {
        assert!(line.starts_with(prefix), "stdout:\n{output}");
    }
}

#[test]
fn rendered_line_number_formats() {
    let plain = render("# Heading\nParagraph.\n", &["-j"]);
    assert_line_prefixes(&plain, &["1 ", "2 ", "3 "]);
    assert!(plain.contains("Heading") && plain.contains("Paragraph."));

    let separated = render("# Heading\nParagraph.\n", &["--line-numbers", "separator"]);
    assert_line_prefixes(&separated, &["1 │ ", "2 │ ", "3 │ "]);
    assert!(separated.contains("Heading") && separated.contains("Paragraph."));
}

#[test]
fn help_lists_line_number_values_and_examples() {
    let output = mdv_cmd().arg("--help").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let normalized_help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    let possible_values_start = help.find("Possible values:").unwrap();
    let examples_start =
        possible_values_start + help[possible_values_start..].find("Examples:").unwrap();
    let possible_values = &help[possible_values_start..examples_start];

    for expected in [
        "-j, --line-numbers [<MODE>]",
        "Possible values:",
        "source: Number physical Markdown source lines instead of rendered rows",
        "separator: Display a separator after each rendered row number",
        "Examples:",
        "--line-numbers separator",
        "--line-numbers source",
        "--line-numbers \"source;separator\"",
    ] {
        assert!(
            normalized_help.contains(expected),
            "missing {expected:?} in help:\n{help}"
        );
    }
    assert!(possible_values_start < examples_start, "help:\n{help}");
    assert!(
        !possible_values.contains("source;separator"),
        "help:\n{help}"
    );
    assert!(!possible_values.contains('│'), "help:\n{help}");
}

#[test]
fn line_number_options_reserve_exact_gutter_width() {
    let markdown = format!("{}abcdefghijklmnopq\n", "\n".repeat(10));

    assert_eq!(
        render(&markdown, &["-j", "-c", "20"]),
        "1 abcdefghijklmnopq\n"
    );
    assert_eq!(
        render(
            &markdown,
            &["--line-numbers", "source;separator", "-c", "20"],
        ),
        "11 │ abcdefghijklmno\n   │ pq\n"
    );
}

#[test]
fn source_lines_leave_wrapped_rows_unnumbered() {
    let output = render(
        "First source line.\nSecond source line in the same paragraph.\n\nThis source line contains enough words to wrap across terminal rows.\n\nFinal source line.\n",
        &["-c", "32", "--line-numbers", "source"],
    );
    let lines = output.lines().collect::<Vec<_>>();

    for (number, text) in [
        ("1 ", "First source line."),
        ("2 ", "Second source line"),
        ("4 ", "This source line"),
        ("6 ", "Final source line."),
    ] {
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with(number) && line.contains(text)),
            "stdout:\n{output}"
        );
    }
    assert!(
        lines.iter().any(|line| line
            .strip_prefix("  ")
            .is_some_and(|continuation| !continuation.trim().is_empty())),
        "stdout:\n{output}"
    );
}

#[test]
fn source_blank_lines_and_renderer_spacing_are_distinct() {
    let output = render(
        "# Heading\nParagraph.\n\n\nSecond paragraph.\n",
        &["--line-numbers", "source"],
    );
    let lines = output.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), 5, "stdout:\n{output}");
    assert!(lines[0].starts_with("1 ") && lines[0].contains("Heading"));
    assert!(lines[1].starts_with("  ") && lines[1][2..].trim().is_empty());
    assert!(lines[2].starts_with("2 ") && lines[2].contains("Paragraph."));
    assert!(lines[3].starts_with("3 ") && lines[3][2..].trim().is_empty());
    assert!(lines[4].starts_with("5 ") && lines[4].contains("Second paragraph."));
    assert!(!lines.iter().any(|line| line.starts_with("4 ")));
}

#[test]
fn source_blank_line_before_heading_reuses_existing_row() {
    let output = render("Paragraph.\n\n## Heading\n", &["--line-numbers", "source"]);
    let lines = output.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), 3, "stdout:\n{output}");
    assert!(lines[0].starts_with("1 ") && lines[0].contains("Paragraph."));
    assert!(lines[1].starts_with("2 ") && lines[1][2..].trim().is_empty());
    assert!(lines[2].starts_with("3 ") && lines[2].contains("Heading"));
}

#[test]
fn source_numbers_share_width_with_heading_indentation() {
    let output = render(
        "### H3\n\nabcdefghijklmnopqrstuvwxyz123456\n",
        &["--line-numbers", "source", "-c", "40"],
    );
    assert!(output.contains("3    abcdefghijklmnopqrstuvwxyz123456\n"));
}

#[test]
fn source_numbers_survive_preprocessor_insertions() {
    let output = render(
        "- [ ] Task\nFollowing paragraph.\n",
        &["--line-numbers", "source"],
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("2 ") && line.contains("Following paragraph.")),
        "stdout:\n{output}"
    );
}

#[test]
fn config_accepts_boolean_and_option_string() {
    let config_dir = TempDir::new().unwrap();
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, "# Heading\nParagraph.\n").unwrap();

    for (setting, expected) in [
        ("true", ["1 ", "2 ", "3 "]),
        ("\"source;separator\"", ["1 │ ", "  │ ", "2 │ "]),
    ] {
        fs::write(
            config_dir.path().join("config.yaml"),
            format!("line_numbers: {setting}\nno_colors: true\ncols: 40\n"),
        )
        .unwrap();

        let output = mdv_cmd()
            .arg("--config-file")
            .arg(config_dir.path())
            .arg(file.path())
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = String::from_utf8(output.stdout).unwrap();
        assert_line_prefixes(&output, &expected);
    }
}

#[test]
fn line_numbers_do_not_change_html_export() {
    let output = render("# Heading\n\nParagraph.\n", &["-j", "--html"]);
    assert!(output.contains("<h1>Heading</h1>"));
    assert!(!output.contains('│'));
}

#[test]
fn line_numbers_do_not_reveal_hidden_comments() {
    assert_eq!(render("<!-- hidden -->\n", &["-j", "--hide-comments"]), "");
}
