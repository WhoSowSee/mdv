use regex::Regex;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;
use unicode_width::UnicodeWidthStr;

/// Utility functions for mdv

/// Calculate the display width of a string, accounting for Unicode characters
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Truncate a string to fit within a given width, adding ellipsis if needed
pub fn truncate_string(s: &str, max_width: usize) -> String {
    if display_width(s) <= max_width {
        return s.to_string();
    }

    if max_width <= 3 {
        return s.chars().take(max_width).collect();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for ch in s.chars() {
        let char_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if current_width + char_width + 3 > max_width {
            result.push_str("...");
            break;
        }
        result.push(ch);
        current_width += char_width;
    }

    result
}

/// Pad a string to a specific width with spaces
pub fn pad_string(s: &str, width: usize, align: Alignment) -> String {
    let current_width = display_width(s);

    if current_width >= width {
        return s.to_string();
    }

    let padding = width - current_width;

    match align {
        Alignment::Left => format!("{}{}", s, " ".repeat(padding)),
        Alignment::Right => format!("{}{}", " ".repeat(padding), s),
        Alignment::Center => {
            let left_padding = padding / 2;
            let right_padding = padding - left_padding;
            format!(
                "{}{}{}",
                " ".repeat(left_padding),
                s,
                " ".repeat(right_padding)
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// Check if a file is likely to be a text file based on its extension
pub fn is_text_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return matches!(
                ext_str.to_lowercase().as_str(),
                "md" | "markdown"
                    | "mdown"
                    | "txt"
                    | "text"
                    | "rst"
                    | "adoc"
                    | "asciidoc"
                    | "org"
                    | "wiki"
                    | "creole"
                    | "textile"
                    | "rdoc"
                    | "pod"
                    | "man"
                    | "1"
                    | "2"
                    | "3"
                    | "4"
                    | "5"
                    | "6"
                    | "7"
                    | "8"
                    | "9"
                    | "py"
                    | "rs"
                    | "js"
                    | "ts"
                    | "go"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "java"
                    | "rb"
                    | "php"
                    | "pl"
                    | "sh"
                    | "bash"
                    | "zsh"
                    | "fish"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "xml"
                    | "html"
                    | "css"
                    | "sql"
                    | "r"
                    | "m"
                    | "scala"
                    | "clj"
                    | "hs"
                    | "elm"
                    | "ex"
                    | "swift"
                    | "kt"
                    | "dart"
                    | "lua"
                    | "vim"
                    | "el"
                    | "lisp"
                    | "cfg"
                    | "conf"
                    | "ini"
                    | "properties"
                    | "env"
                    | "log"
                    | "diff"
                    | "patch"
            );
        }
    }

    // Also check for files without extension that might be text
    if path.extension().is_none() {
        if let Some(filename) = path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                return matches!(
                    filename_str.to_uppercase().as_str(),
                    "README"
                        | "LICENSE"
                        | "CHANGELOG"
                        | "CONTRIBUTING"
                        | "AUTHORS"
                        | "COPYING"
                        | "INSTALL"
                        | "NEWS"
                        | "TODO"
                        | "HISTORY"
                        | "MAKEFILE"
                        | "DOCKERFILE"
                        | "VAGRANTFILE"
                );
            }
        }
    }

    false
}

/// Detect if content is likely markdown based on common patterns
pub fn is_markdown_content(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().take(20).collect();
    let mut markdown_indicators = 0;

    for line in &lines {
        let trimmed = line.trim();

        // Check for markdown headers
        if trimmed.starts_with('#') && trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') {
            markdown_indicators += 2;
        }

        // Check for markdown lists
        if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || (trimmed.len() > 2
                && trimmed.chars().nth(1) == Some('.')
                && trimmed.chars().nth(2) == Some(' '))
        {
            markdown_indicators += 1;
        }

        // Check for markdown links
        if trimmed.contains("](") || trimmed.contains("[^") {
            markdown_indicators += 1;
        }

        // Check for markdown emphasis
        if trimmed.contains("**")
            || trimmed.contains("__")
            || (trimmed.contains('*') && !trimmed.starts_with('*'))
        {
            markdown_indicators += 1;
        }

        // Check for code blocks
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            markdown_indicators += 2;
        }

        // Check for blockquotes
        if trimmed.starts_with("> ") {
            markdown_indicators += 1;
        }

        // Check for horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            markdown_indicators += 1;
        }

        // Check for tables
        if trimmed.contains('|') && trimmed.len() > 3 {
            markdown_indicators += 1;
        }
    }

    // If we found multiple markdown indicators, it's likely markdown
    markdown_indicators >= 3
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

/// Convert a string to a safe filename by replacing invalid characters
pub fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

