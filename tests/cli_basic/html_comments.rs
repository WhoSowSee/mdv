use super::*;

fn render_comments(input: &str, hidden: bool, args: &[&str]) -> String {
    let mut command = mdv_cmd();
    command.args([
        "--render-html",
        "--color",
        "never",
        "--heading-layout",
        "none",
    ]);
    if hidden {
        command.arg("--hide-comments");
    }
    let output = command
        .args(args)
        .arg("-")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn render_html_comments_respect_visibility() {
    for (input, comment) in [
        ("<!-- comment -->\n\nVisible\n", "<!-- comment -->"),
        ("Visible <!-- comment --> text\n", "<!-- comment -->"),
        (
            "<!--\n  comment\n\n  more text\n-->\n\nVisible\n",
            "<!--\n  comment\n\n  more text\n-->",
        ),
        (
            "<!--\n  comment\n-->Visible <strong>text</strong>\n",
            "<!--\n  comment\n-->",
        ),
        (
            "<div>Visible\n<!--\n</div><details>comment</details>\n-->\nAfter\n</div>\n",
            "<!--\n</div><details>comment</details>\n-->",
        ),
        (
            "<table><tr><td>Visible <!-- comment --></td></tr></table>\n",
            "<!-- comment -->",
        ),
        (
            "<details><summary>Visible</summary>\n\n<!--\n</details><div>comment</div>\n-->\n\nAfter\n\n</details>\n",
            "<!--\n</details><div>comment</div>\n-->",
        ),
    ] {
        for hidden in [false, true] {
            let stdout = render_comments(input, hidden, &["-c", "120"]);
            assert!(
                stdout.contains("Visible"),
                "input: {input:?}, stdout: {stdout:?}"
            );
            if input.contains("After") {
                assert!(
                    stdout.contains("After"),
                    "input: {input:?}, stdout: {stdout:?}"
                );
            }
            if input.contains("<strong>text</strong>") {
                assert!(
                    stdout.contains("Visible text"),
                    "input: {input:?}, stdout: {stdout:?}"
                );
                assert!(
                    !stdout.contains("<strong>"),
                    "input: {input:?}, stdout: {stdout:?}"
                );
            }
            if hidden {
                for fragment in ["<!--", "comment", "more text", "-->"] {
                    assert!(
                        !stdout.contains(fragment),
                        "input: {input:?}, stdout: {stdout:?}"
                    );
                }
            } else {
                let content = stdout
                    .lines()
                    .map(|line| line.strip_prefix("│  ").unwrap_or(line))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    content.contains(comment),
                    "input: {input:?}, stdout: {stdout:?}"
                );
            }
        }
    }
}

#[test]
fn render_html_comments_preserve_source_line_numbers() {
    for (input, visible, numbered_lines) in [
        (
            "<!--\n  comment\n-->\n\nVisible\n",
            "5 │ Visible",
            &["1 │ <!--", "2 │   comment", "3 │ -->", "5 │ Visible"][..],
        ),
        (
            "<div>\n<!--\ncomment\n-->\nVisible\n</div>\n",
            "5 │ Visible",
            &["2 │ <!--", "3 │ comment", "4 │ -->"][..],
        ),
        (
            "<!-- first -->Visible<!-- last -->\n",
            "1 │ Visible",
            &["1 │ <!-- first -->Visible<!-- last -->"][..],
        ),
        (
            "<!--\ncomment\n-->Visible<!-- last -->\n",
            "3 │ Visible",
            &["1 │ <!--", "2 │ comment", "3 │ -->Visible<!-- last -->"][..],
        ),
    ] {
        for hidden in [false, true] {
            let stdout = render_comments(input, hidden, &["--line-numbers=source;separator"]);
            if hidden {
                assert!(
                    stdout.contains(visible),
                    "input: {input:?}, stdout: {stdout:?}"
                );
                for fragment in ["<!--", "comment", "first", "last", "-->"] {
                    assert!(!stdout.contains(fragment), "stdout: {stdout:?}");
                }
                if input.starts_with("<!--") {
                    assert_eq!(stdout.trim(), visible);
                }
            } else {
                for line in numbered_lines {
                    assert!(stdout.contains(line), "stdout: {stdout:?}");
                }
            }
        }
    }
}

#[test]
fn render_html_comments_wrap_to_column_width() {
    for wrap_mode in ["char", "word"] {
        let stdout = render_comments(
            "<div><!-- This comment contains enough words to wrap across several terminal rows. --></div>\n",
            false,
            &["-c", "40", "-w", wrap_mode],
        );
        assert!(stdout.contains("<!-- This comment"), "stdout: {stdout:?}");
        assert!(
            stdout.replace('\n', "").contains("-->"),
            "stdout: {stdout:?}"
        );
        assert!(
            stdout.lines().all(|line| display_width(line) <= 40),
            "stdout: {stdout:?}"
        );
    }
}

#[test]
fn render_html_comments_preserve_unclosed_source() {
    for hidden in [false, true] {
        let stdout = render_comments("<!--\ncomment\nremaining text\n", hidden, &[]);
        if hidden {
            assert!(stdout.trim().is_empty(), "stdout: {stdout:?}");
        } else {
            assert_eq!(stdout.trim(), "<!--\ncomment\nremaining text");
        }
    }
}

#[test]
fn render_html_comments_keep_table_cell_line_breaks() {
    for table_wrap in ["fit", "none"] {
        let stdout = render_comments(
            "<table><tr><td>Visible <!--\ncomment\n--> text</td></tr></table>\n",
            false,
            &["-c", "120", "--table-wrap", table_wrap],
        );
        assert!(!stdout.starts_with('\n'), "stdout: {stdout:?}");
        assert!(
            stdout.lines().any(|line| line.contains("Visible <!--")),
            "stdout: {stdout:?}"
        );
        assert!(
            stdout.lines().any(|line| line.trim() == "comment"),
            "stdout: {stdout:?}"
        );
        assert!(
            stdout.lines().any(|line| line.contains("--> text")),
            "stdout: {stdout:?}"
        );
    }
}
