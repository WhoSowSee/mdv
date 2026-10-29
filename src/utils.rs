use std::iter::Peekable;
use std::str::Chars;
use unicode_width::UnicodeWidthStr;

/// Calculate the display width of a string, accounting for Unicode characters
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

mod ansi;
pub use ansi::{display_width_ansi, strip_ansi};
mod sgr;
pub(crate) use sgr::SgrState;
mod wrapping;
pub(crate) use wrapping::wrap_line_with_widths;

pub(crate) fn escape_html_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
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

    let mut wrapped_lines = Vec::new();

    for line in text.split('\n') {
        wrapped_lines.extend(wrap_line_with_widths(line, width, width, mode));
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

fn is_visually_blank(text: &str) -> bool {
    strip_ansi(text).trim().is_empty()
}

/// Split line into words while preserving ANSI codes
fn split_line_into_words_with_ansi(line: &str) -> Vec<(std::borrow::Cow<'_, str>, bool)> {
    use std::borrow::Cow;
    if !line.contains('') {
        let mut words = Vec::new();
        let mut start = 0;
        let mut whitespace = false;
        for (index, character) in line.char_indices() {
            let next = character.is_whitespace();
            if next != whitespace && index > start {
                words.push((Cow::Borrowed(&line[start..index]), whitespace));
                start = index;
            }
            whitespace = next;
        }
        if start < line.len() {
            words.push((Cow::Borrowed(&line[start..]), whitespace));
        }
        return words;
    }
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut pending_ansi = String::new();
    let mut in_whitespace = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            let sequence = consume_escape_sequence(&mut chars);
            pending_ansi.push_str(&sequence);
        } else if ch.is_whitespace() {
            if !in_whitespace && !current_word.is_empty() {
                result.push((Cow::Owned(std::mem::take(&mut current_word)), false));
            }
            current_word.push_str(&pending_ansi);
            pending_ansi.clear();
            current_word.push(ch);
            in_whitespace = true;
        } else {
            if in_whitespace && !current_word.is_empty() {
                result.push((Cow::Owned(std::mem::take(&mut current_word)), true));
            }
            current_word.push_str(&pending_ansi);
            pending_ansi.clear();
            current_word.push(ch);
            in_whitespace = false;
        }
    }

    current_word.push_str(&pending_ansi);
    if !current_word.is_empty() {
        result.push((Cow::Owned(current_word), in_whitespace));
    }

    result
}

#[cfg(test)]
mod tests;
