use assert_cmd::Command;
use mdv::utils::strip_ansi;
use std::{fs, process::Output};
use tempfile::NamedTempFile;

fn render(markdown: &str, args: &[&str]) -> Output {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, markdown).unwrap();
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("mdv"));
    command.args(args).arg(file.path());
    command.output().unwrap()
}

fn stdout(markdown: &str, args: &[&str]) -> String {
    let output = render(markdown, args);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn unstyled_panel_chars(output: &str) -> String {
    let mut in_panel = false;
    let mut unstyled = String::new();
    for line in output.lines() {
        if line.contains("Properties") {
            in_panel = true;
        }
        if !in_panel {
            continue;
        }

        let mut styled = false;
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                let mut sequence = String::new();
                for next in chars.by_ref() {
                    if next == 'm' {
                        break;
                    }
                    sequence.push(next);
                }
                if sequence == "[0" {
                    styled = false;
                } else if sequence.starts_with("[38;") {
                    styled = true;
                }
                continue;
            }
            if !styled && !ch.is_whitespace() {
                unstyled.push(ch);
            }
        }

        if line.contains('╯') {
            break;
        }
    }
    unstyled
}

const DOCUMENT: &str = "---\ntitle: Lua API\ntags: [yazi, lua]\nlast_update:\n  author: Alice\nempty_string: ''\nempty_list: []\nempty_map: {}\n---\n# Body\n";

#[test]
fn front_matter_modes_render_expected_content() {
    let hidden = stdout(DOCUMENT, &["--no-colors"]);
    assert!(hidden.contains("Body"));
    assert!(!hidden.contains("Lua API"));

    let panel = stdout(DOCUMENT, &["--no-colors", "--front-matter", "panel"]);
    for expected in [
        "Properties",
        "title: Lua API",
        "tags: yazi · lua",
        "last_update: author: Alice",
        "\"\"",
        "[]",
        "{}",
        "Body",
    ] {
        assert!(panel.contains(expected), "missing {expected:?}:\n{panel}");
    }

    let source = stdout(DOCUMENT, &["--no-colors", "--front-matter", "source"]);
    assert!(source.contains("title: Lua API"));
    assert!(!source.contains("Properties"));

    let code = stdout(DOCUMENT, &["--no-colors", "--front-matter", "code"]);
    assert!(code.contains("title: Lua API"));
    assert!(code.contains("last_update:"));
    assert!(!code.contains("Properties"));
}

#[test]
fn table_mode_renders_properties_as_two_columns() {
    let terminal = stdout(
        DOCUMENT,
        &["--no-config", "--no-colors", "--front-matter", "table"],
    );
    for expected in ["Property", "Value", "title", "Lua API", "Body"] {
        assert!(
            terminal.contains(expected),
            "missing {expected:?}:\n{terminal}"
        );
    }
    assert!(!terminal.contains("Properties"));

    let html = stdout(
        DOCUMENT,
        &["--no-config", "--html", "--front-matter", "table"],
    );
    for expected in [
        "<table>",
        "<th>Property</th>",
        "<th>Value</th>",
        "<td>title</td>",
        "<td>Lua API</td>",
    ] {
        assert!(html.contains(expected), "missing {expected:?}:\n{html}");
    }
}

#[test]
fn plain_inline_and_blocks_modes_have_distinct_layouts() {
    let markdown = "---\ntitle: Lua API\ntags: [yazi, lua]\nauthor: Alice\n---\n# Body\n";

    let plain = stdout(
        markdown,
        &["--no-config", "--no-colors", "--front-matter", "plain"],
    );
    for expected in ["title: Lua API", "tags: yazi · lua", "author: Alice"] {
        assert!(
            plain.lines().any(|line| line.trim() == expected),
            "missing {expected:?}:\n{plain}"
        );
    }
    let plain_lines: Vec<_> = plain.lines().collect();
    let title = plain_lines
        .iter()
        .position(|line| line.trim() == "title: Lua API")
        .unwrap();
    assert_eq!(plain_lines[title + 1].trim(), "tags: yazi · lua");
    assert_eq!(plain_lines[title + 2].trim(), "author: Alice");

    let inline = stdout(
        markdown,
        &[
            "--no-config",
            "--no-colors",
            "--front-matter",
            "inline",
            "--cols",
            "120",
        ],
    );
    assert!(
        inline
            .lines()
            .any(|line| line.trim() == "title: Lua API • tags: yazi · lua • author: Alice"),
        "stdout:\n{inline}"
    );

    let blocks = stdout(
        markdown,
        &["--no-config", "--no-colors", "--front-matter", "blocks"],
    );
    let lines: Vec<_> = blocks.lines().collect();
    let title = lines
        .iter()
        .position(|line| line.trim() == "title")
        .unwrap();
    let key_indent = lines[title]
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .count();
    let value_indent = lines[title + 1]
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .count();
    assert_eq!(lines[title + 1].trim(), "Lua API", "stdout:\n{blocks}");
    assert_eq!(value_indent, key_indent + 2, "stdout:\n{blocks}");
}

