use super::*;

#[test]
fn code_fences_are_not_normalized() {
    let input = "```markdown\n!!! warning \"Literal\"\n    Body\n:::tip[Literal]\n:::\n```";
    let processor = MarkdownProcessor::new(&crate::config::Config::default());
    let mut context = Admonitions::new(input);
    let output = processor.convert_admonitions_to_callouts(input, None, &mut context);
    assert_eq!(output, input);
}

#[test]
fn invalid_and_unclosed_blocks_remain_literal() {
    for input in [
        ":::warning[Unclosed title\nBody\n:::",
        "::: {.callout-warning title=\"unfinished}\nBody\n:::",
        "{% hint style=\"unknown\" %}\nBody\n{% endhint %}",
        ":::note\nBody",
        "/// warning | Title\nBody",
        "    !!! warning\n        Code",
        "`:::warning[inline]`",
    ] {
        let processor = MarkdownProcessor::new(&crate::config::Config::default());
        let mut context = Admonitions::new(input);
        assert_eq!(
            processor.convert_admonitions_to_callouts(input, None, &mut context),
            input
        );
    }
}

#[test]
fn myst_fences_protect_nested_code_and_keep_following_content() {
    let input =
        "````{note}\n```rust\n!!! warning Literal\n:::tip\n:::\n```\nAfter code.\n````\nOutside.";
    let processor = MarkdownProcessor::new(&crate::config::Config::default());
    let events = processor.parse(input).unwrap();
    assert!(
        events.iter().any(
            |event| matches!(event, Event::Text(text) if text.contains("!!! warning Literal"))
        )
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::Text(text) if text.contains("After code.")))
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::Start(pulldown_cmark::Tag::BlockQuote(_))))
            .count(),
        1
    );
}

#[test]
fn pymdown_options_stop_at_the_first_blank_line() {
    let input = "/// note\n\n    type: warning\n///";
    let processor = MarkdownProcessor::new(&crate::config::Config::default());
    let mut context = Admonitions::new(input);
    let output = processor.convert_admonitions_to_callouts(input, None, &mut context);
    assert!(output.contains("[!note]"));
    assert!(output.contains("type: warning"));
}
#[test]
fn callout_free_content_preserves_line_endings_and_source_numbers() {
    let input = "first\r\n\r\n> text\r\n\r\n- item\r\n";
    let processor = MarkdownProcessor::new(&crate::config::Config::default());
    let mut context = Admonitions::new(input);
    let mut sources = vec![Some(1), Some(2), Some(3), Some(4), Some(5)];
    assert_eq!(
        processor.convert_admonitions_to_callouts(input, Some(&mut sources), &mut context),
        "first\n\n> text\n\n- item"
    );
    assert_eq!(sources, [Some(1), Some(2), Some(3), Some(4), Some(5)]);
}
