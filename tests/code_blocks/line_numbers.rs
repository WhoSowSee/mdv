use assert_cmd::Command;
use mdv::utils::display_width;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

fn render(markdown: &str, args: &[&str]) -> String {
    let file = NamedTempFile::new().expect("create Markdown file");
    fs::write(&file, markdown).expect("write Markdown file");

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .args(args)
        .arg(file.path())
        .output()
        .expect("mdv executed");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout is UTF-8")
}

#[test]
fn code_line_numbers_are_limited_to_code_block_rows() {
    let output = render(
        "Before.\n\n```rust\none();\ntwo();\n```\n\nAfter.\n",
        &["-K"],
    );
    let lines = output.lines().collect::<Vec<_>>();

    assert!(lines.contains(&"  1 one();"), "{output}");
    assert!(lines.contains(&"  2 two();"), "{output}");
    assert!(lines.contains(&"Before."), "{output}");
    assert!(lines.contains(&"After."), "{output}");
}

#[test]
fn separate_code_blocks_share_the_same_gutter_width() {
    let long_block = (1..=12)
        .map(|line| format!("long_{line:02}();"))
        .collect::<Vec<_>>()
        .join("\n");
    let markdown = format!("```rust\nshort();\n```\n\n```rust\n{long_block}\n```\n");

    for args in [
        &["--code-line-numbers"][..],
        &["--code-line-numbers", "source;separator"][..],
    ] {
        let output = render(&markdown, args);
        let short_line = output
            .lines()
            .find(|line| line.contains("short();"))
            .expect("short block line");
        let long_line = output
            .lines()
            .find(|line| line.contains("long_01();"))
            .expect("long block line");
        let short_column = display_width(short_line.split_once("short();").unwrap().0);
        let long_column = display_width(long_line.split_once("long_01();").unwrap().0);

        assert_eq!(
            short_column,
            long_column,
            "arguments: {}\n{output}",
            args.join(" ")
        );
    }
}

#[test]
fn separator_mode_stays_inside_every_code_block_style() {
    for (style, expected) in [
        ("basic", "  1 │ one();"),
        ("simple", "│ 1 │ one();"),
        ("pretty", "│ 1 │ one();"),
    ] {
        let output = render(
            "```rust\none();\n```\n",
            &[
                "--code-block-style",
                style,
                "--code-line-numbers",
                "separator",
            ],
        );

        assert!(
            output.lines().any(|line| line.contains(expected)),
            "{output}"
        );

        if style == "pretty" {
            let frame = output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect::<Vec<_>>();
            assert_eq!(display_width(frame[0]), display_width(frame[1]), "{output}");
            assert_eq!(display_width(frame[0]), display_width(frame[2]), "{output}");
        }
    }
}

#[test]
fn source_mode_leaves_wrapped_continuations_unnumbered() {
    let markdown = "```rust\nabcdefghijklmnop\n```\n";
    let source = render(markdown, &["--cols", "12", "--code-line-numbers", "source"]);
    let source_rows = source.lines().collect::<Vec<_>>();
    assert!(source_rows.len() > 1, "{source}");
    assert!(source_rows[0].starts_with("  1 "), "{source}");
    assert!(source_rows[1].starts_with("    "), "{source}");

    let rendered = render(markdown, &["--cols", "12", "--code-line-numbers"]);
    let rendered_rows = rendered.lines().collect::<Vec<_>>();
    assert!(rendered_rows.len() > 1, "{rendered}");
    assert!(rendered_rows[0].starts_with("  1 "), "{rendered}");
    assert!(rendered_rows[1].starts_with("  2 "), "{rendered}");
}

#[test]
fn config_accepts_boolean_and_option_string() {
    let config_dir = TempDir::new().expect("create config directory");
    let file = NamedTempFile::new().expect("create Markdown file");
    fs::write(&file, "```rust\none();\n```\n").expect("write Markdown file");

    for (setting, expected) in [
        ("true", "  1 one();"),
        ("\"source;separator\"", "  1 │ one();"),
    ] {
        fs::write(
            config_dir.path().join("config.yaml"),
            format!("code_line_numbers: {setting}\nno_colors: true\n"),
        )
        .expect("write config file");

        let output = mdv_cmd()
            .arg("--config-file")
            .arg(config_dir.path())
            .arg(file.path())
            .output()
            .expect("mdv executed");
        assert!(output.status.success());
        let output = String::from_utf8(output.stdout).expect("stdout is UTF-8");
        assert!(output.contains(expected), "{output}");
    }
}

#[test]
fn help_lists_code_line_number_values_and_examples() {
    let output = mdv_cmd().arg("--help").output().expect("mdv executed");
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let normalized = help.split_whitespace().collect::<Vec<_>>().join(" ");

    for expected in [
        "-K, --code-line-numbers [<MODE>]",
        "Number rows inside code blocks",
        "source: Number physical code lines instead of wrapped terminal rows",
        "separator: Display a separator after each code line number",
        "--code-line-numbers separator",
        "--code-line-numbers source",
        "--code-line-numbers \"source;separator\"",
    ] {
        assert!(
            normalized.contains(expected),
            "missing {expected:?}:\n{help}"
        );
    }
}
