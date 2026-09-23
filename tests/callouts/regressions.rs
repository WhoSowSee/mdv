use super::render;

#[test]
fn quoted_list_items_do_not_activate_callouts() {
    for source in [
        "> - [!note] First\n> - Second\n",
        "> 1. [!warning] First\n> 2. Second\n",
    ] {
        for style in ["simple", "pretty"] {
            let output = render(source, style, &[]);
            assert!(
                output.contains("First") && output.contains("Second"),
                "{output}"
            );
            assert!(
                output.contains("[!note]") || output.contains("[!warning]"),
                "{output}"
            );
            assert!(!output.contains('┃') && !output.contains('╭'), "{output}");
        }
    }
}

#[test]
fn actual_callouts_still_work_in_lists_containing_quoted_lists() {
    let source = "- First\n\n  > - [!note] Literal item\n  > - Second literal item\n\n  > [!tip] Actual callout\n  > Visible body\n\n- Last\n";
    for style in ["simple", "pretty"] {
        let output = render(source, style, &[]);
        for text in [
            "[!note] Literal item",
            "Second literal item",
            "Actual callout",
            "Visible body",
            "Last",
        ] {
            assert!(output.contains(text), "{output}");
        }
    }
}

#[test]
fn unclosed_openers_complete_without_exponential_search() {
    for (opener, closer) in [
        (":::note\n", ":::\n"),
        ("/// note\n", "///\n"),
        ("{% hint style=\"info\" %}\n", "{% endhint %}\n"),
    ] {
        let output = render(
            &format!("{}\nTail sentinel\n", opener.repeat(1024)),
            "simple",
            &[],
        );
        assert!(output.contains("Tail sentinel"), "{output}");
        let output = render(
            &format!(
                "{}Body sentinel\n{closer}\nTail sentinel\n",
                opener.repeat(64)
            ),
            "simple",
            &[],
        );
        assert!(
            output.contains("Body sentinel") && output.contains("Tail sentinel"),
            "{output}"
        );
    }
}

#[test]
fn multiline_inline_code_protects_callout_markers() {
    for body in [
        "/// note\nLiteral body\n///",
        "{% hint style=\"info\" %}\nLiteral body\n{% endhint %}",
        "??? note\n    Literal body",
    ] {
        let source =
            format!("Before `literal\n{body}\nend` after.\n\n:::tip[Actual]\nReal body\n:::\n");
        let html = render(&source, "simple", &["--html"]);
        assert!(
            html.contains("<code>literal ") && html.contains(" end</code>"),
            "{html}"
        );
        assert!(html.contains("Literal body"), "{html}");
        assert_eq!(html.matches("<blockquote>").count(), 1, "{html}");
        assert!(
            html.contains("[!tip] Actual") && html.contains("Real body"),
            "{html}"
        );
    }
}

#[test]
fn inline_code_cannot_close_an_outer_callout() {
    let source = "::::note[Outer]\nBefore `literal\n::::\nend` after.\n\n:::tip[Inner]\nInner body\n:::\n\nOuter tail\n::::\nOutside\n";
    let output = render(source, "simple", &[]);
    assert!(
        output.contains("[Outer]") && output.contains("[Inner]"),
        "{output}"
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with('┃') && line.contains("Outer tail")),
        "{output}"
    );
    assert!(
        output.lines().any(|line| line.trim() == "Outside"),
        "{output}"
    );

    let source =
        "::::note[Outer]\n- Before `literal\n  :::: \n  end` after.\n\nOuter tail\n::::\nOutside\n";
    let output = render(source, "simple", &[]);
    assert!(
        output
            .lines()
            .any(|line| line.starts_with('┃') && line.contains("Outer tail")),
        "{output}"
    );
    assert!(
        output.lines().any(|line| line.trim() == "Outside"),
        "{output}"
    );
}

#[test]
fn heading_titles_preserve_literal_and_escaped_hashes() {
    for (heading, title) in [
        ("C#", "C#"),
        ("F# ###", "F#"),
        ("Escaped \\#", "Escaped #"),
        ("Title ###", "Title"),
        ("Use `#`", "Use #"),
    ] {
        for source in [
            format!("::: {{.callout-note}}\n## {heading}\nBody.\n:::\n"),
            format!("/// tip | ### {heading}\nBody.\n///\n"),
        ] {
            let output = render(&source, "simple", &[]);
            assert!(
                output.contains(title) && output.contains("Body."),
                "heading {heading:?}:\n{output}"
            );
            assert!(!output.contains("###"), "{output}");
        }
    }
}
