use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

fn capture_indent_spaces(mode: Option<&str>) -> (usize, usize) {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(
        &temp_file,
        "```lua\n  key = \"value\" -- comment that wraps because indentation behaviour should be clear in this example\n```",
    )
    .expect("write markdown");

    let mut cmd = Command::cargo_bin("mdv").expect("mdv binary");
    cmd.arg("-A")
        .arg("--style-code-block")
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
