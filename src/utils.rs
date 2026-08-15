use regex::regex;
use std::iter::Peekable;
use std::str::Chars;
use unicode_width::UnicodeWidthStr;

/// Calculate the display width of a string, accounting for Unicode characters
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Clean ANSI escape sequences and OSC 8 hyperlink sequences from a string
pub fn strip_ansi(s: &str) -> String {
    if !s.contains('\x1b') {
        return s.to_string();
    }

    let without_ansi = regex!(r"\x1b\[[0-9;]*m").replace_all(s, "");
    regex!(r"\x1b\]8;;[^\x1b]*\x1b\\")
        .replace_all(&without_ansi, "")
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
    let wrap_line = match mode {
        WrapMode::None => return text.to_string(),
        WrapMode::Character => wrap_line_character,
        WrapMode::Word => wrap_line_word,
    };
    if width == 0 {
        return text.to_string();
    }

    let mut wrapped_lines = Vec::new();

    for line in text.split('\n') {
        if line.trim().is_empty() {
            wrapped_lines.push(String::new());
            continue;
        }

        wrapped_lines.extend(wrap_line(line, width));
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
        .into_iter()
        .flat_map(|line| {
            if display_width(&strip_ansi(&line)) > width {
                wrap_line_character(&line, width)
            } else {
                vec![line]
            }
        })
        .collect()
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
mod tests;
