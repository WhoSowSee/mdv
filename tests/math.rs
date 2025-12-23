use assert_cmd::Command;
use mdv::utils::strip_ansi;
use std::fs;
use tempfile::NamedTempFile;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

#[test]
fn test_inline_math_renders_unicode() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Inline math: $E = mc^2$ and $\\alpha + \\beta$.",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);

    assert!(clean.contains("E = mc²"));
    assert!(clean.contains("α + β"));
    assert!(!clean.contains("$E = mc^2$"));
}

#[test]
fn test_display_math_renders_block() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "$$\\frac{1}{2} + \\sqrt{3}$$").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("--style-code-block")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);

    assert!(clean.contains("1⁄2 + √3"));
}

#[test]
fn test_fenced_math_block_renders() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "```math\n\\int_0^1 x^2 dx\n```",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("--style-code-block")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);

    assert!(clean.contains("∫₀¹ x² dx"));
    assert!(clean.contains("Math"));
    assert!(clean.contains("│"));
    assert!(!clean.contains("```"));
    assert!(!clean.contains("\\int_0^1 x^2 dx"));
}
