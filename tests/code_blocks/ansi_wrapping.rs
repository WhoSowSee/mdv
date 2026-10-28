use crate::support::mdv_cmd;
use mdv::utils::{display_width_ansi, strip_ansi};

fn render(markdown: &str, args: &[&str]) -> String {
    let output = mdv_cmd()
        .args([
            "--no-config",
            "--theme=catppuccin",
            "--color=always",
            "--color-depth=truecolor",
        ])
        .args(args)
        .write_stdin(markdown)
        .assert()
        .success();
    String::from_utf8(output.get_output().stdout.clone()).expect("UTF-8 output")
}

#[test]
fn ansi_wrapping_output_grows_linearly() {
    for wrap in ["char", "word"] {
        let mut sizes = Vec::new();
        for repetitions in [200, 400] {
            let source = "local x = \"value\"; ".repeat(repetitions);
            let markdown = format!("```lua\n{source}\n```\n");
            let output = render(
                &markdown,
                &[
                    "--cols=80",
                    "--wrap",
                    wrap,
                    "--code-block-style=basic",
                    "--code-wrap-indent=none",
                ],
            );
            assert!(output.lines().all(|line| display_width_ansi(line) <= 80));
            let visible = strip_ansi(&output);
            let non_whitespace = |text: &str| {
                text.chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
            };
            assert_eq!(non_whitespace(&visible), non_whitespace(&source));
            sizes.push(output.len());
        }
        assert!(
            sizes[1] <= sizes[0] * 5 / 2,
            "doubling source produced excessive ANSI growth in {wrap}: {sizes:?}"
        );
    }
}

#[test]
fn ansi_wrapping_preserves_comment_style() {
    let tokens = regex::Regex::new(r"\x1b\[[0-9;]*m|[^\x1b]+").unwrap();
    let markdown = format!("```lua\n-- {}\n```\n", "alpha beta ".repeat(24));
    for style in ["basic", "simple", "pretty"] {
        for wrap in ["char", "word"] {
            let output = render(
                &markdown,
                &[
                    "--cols=40",
                    "--code-block-style",
                    style,
                    "--wrap",
                    wrap,
                    "--custom-code-theme=comment=#00ff00",
                ],
            );
            let mut green = false;
            let mut italic = false;
            let mut letters = 0;
            for token in tokens.find_iter(&output).map(|token| token.as_str()) {
                match token {
                    "\x1b[0m" => {
                        green = false;
                        italic = false;
                    }
                    "\x1b[38;2;0;255;0m" => green = true,
                    "\x1b[3m" => italic = true,
                    "\x1b[23m" => italic = false,
                    sgr if sgr.starts_with("\x1b[38;") || sgr == "\x1b[39m" => green = false,
                    text if !text.starts_with('\x1b') => {
                        for character in text.chars().filter(char::is_ascii_alphabetic) {
                            assert!(
                                green && italic,
                                "lost style at {character:?}: {style}/{wrap}: {output:?}"
                            );
                            letters += 1;
                        }
                    }
                    _ => {}
                }
            }
            assert_eq!(letters, 24 * 9, "{style}/{wrap}");
        }
    }
}
