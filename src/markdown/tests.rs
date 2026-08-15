use super::*;
use crate::config::Config;
use pulldown_cmark::HeadingLevel;

#[test]
fn test_markdown_parsing() {
    let config = Config::default();
    let processor = MarkdownProcessor::new(&config);

    let markdown = "# Hello\n\nThis is **bold** text.";
    let events = processor.parse(markdown).unwrap();

    assert!(!events.is_empty());
}

#[test]
fn test_filter_from_text() {
    let content = "Line 1\nTarget Line\nLine 3\nLine 4";
    let lines: Vec<&str> = content.lines().collect();
    let result = lines[MarkdownProcessor::filter_line_range(&lines, "Target:2")].join("\n");

    assert_eq!(result, "Target Line\nLine 3");
}

#[test]
fn test_extract_code_language() {
    let fenced = CodeBlockKind::Fenced("rust".into());
    assert_eq!(extract_code_language(&fenced), Some("rust".to_string()));

    let indented = CodeBlockKind::Indented;
    assert_eq!(extract_code_language(&indented), None);
}

#[test]
fn test_detect_source_code() {
    // Test file extension detection
    assert_eq!(
        detect_source_code("", Some("test.rs")),
        Some("rust".to_string())
    );
    assert_eq!(
        detect_source_code("", Some("test.py")),
        Some("python".to_string())
    );

    // Test content detection
    let python_code = "def hello():\n    print('world')";
    assert_eq!(
        detect_source_code(python_code, None),
        Some("python".to_string())
    );

    let rust_code = "fn main() {\n    println!(\"Hello\");\n}";
    assert_eq!(
        detect_source_code(rust_code, None),
        Some("rust".to_string())
    );
}

#[test]
fn setext_dashes_create_h2() {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let config = Config::default();
    let processor = MarkdownProcessor::new(&config);

    let markdown = "Text\n---\nNext\n";
    let events = processor.parse(markdown).unwrap();

    assert!(matches!(
        events.as_slice(),
        [
            Event::Start(Tag::Heading { level: HeadingLevel::H2, .. }),
            Event::Text(first),
            Event::End(TagEnd::Heading(HeadingLevel::H2)),
            Event::Start(Tag::Paragraph),
            Event::Text(second),
            Event::End(TagEnd::Paragraph)
        ] if first.as_ref() == "Text" && second.as_ref() == "Next"
    ));
}

#[test]
fn setext_equals_create_h1() {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let config = Config::default();
    let processor = MarkdownProcessor::new(&config);

    let markdown = "Title\n===\nBody\n";
    let events = processor.parse(markdown).unwrap();

    assert!(matches!(
        events.as_slice(),
        [
            Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }),
            Event::Text(first),
            Event::End(TagEnd::Heading(HeadingLevel::H1)),
            Event::Start(Tag::Paragraph),
            Event::Text(second),
            Event::End(TagEnd::Paragraph)
        ] if first.as_ref() == "Title" && second.as_ref() == "Body"
    ));
}

#[test]
fn top_level_tab_indented_text_is_not_code_block() {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let config = Config::default();
    let processor = MarkdownProcessor::new(&config);
    let events = processor.parse("\tTest text\n").unwrap();

    assert!(matches!(
        events.as_slice(),
        [
            Event::Start(Tag::Paragraph),
            Event::Text(text),
            Event::End(TagEnd::Paragraph)
        ] if text.as_ref() == "Test text"
    ));
}

#[test]
fn top_level_space_indented_text_is_not_code_block() {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let config = Config::default();
    let processor = MarkdownProcessor::new(&config);
    let events = processor.parse("    Test text\n").unwrap();

    assert!(matches!(
        events.as_slice(),
        [
            Event::Start(Tag::Paragraph),
            Event::Text(text),
            Event::End(TagEnd::Paragraph)
        ] if text.as_ref() == "Test text"
    ));
}

#[test]
fn top_level_indented_atx_headings_are_headings() {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let config = Config::default();
    let processor = MarkdownProcessor::new(&config);
    let events = processor
        .parse("    # Space heading\n\t# Tab heading\n")
        .unwrap();

    assert!(matches!(
        events.as_slice(),
        [
            Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }),
            Event::Text(first),
            Event::End(TagEnd::Heading(HeadingLevel::H1)),
            Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }),
            Event::Text(second),
            Event::End(TagEnd::Heading(HeadingLevel::H1))
        ] if first.as_ref() == "Space heading" && second.as_ref() == "Tab heading"
    ));
}
