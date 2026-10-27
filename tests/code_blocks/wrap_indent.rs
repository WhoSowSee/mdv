use crate::support::mdv_cmd;
use mdv::utils::{display_width_ansi, strip_ansi};

use std::fs;
use tempfile::NamedTempFile;

fn capture_indent_spaces(mode: Option<&str>) -> (usize, usize) {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(
        &temp_file,
        "```lua\n  key = \"value\" -- comment that wraps because indentation behaviour should be clear in this example\n```",
    )
    .expect("write markdown");

    let mut cmd = mdv_cmd();
    cmd.args(["--color", "never"])
        .arg("--code-block-style")
        .arg("simple")
        .arg("--wrap")
        .arg("word")
        .arg("--cols")
        .arg("60");

    if let Some(mode) = mode {
        cmd.arg("--code-wrap-indent").arg(mode);
    }

    cmd.arg(temp_file.path());

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("valid UTF-8 stdout");

    let mut lines = stdout.lines();
    let mut found = None;
    while let Some(line) = lines.next() {
        if line.contains("key = \"value\"") {
            let continuation = lines
                .next()
                .expect("wrapped continuation line in code block");
            found = Some((line, continuation));
            break;
        }
    }

    let (first_line, continuation_line) = found.expect("code block output present");

    let base_after_pipe = first_line.strip_prefix('│').unwrap_or(first_line);
    let base_spaces = base_after_pipe.chars().take_while(|c| *c == ' ').count();

    let continuation_after_pipe = continuation_line
        .strip_prefix('│')
        .unwrap_or(continuation_line);
    let continuation_spaces = continuation_after_pipe
        .chars()
        .take_while(|c| *c == ' ')
        .count();

    (base_spaces, continuation_spaces)
}

#[test]
fn code_wrap_indent_defaults_to_double() {
    let (base_spaces, continuation_spaces) = capture_indent_spaces(None);
    let base_indent = base_spaces.saturating_sub(1);
    assert_eq!(
        continuation_spaces,
        1 + base_indent + 2,
        "expected double hanging indent (base + 2 spaces)"
    );
}

#[test]
fn code_wrap_indent_base_matches_original_indentation() {
    let (base_spaces, continuation_spaces) = capture_indent_spaces(Some("base"));
    let base_indent = base_spaces.saturating_sub(1);
    assert_eq!(
        continuation_spaces,
        1 + base_indent,
        "expected continuation to align with original indentation"
    );
}

#[test]
fn code_wrap_indent_none_preserves_legacy_alignment() {
    let (_, continuation_spaces) = capture_indent_spaces(Some("none"));
    assert_eq!(
        continuation_spaces, 1,
        "expected continuation to start immediately after border"
    );
}

#[test]
fn code_wrap_indent_reserves_width_on_every_continuation() {
    for body in [
        "word ".repeat(24),
        "слово ".repeat(24),
        "界界 ".repeat(24),
        "longtoken".repeat(16),
    ] {
        let markdown = format!("```lua\n  -- {}\n```\n", body.trim_end());
        for style in ["basic", "simple", "pretty"] {
            for wrap in ["char", "word"] {
                for (indent, expected_spaces) in [("none", 0), ("base", 2), ("double", 4)] {
                    for color in ["never", "always"] {
                        let output = mdv_cmd()
                            .args([
                                "--no-config",
                                "--cols=40",
                                "--theme=catppuccin",
                                "--code-block-style",
                                style,
                                "--code-wrap-indent",
                                indent,
                                "--wrap",
                                wrap,
                                "--color",
                                color,
                            ])
                            .write_stdin(markdown.as_str())
                            .assert()
                            .success();
                        let output = String::from_utf8(output.get_output().stdout.clone())
                            .expect("valid UTF-8 stdout");
                        let plain = strip_ansi(&output);
                        let context = format!("{style}/{wrap}/{indent}/{color}: {plain}");
                        assert!(
                            output.lines().all(|line| display_width_ansi(line) <= 40),
                            "row exceeds terminal width: {context}"
                        );

                        let content = plain
                            .lines()
                            .filter_map(|line| match style {
                                "basic" => line.strip_prefix("  "),
                                "simple" => line.strip_prefix("│ "),
                                "pretty" => line
                                    .strip_prefix("│ ")
                                    .and_then(|line| line.strip_suffix(" │")),
                                _ => unreachable!(),
                            })
                            .collect::<Vec<_>>();
                        assert!(
                            content.len() > 2,
                            "expected several wrapped rows: {context}"
                        );
                        for line in content.iter().skip(1) {
                            assert_eq!(
                                line.chars().take_while(|ch| *ch == ' ').count(),
                                expected_spaces,
                                "incorrect continuation indent: {context}"
                            );
                        }
                        let rendered = content.concat();
                        let visible_content = |text: &str| {
                            text.chars()
                                .filter(|ch| !ch.is_whitespace())
                                .collect::<String>()
                        };
                        assert_eq!(
                            visible_content(&rendered),
                            visible_content(&format!("-- {body}")),
                            "wrapped content changed: {context}"
                        );
                    }
                }
            }
        }
    }
}
