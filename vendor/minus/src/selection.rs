use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const RESET: &str = "\x1b[0m";
const SELECTION_STYLE: &str = "\x1b[0;38;2;143;147;162;48;2;31;34;51m";

pub fn char_index_at_display_column(line: &str, target_column: usize) -> usize {
    let mut display_column: usize = 0;

    for (byte_index, grapheme) in line.grapheme_indices(true) {
        let width = grapheme.width();
        if target_column < display_column.saturating_add(width.max(1)) {
            return line[..byte_index].chars().count();
        }
        display_column = display_column.saturating_add(width);
    }

    line.chars().count()
}

pub fn grapheme_end_char_index(line: &str, char_index: usize) -> usize {
    let visible = strip_ansi(line);
    let mut grapheme_start = 0;

    for grapheme in visible.graphemes(true) {
        let grapheme_end = grapheme_start + grapheme.chars().count();
        if char_index < grapheme_end {
            return grapheme_end;
        }
        grapheme_start = grapheme_end;
    }

    grapheme_start
}

pub fn highlight_visible_range(line: Cow<'_, str>, start: usize, end: usize) -> Cow<'_, str> {
    if start >= end {
        return line;
    }

    let end = grapheme_end_char_index(&line, end - 1);

    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len() + SELECTION_STYLE.len() + RESET.len());
    let mut sgr_history = String::new();
    let mut byte_index = 0;
    let mut visible_index = 0;
    let mut highlighted = false;

    while byte_index < bytes.len() {
        if let Some((sequence_end, is_sgr)) = ansi_sequence_end(bytes, byte_index) {
            if highlighted && visible_index >= end {
                restore_original_style(&mut out, &sgr_history);
                highlighted = false;
            }
            let sequence = &line[byte_index..sequence_end];
            out.push_str(sequence);
            if is_sgr {
                sgr_history.push_str(sequence);
                if highlighted {
                    out.push_str(SELECTION_STYLE);
                }
            }
            byte_index = sequence_end;
            continue;
        }

        if !highlighted && visible_index == start {
            out.push_str(SELECTION_STYLE);
            highlighted = true;
        }
        if highlighted && visible_index == end {
            restore_original_style(&mut out, &sgr_history);
            highlighted = false;
        }

        let character = line[byte_index..].chars().next().unwrap();
        out.push(character);
        visible_index = visible_index.saturating_add(1);
        byte_index += character.len_utf8();
    }

    if highlighted {
        out.push_str(RESET);
    }

    out.into()
}

fn restore_original_style(out: &mut String, sgr_history: &str) {
    out.push_str(RESET);
    out.push_str(sgr_history);
}

pub fn strip_ansi(line: &str) -> Cow<'_, str> {
    if !line.as_bytes().contains(&b'\x1b') {
        return Cow::Borrowed(line);
    }

    let bytes = line.as_bytes();
    let mut visible = String::with_capacity(line.len());
    let mut byte_index = 0;
    while byte_index < bytes.len() {
        if let Some((sequence_end, _)) = ansi_sequence_end(bytes, byte_index) {
            byte_index = sequence_end;
            continue;
        }

        let character = line[byte_index..].chars().next().unwrap();
        visible.push(character);
        byte_index += character.len_utf8();
    }
    Cow::Owned(visible)
}

pub fn ansi_sequence_end(bytes: &[u8], start: usize) -> Option<(usize, bool)> {
    if bytes.get(start) != Some(&b'\x1b') {
        return None;
    }

    match bytes.get(start + 1) {
        Some(b'[') => {
            let mut index = start + 2;
            while let Some(&byte) = bytes.get(index) {
                index += 1;
                if (0x40..=0x7e).contains(&byte) {
                    return Some((index, byte == b'm'));
                }
            }
            Some((bytes.len(), false))
        }
        Some(b']') => {
            let mut index = start + 2;
            while let Some(&byte) = bytes.get(index) {
                if byte == b'\x07' {
                    return Some((index + 1, false));
                }
                if byte == b'\x1b' && bytes.get(index + 1) == Some(&b'\\') {
                    return Some((index + 2, false));
                }
                index += 1;
            }
            Some((bytes.len(), false))
        }
        Some(_) => Some(((start + 2).min(bytes.len()), false)),
        None => Some((bytes.len(), false)),
    }
}
