use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
use mdv::utils::display_width;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_code_language_pretty_style_named_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hello\")\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("pretty")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(
        output.status.success(),
        "mdv exited with status {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(
        stdout.contains("│ print(\"hello\") │"),
        "expected symmetric padding, got:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\n\n│ print(\"hello\")"),
        "should not duplicate blank lines:\n{}",
        stdout
    );

    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        lines.len() >= 3,
        "expected pretty block lines, got {:?}",
        lines
    );

    let top = lines[0];
    let middle = lines[1];
    let bottom = lines[2];

    assert!(top.starts_with("╭─ Python"), "top line: {}", top);
    assert!(top.ends_with('╮'), "top line should end with ╮: {}", top);
    assert!(middle.starts_with("│ "), "middle line: {}", middle);
    assert!(
        middle.ends_with(" │"),
        "middle line should end with space+│: {}",
        middle
    );
    assert!(bottom.starts_with('╰'), "bottom line: {}", bottom);
    assert!(
        bottom.ends_with('╯'),
        "bottom line should end with ╯: {}",
        bottom
    );

    let top_width = display_width(top);
    let middle_width = display_width(middle);
    let bottom_width = display_width(bottom);

    assert_eq!(
        top_width, middle_width,
        "top and middle widths should match ({} vs {}); stdout:\n{}",
        top_width, middle_width, stdout
    );
    assert_eq!(
        top_width, bottom_width,
        "top and bottom widths should match ({} vs {}); stdout:\n{}",
        top_width, bottom_width, stdout
    );
}

#[test]
fn test_default_code_block_style_is_pretty() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn demo() {}\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-A").arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("╭─ Rust"))
        .stdout(predicate::str::contains("╰"));
}

#[test]
fn test_pretty_style_empty_code_block_has_right_padding() {
    let temp_file = NamedTempFile::new().unwrap();
    // Empty fenced block; language shown to match the reported case
    fs::write(&temp_file, "# T\n\n```\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("pretty")
        .arg("--show-empty-elements")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    // Find the first pretty frame after the header
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
    // Skip the header line
    let _ = lines.next();
    // Expect pretty top border line with a dash after label
    let top = lines.next().expect("top border");
    assert!(top.contains("╭─ Text ─╮"), "top border: {}", top);
    // Middle line must end with space + right border
    let middle = lines.next().expect("middle line");
    assert!(middle.ends_with(" │"), "middle: {}", middle);
}

#[test]
fn test_pretty_style_empty_block_falls_back_when_too_narrow() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```text\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("pretty")
        .arg("--show-empty-elements")
        .arg("--wrap")
        .arg("word")
        .arg("-c")
        .arg("9")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(stdout.contains("│ Text"), "stdout: {}", stdout);
    assert!(!stdout.contains("╭─ Text"), "stdout: {}", stdout);
}

#[test]
fn test_simple_language_label_wraps_under_char_width() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```elixir\nIO.puts(\"Hello\")\n```\n").unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--code-block-style")
        .arg("pretty")
        .arg("--wrap")
        .arg("char")
        .arg("-c")
        .arg("6")
        .arg("-A")
        .arg(temp_file.path());

    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    for line in stdout.lines() {
        if line.trim_start().starts_with('│') {
            let width = display_width(line);
            assert!(
                width <= 6,
                "code-block line exceeds width: {} ({} cols)",
                line,
                width
            );
        }
    }
}
