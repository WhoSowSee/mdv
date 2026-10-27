use super::{
    WrapMode, consume_escape_sequence, display_width, display_width_ansi, is_sgr_reset,
    is_sgr_sequence, is_visually_blank, split_line_into_words_with_ansi, strip_ansi,
    update_ansi_stack,
};
use unicode_width::UnicodeWidthStr;

pub(crate) fn wrap_line_with_widths(
    line: &str,
    initial_width: usize,
    continuation_width: usize,
    mode: WrapMode,
) -> Vec<String> {
    match mode {
        WrapMode::None => vec![line.to_string()],
        _ if line.trim().is_empty() => vec![String::new()],
        WrapMode::Character => wrap_line_character(line, initial_width, continuation_width),
        WrapMode::Word => wrap_line_word(line, initial_width, continuation_width),
    }
}

fn wrap_line_character(line: &str, mut width: usize, continuation_width: usize) -> Vec<String> {
    let clean_line = strip_ansi(line);
    if display_width(&clean_line) <= width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    let mut ansi_stack = String::new();
    let mut pending_escape_start = None;

    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            let sequence = consume_escape_sequence(&mut chars);
            pending_escape_start.get_or_insert(current_line.len());
            current_line.push_str(&sequence);

            if is_sgr_sequence(&sequence) {
                if is_sgr_reset(&sequence) {
                    ansi_stack.clear();
                } else {
                    ansi_stack.push_str(&sequence);
                }
            }
        } else if ch.is_whitespace() {
            let char_width = if ch == '\t' { 4 } else { 1 };

            if current_width + char_width > width && !is_visually_blank(&current_line) {
                let pending = pending_escape_start
                    .take()
                    .map(|start| current_line.split_off(start))
                    .unwrap_or_default();
                result.push(current_line.trim_end().to_string());
                current_line = if pending.is_empty() {
                    ansi_stack.clone()
                } else {
                    pending
                };
                current_width = 0;
                width = continuation_width;
            } else {
                current_line.push(ch);
                current_width += char_width;
            }
            pending_escape_start = None;
        } else {
            let char_width = UnicodeWidthStr::width(ch.to_string().as_str());

            if current_width + char_width > width && !is_visually_blank(&current_line) {
                let pending = pending_escape_start
                    .take()
                    .map(|start| current_line.split_off(start))
                    .unwrap_or_default();
                result.push(current_line);
                current_line = if pending.is_empty() {
                    ansi_stack.clone()
                } else {
                    pending
                };
                current_width = 0;
                width = continuation_width;
            }

            current_line.push(ch);
            current_width += char_width;
            pending_escape_start = None;
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

fn wrap_line_word(line: &str, initial_width: usize, continuation_width: usize) -> Vec<String> {
    let mut width = initial_width;
    let clean_line = strip_ansi(line);
    if display_width(&clean_line) <= width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    let mut ansi_stack = String::new();

    let words = split_line_into_words_with_ansi(line);

    for (word, is_whitespace) in words {
        let clean_word = strip_ansi(&word);
        let word_width = display_width(&clean_word);

        if word.contains('\x1b') {
            update_ansi_stack(&mut ansi_stack, &word);
        }

        if is_whitespace {
            if current_width + word_width <= width {
                current_line.push_str(&word);
                current_width += word_width;
            } else if !is_visually_blank(&current_line) {
                result.push(current_line.trim_end().to_string());
                current_line = ansi_stack.clone();
                current_width = 0;
                width = continuation_width;
            }
        } else {
            if current_width + word_width <= width || is_visually_blank(&current_line) {
                current_line.push_str(&word);
                current_width += word_width;
            } else {
                result.push(current_line.trim_end().to_string());
                current_line = format!("{}{}", ansi_stack, word);
                current_width = word_width;
                width = continuation_width;
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
        .enumerate()
        .flat_map(|(index, line)| {
            let width = if index == 0 {
                initial_width
            } else {
                continuation_width
            };
            if display_width_ansi(&line) > width {
                wrap_line_character(&line, width, continuation_width)
            } else {
                vec![line]
            }
        })
        .collect()
}
