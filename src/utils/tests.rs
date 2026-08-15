use super::*;

#[test]
fn test_display_width() {
    assert_eq!(display_width("hello"), 5);
    assert_eq!(display_width("héllo"), 5);
    assert_eq!(display_width("你好"), 4); // Chinese characters are width 2
}

#[test]
fn test_strip_ansi() {
    let colored = "\x1b[31mRed text\x1b[0m";
    assert_eq!(strip_ansi(colored), "Red text");

    // Test OSC 8 hyperlink sequences
    let clickable = "\x1b]8;;https://example.com\x1b\\link text\x1b]8;;\x1b\\";
    assert_eq!(strip_ansi(clickable), "link text");

    // Test combined ANSI and OSC 8
    let combined = "\x1b[31m\x1b]8;;https://example.com\x1b\\red link\x1b]8;;\x1b\\\x1b[0m";
    assert_eq!(strip_ansi(combined), "red link");
}

#[test]
fn test_wrap_text() {
    let text = "This is a long line that should be wrapped at a specific width to test the wrapping functionality.";
    let wrapped = wrap_text_with_mode(text, 20, WrapMode::Character);

    // Check that no line exceeds the width
    for line in wrapped.lines() {
        let clean_line = strip_ansi(line);
        assert!(
            display_width(&clean_line) <= 20,
            "Line too long: '{}'",
            line
        );
    }

    // Check that wrapping occurred (text should be split into multiple lines)
    assert!(
        wrapped.contains('\n'),
        "Text should be wrapped into multiple lines"
    );

    // Check that most characters are preserved (some whitespace may be lost in character wrapping)
    let original_chars = text.chars().filter(|c| !c.is_whitespace()).count();
    let wrapped_chars = wrapped.chars().filter(|c| !c.is_whitespace()).count();
    assert!(
        wrapped_chars >= original_chars - 2,
        "Most characters should be preserved"
    );
}

#[test]
fn test_wrap_text_with_ansi() {
    let text = "\x1b[31mThis is red text that should be wrapped\x1b[0m while preserving colors.";
    let wrapped = wrap_text_with_mode(text, 20, WrapMode::Character);

    // Should contain ANSI codes
    assert!(wrapped.contains("\x1b[31m"));
    assert!(wrapped.contains("\x1b[0m"));

    // Check line lengths
    for line in wrapped.lines() {
        let clean_line = strip_ansi(line);
        assert!(display_width(&clean_line) <= 20);
    }
}

#[test]
fn test_wrap_modes() {
    let text =
        "This is a very long line that should be wrapped differently based on the wrapping mode.";

    // Test character wrapping
    let char_wrapped = wrap_text_with_mode(text, 20, WrapMode::Character);
    assert!(char_wrapped.contains('\n'));

    // Test word wrapping
    let word_wrapped = wrap_text_with_mode(text, 20, WrapMode::Word);
    assert!(word_wrapped.contains('\n'));

    // Test no wrapping
    let no_wrapped = wrap_text_with_mode(text, 20, WrapMode::None);
    assert!(!no_wrapped.contains('\n'));
    assert_eq!(no_wrapped, text);
}

#[test]
fn test_word_wrapping_preserves_words() {
    let text = "Hello world this is a test";
    let wrapped = wrap_text_with_mode(text, 10, WrapMode::Word);

    // Should not break words
    for line in wrapped.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        for word in words {
            assert!(text.contains(word), "Word '{}' should be preserved", word);
        }
    }
}

#[test]
fn test_word_wrap_splits_oversized_ansi_token() {
    let token = "http://bpuser-bP8zwDSb:CO5cB2oXMyupkschZbsT_country-FR_session-V";
    let text = format!("\x1b[31m{token}\x1b[0m");
    let wrapped = wrap_text_with_mode(&text, 20, WrapMode::Word);

    assert!(
        wrapped
            .lines()
            .all(|line| display_width(&strip_ansi(line)) <= 20),
        "wrapped token exceeds width: {wrapped:?}"
    );
    assert_eq!(strip_ansi(&wrapped).replace('\n', ""), token);
}

#[test]
fn test_word_wrap_with_leading_ansi_and_indent_has_no_blank_first_line() {
    let text = "\x1b[31m    abcdefghijk\x1b[0m";
    let wrapped = wrap_text_with_mode(text, 10, WrapMode::Word);
    let mut lines = wrapped.lines();
    let first_line = lines.next().expect("wrapped line");
    let clean_first = strip_ansi(first_line);

    assert!(
        !clean_first.trim().is_empty(),
        "first line must contain visible content: {:?}",
        wrapped
    );
    assert!(
        clean_first.starts_with("    "),
        "leading indentation must be preserved: {:?}",
        wrapped
    );
}
