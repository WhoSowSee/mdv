use assert_cmd::Command;
use mdv::utils::{display_width, strip_ansi};
use std::fs;
use tempfile::NamedTempFile;

fn mdv_cmd() -> Command {
    let mut command = crate::support::mdv_cmd();
    command.arg("--no-config");
    command
}

fn render(markdown: &str, args: &[&str]) -> String {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, markdown).unwrap();
    let output = mdv_cmd()
        .args(["--color", "never"])
        .args(args)
        .arg(file.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    strip_ansi(&String::from_utf8(output.stdout).unwrap())
}

fn assert_framed_fraction(output: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let expected = [
        "╭───────────────────╮",
        "│  abcdefghijklmnop │",
        "│  ──────────────── │",
        "│     qrstuvwxyz    │",
        "╰───────────────────╯",
    ];
    let start = lines
        .iter()
        .position(|line| line.contains(expected[0]))
        .unwrap_or_else(|| panic!("missing math frame:\n{output}"));
    let frame = &lines[start..start + expected.len()];
    let width = display_width(frame[0]);
    for (line, expected) in frame.iter().zip(expected) {
        assert!(line.contains(expected), "missing {expected:?}:\n{output}");
        assert!(line.starts_with('│') && line.ends_with('│'), "{output}");
        assert_eq!(display_width(line), width, "{output}");
    }
    assert!(!output.contains('\u{2062}'), "{output}");
}

#[test]
fn test_inline_math_renders_unicode() {
    let output = render(
        "Inline math: $E = mc^2$, $H_2O$, and $\\alpha + \\beta$.",
        &[],
    );
    assert!(output.contains("E = mc²"));
    assert!(output.contains("H₂O"));
    assert!(output.contains("α + β"));
    assert!(!output.contains("$E = mc^2$"));
}

#[test]
fn test_display_math_renders_block() {
    let output = render("$$\\frac{1}{2} + \\sqrt{3}$$", &[]);
    assert!(output.contains("  1\n  ─ + √3\n  2"), "{output}");
}

#[test]
fn math_containers_preserve_spacing_and_ignore_code_options() {
    for (style, expected) in [
        ("simple", "A\n\n│ x\n\nB\n"),
        ("pretty", "A\n\n╭───╮\n│ x │\n╰───╯\n\nB\n"),
    ] {
        let output = render(
            "A\n\n$$x$$\n\nB\n",
            &[
                "--math-block-style",
                style,
                "--code-block-style",
                "pretty:show-name;show-icon",
                "--code-line-numbers",
            ],
        );
        assert_eq!(output, expected, "{style}");
    }
}

#[test]
fn test_fenced_math_block_renders() {
    let output = render(
        "\x60\x60\x60math\n\\int_0^1 x^2 dx\n\x60\x60\x60",
        &[
            "--code-block-style",
            "simple:show-name",
            "--math-block-style",
            "simple",
        ],
    );
    assert!(output.contains("│  1\n│ ∫  x² dx\n│  0"), "{output}");
    assert!(!output.contains("Math"));
    assert!(!output.contains("\x60\x60\x60"));
    assert!(!output.contains("\\int_0^1 x^2 dx"));
}

#[test]
fn extended_terminal_math_renders_structured_layout() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/math-layout.md");
    let output = mdv_cmd()
        .args([
            "--color",
            "never",
            "--math-block-style",
            "simple",
            "--line-numbers",
            "source",
        ])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    for expected in [
        "‖",
        "x‖₂",
        "a+b",
        "───",
        "c+d",
        "⎡",
        "⎣",
        "⎧",
        "⎩",
        "def",
        "└",
        "times",
        "α+β",
        "ℙ(A∣ B)",
        "x+$y$",
        "|x|",
    ] {
        assert!(output.contains(expected), "missing {expected:?}:\n{output}");
    }
    for source in ["\\dfrac", "\\begin", "\\overset", "\\underbrace"] {
        assert!(!output.contains(source), "unrendered {source:?}:\n{output}");
    }
    assert!(
        output.lines().any(|line| line.starts_with(" 5 │")),
        "{output}"
    );
}

#[test]
fn html_keeps_backslash_math_delimiters_outside_math_events() {
    let output = render("Inline \\(x_1\\).\n\n\\[x^2\\]\n", &["--html"]);
    assert!(!output.contains("class=\"math"), "{output}");
    assert!(output.contains("x_1") && output.contains("x^2"), "{output}");
}

#[test]
fn pretty_callout_does_not_split_a_wide_math_layout() {
    let output = render(
        "> [!NOTE]\n>\n> \\[\n> \\frac{abcdefghijklmnop}{qrstuvwxyz}\n> \\]\n",
        &["--cols", "16", "--math-block-style", "pretty"],
    );
    assert_framed_fraction(&output);
}

#[test]
fn inline_math_stays_inside_link_text() {
    let source = "| Link |\n| --- |\n| [value \\(x_1\\)](https://example.com) |\n";
    for style in ["inline", "clickable", "inlinetable"] {
        let output = render(source, &["--link-style", style]);
        assert!(output.contains("value x₁"), "{style}:\n{output}");
    }
}

#[test]
fn dollar_math_pipes_preserve_table_headers_and_display_cells() {
    let output = render(
        "| $|h|$ | Value |\n| --- | --- |\n| $|b|$ | 1 |\n| $\\displaystyle \\sum_{n=0}^{N-1}x_n$ | 2 |\n",
        &[],
    );
    assert!(output.contains("|h|") && output.contains("|b|"), "{output}");
    assert!(output.contains("Value") && output.contains('┼'), "{output}");
    assert!(output.contains("N-1") && output.contains("n=0"), "{output}");
    assert!(!output.contains("_(") && !output.contains("^("), "{output}");
}

#[path = "math/contexts.rs"]
mod contexts;