#[test]
fn front_matter_requires_an_exact_first_line_mapping() {
    for markdown in [
        "\n---\nproperty: value\n---\n# Body\n",
        "--- \nproperty: value\n---\n# Body\n",
        "---\nproperty: value\n--- \n# Body\n",
        "---\nproperty: value\n# Body\n",
        "---\nOrdinary introduction.\n---\n# Body\n",
        "---\n- first item\n---\n# Body\n",
    ] {
        let output = stdout(markdown, &["--no-colors"]);
        assert!(
            output.contains("property: value")
                || output.contains("Ordinary introduction.")
                || output.contains("first item"),
            "stdout:\n{output}"
        );
    }
}

#[test]
fn invalid_yaml_reports_the_document_line() {
    let output = render("---\nfoo: :\n---\n# Body\n", &[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("document line 2, column 6"));
}

#[test]
fn front_matter_accepts_crlf_and_preserves_source_lines() {
    let output = stdout(
        "---\r\ntitle: Example\r\n---\r\n# Heading\r\n",
        &["--no-colors", "--line-numbers", "source"],
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("4 ") && line.contains("Heading")),
        "stdout:\n{output}"
    );
}

#[test]
fn reverse_places_metadata_after_the_body() {
    let markdown = "---\ntitle: Example\n---\n# First\n\n# Second\n";
    for (mode, marker) in [
        ("panel", "Properties"),
        ("table", "Property"),
        ("plain", "title: Example"),
        ("inline", "title: Example"),
        ("blocks", "title"),
        ("source", "title: Example"),
        ("code", "title: Example"),
    ] {
        let output = stdout(
            markdown,
            &["--no-colors", "--front-matter", mode, "--reverse"],
        );
        let second = output.find("Second").unwrap();
        let first = output.find("First").unwrap();
        let metadata = output.find(marker).unwrap();
        assert!(second < first && first < metadata, "mode={mode}\n{output}");
    }
}

#[test]
fn html_preserves_modes_escaping_and_reverse_order() {
    let markdown = "---\ntitle: 'A < B'\nowner: Alice\n---\n# First\n\n# Second\n";
    let hidden = stdout(markdown, &["--html"]);
    assert!(!hidden.contains("A &lt; B"));

    for (mode, marker) in [
        ("panel", "<section class=\"front-matter\">"),
        ("table", "<table>"),
        ("plain", "<strong>title:</strong>"),
        ("inline", "<strong>title:</strong>"),
        ("blocks", "<dl>"),
        ("code", "<pre><code class=\"language-yaml\">"),
    ] {
        let output = stdout(markdown, &["--html", "--front-matter", mode, "--reverse"]);
        let second = output.find("<h1>Second</h1>").unwrap();
        let first = output.find("<h1>First</h1>").unwrap();
        let metadata = output.find(marker).unwrap();
        assert!(second < first && first < metadata, "mode={mode}\n{output}");
        assert!(output.contains("A &lt; B"), "mode={mode}\n{output}");
        if mode == "panel" {
            assert!(output.contains("<dd>A &lt; B</dd>"));
        } else if mode == "plain" {
            assert!(output.contains("<br />"), "stdout:\n{output}");
        } else if mode == "inline" {
            assert!(
                output.contains("• <strong>owner:</strong> Alice"),
                "stdout:\n{output}"
            );
        }
    }
    let source = stdout(
        "---\ntitle: Example\n\n***\n---\n# Body\n",
        &["--html", "--front-matter", "source"],
    );
    assert!(source.contains("<p>title: Example</p>"));
    assert!(source.matches("<hr").count() >= 2, "stdout:\n{source}");
    assert!(!source.contains("language-yaml"));
}

#[test]
fn from_filter_keeps_panel_properties() {
    let markdown = "---\ntitle: Example\n---\n# Start\n\n## Target\n\nSelected body.\n";
    let output = stdout(
        markdown,
        &["--no-colors", "--front-matter", "panel", "--from", "Target"],
    );
    assert!(output.contains("Properties"));
    assert!(output.contains("Target"));
    assert!(!output.contains("Start"));
}

#[test]
fn panel_preserves_dedicated_colors_and_wraps_once() {
    let long_value = "d".repeat(114);
    let long_key = "в".repeat(128);
    let markdown = format!("---\nsdsd: dasdas{long_value}\ndsadas{long_key}: dasda\n---\n# Body\n");
    let output = render(
        &markdown,
        &[
            "--no-config",
            "--front-matter",
            "panel",
            "--reverse",
            "--cols",
            "120",
            "--custom-theme",
            "front_matter_title=#010203;front_matter_key=#040506;front_matter_value=#070809;front_matter_border=#0a0b0c",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for color in ["1;2;3", "4;5;6", "7;8;9", "10;11;12"] {
        assert!(
            stdout.contains(&format!("38;2;{color}m")),
            "stdout:\n{stdout}"
        );
    }
    assert_eq!(unstyled_panel_chars(&stdout), "", "stdout:\n{stdout}");

    let clean = strip_ansi(&stdout);
    let property_lines: Vec<_> = clean
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('│') && line.ends_with('│'))
        .map(|line| line.trim_matches('│').trim())
        .collect();

    assert!(
        property_lines
            .iter()
            .any(|line| line.starts_with("sdsd: dasdas")),
        "stdout:\n{stdout}"
    );
    assert!(
        property_lines.iter().all(|line| line.chars().count() > 1),
        "stdout:\n{stdout}"
    );
}
