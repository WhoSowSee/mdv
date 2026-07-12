use regex::Regex;
use std::iter::Peekable;
use std::str::Chars;
use unicode_width::UnicodeWidthStr;

/// Utility functions for mdv
/// Calculate the display width of a string, accounting for Unicode characters
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Clean ANSI escape sequences and OSC 8 hyperlink sequences from a string
pub fn strip_ansi(s: &str) -> String {
    // Remove standard ANSI color codes
    let ansi_regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let without_ansi = ansi_regex.replace_all(s, "");

    // Remove OSC 8 hyperlink sequences
    // Start sequence: \x1b]8;;URL\x1b\\
    let osc8_start_regex = Regex::new(r"\x1b\]8;;[^\x1b]*\x1b\\").unwrap();
    let without_osc8_start = osc8_start_regex.replace_all(&without_ansi, "");

    // End sequence: \x1b]8;;\x1b\\
    let osc8_end_regex = Regex::new(r"\x1b\]8;;\x1b\\").unwrap();
    osc8_end_regex
        .replace_all(&without_osc8_start, "")
        .to_string()
}

/// Text wrapping mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrapMode {
    /// No wrapping
    None,
    /// Character-based wrapping (default)
    Character,
    /// Word-based wrapping
    Word,
}

/// Wrap text with specified wrapping mode
pub fn wrap_text_with_mode(text: &str, width: usize, mode: WrapMode) -> String {
    if width == 0 || mode == WrapMode::None {
        return text.to_string();
    }

    let lines: Vec<&str> = text.split('\n').collect();
    let mut wrapped_lines = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            wrapped_lines.push(String::new());
            continue;
        }

        let wrapped = match mode {
            WrapMode::None => vec![line.to_string()],
            WrapMode::Character => wrap_line_character(line, width),
            WrapMode::Word => wrap_line_word(line, width),
        };
        wrapped_lines.extend(wrapped);
    }

    wrapped_lines.join("\n")
}

fn consume_escape_sequence(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut sequence = String::from('\x1b');

    if let Some(&next) = chars.peek() {
        match next {
            '[' => {
                sequence.push(chars.next().unwrap());
                for ch in chars.by_ref() {
                    sequence.push(ch);
                    if ('@'..='~').contains(&ch) {
                        break;
                    }
                }
            }
            ']' => {
                sequence.push(chars.next().unwrap());
                while let Some(ch) = chars.next() {
                    sequence.push(ch);
                    if ch == '\x07' {
                        break;
                    }
                    if ch == '\x1b'
                        && let Some(&following) = chars.peek()
                        && following == '\\'
                    {
                        sequence.push(chars.next().unwrap());
                        break;
                    }
                }
            }
            _ => {
                sequence.push(chars.next().unwrap());
            }
        }
    }

    sequence
}

fn is_sgr_sequence(sequence: &str) -> bool {
    sequence.starts_with("\x1b[") && sequence.ends_with('m')
}

fn is_sgr_reset(sequence: &str) -> bool {
    if !is_sgr_sequence(sequence) {
        return false;
    }

    let inner = &sequence[2..sequence.len().saturating_sub(1)];
    inner
        .split(';')
        .any(|param| param.trim().is_empty() || param.trim() == "0")
}

fn is_visually_blank(text: &str) -> bool {
    strip_ansi(text).trim().is_empty()
}