/// Get file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Extract the first line of text that looks like a title
pub fn extract_title(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate().take(10) {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for markdown header
        if trimmed.starts_with('#') {
            let title = trimmed.trim_start_matches('#').trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }

        // Check for underlined header (setext style)
        if i + 1 < lines.len() {
            let next_line = lines[i + 1];
            let next_trimmed = next_line.trim();
            if (next_trimmed.chars().all(|c| c == '=') || next_trimmed.chars().all(|c| c == '-'))
                && next_trimmed.len() >= trimmed.len() / 2
            {
                return Some(trimmed.to_string());
            }
        }

        // Return the first non-empty line as title
        return Some(trimmed.to_string());
    }

    None
}

/// Split text into words while preserving whitespace information
pub fn split_preserving_whitespace(text: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_whitespace = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
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

/// Wrap text to fit within specified width, preserving ANSI escape sequences
pub fn wrap_text(text: &str, width: usize) -> String {
    wrap_text_with_mode(text, width, WrapMode::Character)
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
                while let Some(ch) = chars.next() {
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
                    if ch == '\x1b' {
                        if let Some(&following) = chars.peek() {
                            if following == '\\' {
                                sequence.push(chars.next().unwrap());
                                break;
                            }
                        }
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

            if current_width + char_width > width && !current_line.trim().is_empty() {
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

            if current_width + char_width > width && !current_line.trim().is_empty() {
                // Character-based wrapping: force break at current position
                result.push(current_line);
                current_line = ansi_stack.clone();
                current_width = 0;
            }

            current_line.push(ch);
            current_width += char_width;
        }
    }

    if !current_line.trim().is_empty() {
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
            } else if !current_line.trim().is_empty() {
                // Start new line
                result.push(current_line.trim_end().to_string());
                current_line = ansi_stack.clone();
                current_width = 0;
                // Skip leading whitespace on new line
            }
        } else {
            // Handle word
            if current_width + word_width <= width || current_line.trim().is_empty() {
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

    if !current_line.trim().is_empty() {
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

/// Wrap text with indentation support
pub fn wrap_text_with_indent(text: &str, width: usize, indent: usize) -> String {
    wrap_text_with_indent_and_mode(text, width, indent, WrapMode::Character)
}

/// Wrap text with indentation support and specified wrapping mode
pub fn wrap_text_with_indent_and_mode(
    text: &str,
    width: usize,
    indent: usize,
    mode: WrapMode,
) -> String {
    if width <= indent || mode == WrapMode::None {
        return text.to_string();
    }

    let effective_width = width - indent;
    let wrapped = wrap_text_with_mode(text, effective_width, mode);

    // Add indentation to each line
    let indent_str = " ".repeat(indent);
    wrapped
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent_str, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_display_width() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("héllo"), 5);
        assert_eq!(display_width("你好"), 4); // Chinese characters are width 2
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello world", 20), "hello world");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hello", 3), "hel");
    }

    #[test]
    fn test_pad_string() {
        assert_eq!(pad_string("hello", 10, Alignment::Left), "hello     ");
        assert_eq!(pad_string("hello", 10, Alignment::Right), "     hello");
        assert_eq!(pad_string("hello", 10, Alignment::Center), "  hello   ");
    }

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file(&PathBuf::from("test.md")));
        assert!(is_text_file(&PathBuf::from("test.txt")));
        assert!(is_text_file(&PathBuf::from("README")));
        assert!(!is_text_file(&PathBuf::from("test.jpg")));
        assert!(!is_text_file(&PathBuf::from("test.exe")));
    }

    #[test]
    fn test_is_markdown_content() {
        let markdown = "# Title\n\nThis is **bold** text.\n\n- List item\n- Another item";
        assert!(is_markdown_content(markdown));

        let plain_text = "This is just plain text without any markdown formatting.";
        assert!(!is_markdown_content(plain_text));
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
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello/world"), "hello_world");
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_extract_title() {
        let content1 = "# Main Title\n\nSome content here.";
        assert_eq!(extract_title(content1), Some("Main Title".to_string()));

        let content2 = "Main Title\n==========\n\nSome content here.";
        assert_eq!(extract_title(content2), Some("Main Title".to_string()));

        let content3 = "Just some regular text without a clear title.";
        assert_eq!(
            extract_title(content3),
            Some("Just some regular text without a clear title.".to_string())
        );
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a long line that should be wrapped at a specific width to test the wrapping functionality.";
        let wrapped = wrap_text(text, 20);

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
        let wrapped = wrap_text(text, 20);

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
    fn test_wrap_text_with_indent() {
        let text = "This is a long line that should be wrapped with indentation.";
        let wrapped = wrap_text_with_indent(text, 30, 4);

        // Each non-empty line should start with 4 spaces
        for line in wrapped.lines() {
            if !line.trim().is_empty() {
                assert!(
                    line.starts_with("    "),
                    "Line should be indented: '{}'",
                    line
                );
            }
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
            let words: Vec<&str> = line.trim().split_whitespace().collect();
            for word in words {
                assert!(text.contains(word), "Word '{}' should be preserved", word);
            }
        }
    }
}
