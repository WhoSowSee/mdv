use super::render;

#[test]
fn fenced_code_in_list_items_does_not_close_the_outer_callout() {
    for body in [
        "- ```rust\n  ::::\n  fn main() {}\n  ```",
        "1. ```rust\n   ::::\n   fn main() {}\n   ```",
        "- > ```rust\n  > ::::\n  > fn main() {}\n  > ```",
        "- ```rust\n  ::::\n  fn main() {}\n  ```\n\n  :::tip[Inner]\n  Inner body.\n  :::",
    ] {
        let source = format!("::::note[Outer]\n{body}\n\nOuter tail.\n::::\nOutside.\n");
        let html = render(&source, "simple", &["--html"]);
        assert!(
            html.contains("<code class=\"language-rust\">::::\nfn main() {}\n</code>"),
            "{html}"
        );
        let closing = html.rfind("</blockquote>").unwrap();
        assert!(html.find("Outer tail.").unwrap() < closing, "{html}");
        assert!(html.find("Outside.").unwrap() > closing, "{html}");
        if body.contains("Inner body.") {
            assert!(
                html.contains("[!tip] Inner") && html.contains("Inner body."),
                "{html}"
            );
        }
    }
}

#[test]
fn quarto_keeps_indented_heading_text_in_the_body() {
    for indent in ["    ", "\t"] {
        for (attributes, native_title) in
            [("", ""), (" title=\"Explicit title\"", " Explicit title")]
        {
            let source = format!(
                "::: {{.callout-note{attributes}}}\n\n{indent}## Literal code heading\n\nBody.\n:::\n"
            );
            let native = format!(
                "> [!note]{native_title}\n>\n> {indent}## Literal code heading\n>\n> Body.\n"
            );
            for extra in [&[][..], &["--html"][..]] {
                assert_eq!(
                    render(&source, "simple", extra),
                    render(&native, "simple", extra),
                    "{source}"
                );
            }
        }
    }
    let source = "::: {.callout-note}\n   ## Real heading\nBody.\n:::\n";
    let output = render(source, "simple", &[]);
    assert!(
        output.contains("[Real heading]") && !output.contains("##"),
        "{output}"
    );
}

#[test]
fn legacy_bang_blocks_stop_at_the_parent_closing_fence() {
    for (opening, closing) in [
        (":::note[Outer]", ":::"),
        ("/// note | Outer", "///"),
        ("{% hint style=\"info\" %}", "{% endhint %}"),
        ("````{note} Outer", "````"),
    ] {
        let source = format!("{opening}\n!!! tip Legacy title\nChild.\n{closing}\nAfter.\n");
        let html = render(&source, "simple", &["--html"]);
        assert_eq!(html.matches("<blockquote>").count(), 2, "{html}");
        assert!(
            html.find("After.").unwrap() > html.rfind("</blockquote>").unwrap(),
            "{html}"
        );
        assert!(
            html.contains("[!tip] Legacy title") && html.contains("Child."),
            "{html}"
        );
    }
}
