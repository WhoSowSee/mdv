use super::*;
use mdv::utils::{display_width, strip_ansi};

fn render(input: &str, cols: usize, color: &str) -> String {
    let output = mdv_cmd()
        .args(["--no-config", "--render-html", "--color", color, "--cols"])
        .arg(cols.to_string())
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    strip_ansi(&String::from_utf8(output.stdout).unwrap())
}

#[test]
fn narrow_headings_wrap_once_with_consistent_indentation() {
    let text = "abcdefghijklmnopqrstuvwxyz".repeat(3);
    for (cols, color) in [(16, "never"), (30, "always")] {
        let output = render(&format!("### {text}\n"), cols, color);
        let lines: Vec<_> = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert_eq!(lines.len(), text.len().div_ceil(cols - 2), "{output}");
        for line in &lines[..lines.len() - 1] {
            assert_eq!(display_width(line), cols, "{output}");
            assert!(line.starts_with("  "), "{output}");
        }
        assert_eq!(
            lines.iter().map(|line| line.trim()).collect::<String>(),
            text
        );
    }
}

#[test]
fn narrow_callouts_reserve_outer_heading_indent_before_wrapping() {
    let cols = 30;
    let text = "abcdefghijklmnopqrstuvwxyz".repeat(3);
    for color in ["never", "always"] {
        let output = render(&format!("## Intro\n\n> [!NOTE]\n> {text}\n"), cols, color);
        let lines: Vec<_> = output
            .lines()
            .filter_map(|line| {
                line.strip_prefix("  │ ")
                    .and_then(|line| line.strip_suffix(" │"))
            })
            .collect();
        let inner_width = cols - 6;
        assert_eq!(lines.len(), text.len().div_ceil(inner_width), "{output}");
        for line in &lines[..lines.len() - 1] {
            assert_eq!(display_width(line.trim_end()), inner_width, "{output}");
        }
        assert_eq!(
            lines.iter().map(|line| line.trim()).collect::<String>(),
            text
        );
        assert!(
            output.lines().all(|line| display_width(line) <= cols),
            "{output}"
        );
    }
}

#[test]
fn narrow_nested_callouts_preserve_unicode_and_inline_code() {
    let text = "配置".repeat(20);
    for color in ["never", "always"] {
        let output = render(
            &format!("## Intro\n\n> [!NOTE]\n> > [!TIP]\n> > `{text}`\n"),
            30,
            color,
        );
        let lines: Vec<_> = output
            .lines()
            .filter_map(|line| {
                line.strip_prefix("  │ │ ")
                    .and_then(|line| line.strip_suffix(" │ │"))
            })
            .collect();
        assert_eq!(lines.len(), 5, "{output}");
        assert_eq!(
            lines.iter().map(|line| line.trim()).collect::<String>(),
            format!("`{text}`")
        );
        assert!(
            output.lines().all(|line| display_width(line) <= 30),
            "{output}"
        );
    }
}

#[test]
fn narrow_formatted_headings_match_plain_heading_layout() {
    let expected = render("### Run locally without installing\n", 30, "always");
    for input in [
        "### **Run locally** without installing\n",
        "### [Run locally without installing](https://example.com)\n",
        "<h3>Run locally without installing</h3>\n",
    ] {
        assert_eq!(render(input, 30, "always"), expected, "{input}");
    }
}
