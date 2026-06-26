use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_pretty_code_block_with_icon() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hello\")\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty:show-icons")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("Python"),
        "expected Python label, got:\n{}",
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
    assert!(
        top.starts_with("╭─  Python"),
        "expected icon before label on top line, got: {}",
        top
    );
    assert!(top.ends_with('╮'), "top line should end with ╮: {}", top);
}

#[test]
fn test_simple_code_block_with_icon() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn main() {}\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:show-icons")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│  Rust\n│ \n│ fn main()"),
        "expected icon+label line before code, got:\n{}",
        stdout
    );
}

#[test]
fn test_code_block_icons_hidden_when_language_label_hidden() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hello\")\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty:show-icons")
        .arg("--no-code-language")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        !stdout.contains("Python"),
        "language label should be hidden, got:\n{}",
        stdout
    );
    assert!(
        !stdout.contains(""),
        "icon should be hidden with label, got:\n{}",
        stdout
    );
}

#[test]
fn test_unknown_language_uses_default_icon() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```unknownlang\nx\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:show-icons")
        .arg("--no-code-guessing")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│   Unknownlang"),
        "expected default icon before unknown language label, got:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("│ "),
        "unknown language should not get a Python icon, got:\n{}",
        stdout
    );
}

#[test]
fn test_custom_default_icon_overrides_builtin_default() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```unknownlang\nx\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:show-icons")
        .arg("--custom-code-block")
        .arg("default:icon=🚀")
        .arg("--no-code-guessing")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│ 🚀 Unknownlang"),
        "expected custom default icon before unknown language label, got:\n{}",
        stdout
    );
}

#[test]
fn test_pretty_code_block_icon_only() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```python\nprint(\"hello\")\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("pretty:icon-only")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let top = lines[0];

    assert!(
        !stdout.contains("Python"),
        "language label should be hidden in icon-only mode, got:\n{}",
        stdout
    );
    assert!(
        top.starts_with("╭─  "),
        "expected only icon on top line, got: {}",
        top
    );
    assert!(top.ends_with('╮'), "top line should end with ╮: {}", top);
}

#[test]
fn test_simple_code_block_icon_only() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn main() {}\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:icon-only")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│ \n│ \n│ fn main()"),
        "expected only icon before code, got:\n{}",
        stdout
    );
}

#[test]
fn test_custom_code_block_icon_overrides_default() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn main() {}\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:show-icons")
        .arg("--custom-code-block")
        .arg("rust:icon=🦀")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│ 🦀 Rust\n│ \n│ fn main()"),
        "expected custom icon before label, got:\n{}",
        stdout
    );
}

#[test]
fn test_custom_code_block_label_override() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```rust\nfn main() {}\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:show-icons")
        .arg("--custom-code-block")
        .arg("rust:icon=🦀,label=russst")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│ 🦀 russst\n│ \n│ fn main()"),
        "expected custom icon and label, got:\n{}",
        stdout
    );
}

#[test]
fn test_custom_code_block_aliases_match_alternative_hints() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "```py\nprint(1)\n```\n").unwrap();

    let output = mdv_cmd()
        .arg("--code-block-style")
        .arg("simple:show-icons")
        .arg("--custom-code-block")
        .arg("python:icon=*,label=kd,aliases=py|py3")
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .expect("mdv executed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(
        stdout.contains("│ * kd\n│ \n│ print(1)"),
        "expected alias-matched custom icon and label, got:\n{}",
        stdout
    );
}