/// Wrap a single line using character-based wrapping, handling ANSI codes
fn wrap_line_character(line: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![line.to_string()];
    }

    // Check if line fits without wrapping
    let clean_line = strip_ansi(line);
    if display_width(&clean_line) <= width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    let mut ansi_stack = String::new(); // Track active ANSI codes

    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Start of ANSI/OSC escape sequence
            let sequence = consume_escape_sequence(&mut chars);
            current_line.push_str(&sequence);

            if is_sgr_sequence(&sequence) {
                if is_sgr_reset(&sequence) {
                    ansi_stack.clear();
                } else {
                    ansi_stack.push_str(&sequence);
                }
            }
        } else if ch.is_whitespace() {
            // Handle whitespace - good breaking point
            let char_width = if ch == '\t' { 4 } else { 1 };

            if current_width + char_width > width && !is_visually_blank(&current_line) {
                // Need to wrap before this whitespace
                result.push(current_line.trim_end().to_string());
                current_line = ansi_stack.clone(); // Start new line with active ANSI codes
                current_width = 0;
            } else {
                current_line.push(ch);
                current_width += char_width;
            }
        } else {
            // Regular character
            let char_width = UnicodeWidthStr::width(ch.to_string().as_str());

            if current_width + char_width > width && !is_visually_blank(&current_line) {
                // Character-based wrapping: force break at current position
                result.push(current_line);
                current_line = ansi_stack.clone();
                current_width = 0;
            }

            current_line.push(ch);
            current_width += char_width;
        }
    }

    if !is_visually_blank(&current_line) {
        result.push(current_line);
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

/// Wrap a single line using word-based wrapping, handling ANSI codes
fn wrap_line_word(line: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![line.to_string()];
    }

    // Check if line fits without wrapping
    let clean_line = strip_ansi(line);
    if display_width(&clean_line) <= width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    let mut ansi_stack = String::new(); // Track active ANSI codes

    // Split into words while preserving ANSI codes
    let words = split_line_into_words_with_ansi(line);

    for (word, is_whitespace) in words {
        let clean_word = strip_ansi(&word);
        let word_width = display_width(&clean_word);

        // Update ANSI stack
        if word.contains('\x1b') {
            update_ansi_stack(&mut ansi_stack, &word);
        }

        if is_whitespace {
            // Handle whitespace
            if current_width + word_width <= width {
                current_line.push_str(&word);
                current_width += word_width;
            } else if !is_visually_blank(&current_line) {
                // Start new line
                result.push(current_line.trim_end().to_string());
                current_line = ansi_stack.clone();
                current_width = 0;
                // Skip leading whitespace on new line
            }
        } else {
            // Handle word
            if current_width + word_width <= width || is_visually_blank(&current_line) {
                current_line.push_str(&word);
                current_width += word_width;
            } else {
                // Word doesn't fit, start new line
                result.push(current_line.trim_end().to_string());
                current_line = format!("{}{}", ansi_stack, word);
                current_width = word_width;
            }
        }
    }

    if !is_visually_blank(&current_line) {
        result.push(current_line);
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

/// Split line into words while preserving ANSI codes
fn split_line_into_words_with_ansi(line: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_whitespace = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            let sequence = consume_escape_sequence(&mut chars);
            current_word.push_str(&sequence);
        } else if ch.is_whitespace() {
            if !in_whitespace && !current_word.is_empty() {
                result.push((current_word.clone(), false));
                current_word.clear();
            }
            current_word.push(ch);
            in_whitespace = true;
        } else {
            if in_whitespace && !current_word.is_empty() {
                result.push((current_word.clone(), true));
                current_word.clear();
            }
            current_word.push(ch);
            in_whitespace = false;
        }
    }

    if !current_word.is_empty() {
        result.push((current_word, in_whitespace));
    }

    result
}

/// Update ANSI stack with new codes from a word
fn update_ansi_stack(ansi_stack: &mut String, word: &str) {
    let mut chars = word.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            let sequence = consume_escape_sequence(&mut chars);

            if is_sgr_sequence(&sequence) {
                if is_sgr_reset(&sequence) {
                    ansi_stack.clear();
                } else {
                    ansi_stack.push_str(&sequence);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
        let text =
            "\x1b[31mThis is red text that should be wrapped\x1b[0m while preserving colors.";
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
        let text = "This is a very long line that should be wrapped differently based on the wrapping mode.";

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
}
