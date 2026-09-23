use super::render;

#[test]
fn all_dialects_render_titles_bodies_and_nested_blocks() {
    let style = "simple";
    let output = render(include_str!("../files/callout-dialects.md"), style, &[]);
    for expected in [
        "MkDocs title",
        "MkDocs first paragraph.",
        "MkDocs second paragraph.",
        "Nested MkDocs",
        "Docusaurus title",
        "Nested Docusaurus",
        "VitePress title",
        "VitePress details",
        "MyST title",
        "MyST backtick body.",
        "Quarto title",
        "PyMdown title",
        "PyMdown details",
        "GitBook body.",
        "Nested GitBook body.",
        "After GitBook.",
    ] {
        assert!(
            output.contains(expected),
            "missing {expected:?} in {style}:\n{output}"
        );
    }
    for raw in [
        "!!!",
        "???",
        ":::warning",
        ":::{",
        "///",
        "{%",
        ":class:",
        ":name:",
        "type: danger",
        "attrs:",
        "MDVCALLOUTMETA",
        "MDV_CALLOUT_OPTIONS",
    ] {
        assert!(!output.contains(raw), "raw {raw:?} in {style}:\n{output}");
    }
    assert!(
        output
            .lines()
            .any(|line| line.contains("┃┃") && line.contains("Nested Docusaurus body.")),
        "{output}"
    );
    for after in ["After MkDocs.", "After Docusaurus.", "After GitBook."] {
        assert!(output.lines().any(|line| line.trim() == after), "{output}");
    }
}

#[test]
fn titleless_blocks_and_icon_options_keep_the_body() {
    let input = "!!! note \"\"\n    First body.\n\n::: {.callout-warning title=\"No icon\" icon=false}\nSecond body.\n:::\n";
    for style in [
        "simple:show-simple-icons",
        "pretty:show-simple-icons",
        "pretty:label-inside;show-simple-icons",
    ] {
        let output = render(input, style, &[]);
        assert!(
            output.contains("First body.") && output.contains("Second body."),
            "{output}"
        );
        assert!(
            !output.contains("Note") && !output.contains("[i]") && !output.contains("[!]"),
            "{output}"
        );
        assert!(output.contains("No icon"), "{output}");
    }
}

#[test]
fn fold_states_are_preserved_across_dialects() {
    let input = "??? warning \"Closed\"\n    Visible body.\n\n???+ note \"Opened\"\n    Visible body.\n\n::: details Details\nVisible body.\n:::\n\n/// details | Expanded\n    open: True\n\nVisible body.\n///\n";
    let output = render(input, "simple:show-icons;fold-icons", &[]);
    for expected in ["Closed ", "Opened ", "Details ", "Expanded "] {
        assert!(output.contains(expected), "{output}");
    }
    assert_eq!(output.matches("Visible body.").count(), 4, "{output}");
}

#[test]
fn fences_and_html_protect_literal_admonition_examples() {
    let input = "```rust\n!!! warning \"Literal\"\n    Body\n:::tip[Literal]\n:::\n/// note | Literal\n///\n{% hint style=\"info\" %}\n{% endhint %}\n```\n\n<!--\n::: warning Comment\n:::\n-->\n\n::: note Outer\n```rust\n:::\n!!! note Literal\n```\nAfter code.\n:::\n";
    let output = render(input, "simple", &[]);
    for expected in [
        "!!! warning",
        ":::tip[Literal]",
        "/// note | Literal",
        "{% hint",
        "After code.",
    ] {
        assert!(output.contains(expected), "{output}");
    }
    assert_eq!(output.matches("[Outer]").count(), 1, "{output}");
}

#[test]
fn dialects_work_inside_markdown_lists_and_quotes() {
    let input = "> ::: warning Quoted\n> Quoted body.\n> :::\n\n- Item\n\n  !!! tip \"Listed\"\n      Listed body.\n\n- Next item\n";
    let output = render(input, "simple", &[]);
    for expected in [
        "[Quoted]",
        "Quoted body.",
        "[Listed]",
        "Listed body.",
        "Next item",
    ] {
        assert!(output.contains(expected), "{output}");
    }
    assert!(
        !output.contains("!!!") && !output.contains(":::"),
        "{output}"
    );
}

#[test]
fn export_has_no_internal_callout_metadata() {
    let output = render(
        include_str!("../files/callout-dialects.md"),
        "simple",
        &["--html"],
    );
    assert!(output.contains("<blockquote>"), "{output}");
    assert!(
        !output.contains("MDVCALLOUTMETA") && !output.contains("MDV_CALLOUT_OPTIONS"),
        "{output}"
    );
}

#[test]
fn structured_bodies_match_native_callout_rendering() {
    let body = "Paragraph with [link](https://example.com).\n\n- One\n- Two\n\n| A | B |\n| --- | --- |\n| left | right |\n\n```rust\nlet value = 7;\n```\n";
    let quoted = body
        .lines()
        .map(|line| format!("> {line}\n"))
        .collect::<String>();
    let indented = body
        .lines()
        .map(|line| format!("    {line}\n"))
        .collect::<String>();
    let inputs = [
        format!("!!! warning \"Shared\"\n{indented}"),
        format!(":::warning[Shared]\n{body}:::\n"),
        format!("::: warning Shared\n{body}:::\n"),
        format!(":::{{admonition}} Shared\n:class: warning\n\n{body}:::\n"),
        format!("::: {{.callout-warning title=\"Shared\"}}\n{body}:::\n"),
        format!("/// warning | Shared\n{body}///\n"),
    ];
    let style = "pretty";
    let native = render(&format!("> [!warning] Shared\n{quoted}"), style, &[]);
    for input in &inputs {
        assert_eq!(render(input, style, &[]), native, "{input}");
    }
    let native = render(&format!("> [!warning]\n{quoted}"), style, &[]);
    assert_eq!(
        render(
            &format!("{{% hint style='warning' %}}\n{body}{{% endhint %}}"),
            style,
            &[]
        ),
        native
    );
}

#[test]
fn attribute_options_and_rich_titles_are_not_body_text() {
    let input = "::: details Open details {open #details-id}\nOpen body.\n:::\n\n:::{admonition} YAML title\n---\nclass: tip dropdown toggle-shown\nname: yaml-id\n---\nYAML body.\n:::\n\n:::note[Read [docs](https://example.com) and `code`]\nTitle body.\n:::\n\n!!! info inline end\n    Inline body.\n";
    let output = render(input, "simple:show-icons;fold-icons", &[]);
    for text in [
        "Open details ",
        "YAML title ",
        "Read docs and code",
        "Inline body.",
    ] {
        assert!(output.contains(text), "{output}");
    }
    for raw in ["{open", "details-id", "yaml-id", "class:", "inline end"] {
        assert!(!output.contains(raw), "{output}");
    }
}

#[test]
fn source_numbers_follow_body_lines_after_metadata_removal() {
    let input = "::: {.callout-tip}\n## Title\nBody\n:::\nAfter\n\n:::{note}\n:name: target\n\nLast body\n:::\n";
    let output = render(input, "simple", &["--line-numbers=source"]);
    for (text, number) in [("Body", "3"), ("After", "5"), ("Last body", "10")] {
        let line = output.lines().find(|line| line.contains(text)).unwrap();
        assert!(line.trim_start().starts_with(number), "{output}");
    }
}
