use std::{borrow::Cow, ops::Range};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::selection::{ansi_sequence_end, strip_ansi};

pub(super) fn line_width(line: &str) -> usize {
    strip_ansi(line)
        .graphemes(true)
        .map(UnicodeWidthStr::width)
        .sum()
}

pub fn horizontal_scroll_view(
    line: &str,
    columns: usize,
    left: usize,
    prefix_width: usize,
) -> (Cow<'_, str>, usize) {
    let visible = strip_ansi(line);
    let mut graphemes = visible.grapheme_indices(true);
    let mut prefix_end = 0;
    let mut prefix_columns: usize = 0;
    while prefix_columns < prefix_width {
        let Some((index, grapheme)) = graphemes.next() else {
            break;
        };
        prefix_columns = prefix_columns.saturating_add(grapheme.width());
        prefix_end = index + grapheme.len();
    }

    let mut start = prefix_end;
    let mut skipped_columns: usize = 0;
    let mut skipped_chars = 0;
    while skipped_columns < left {
        let Some((index, grapheme)) = graphemes.next() else {
            break;
        };
        skipped_columns = skipped_columns.saturating_add(grapheme.width());
        skipped_chars += grapheme.chars().count();
        start = index + grapheme.len();
    }

    let available = columns.saturating_sub(prefix_width);
    let mut used_columns: usize = 0;
    let mut end = start;
    for (index, grapheme) in graphemes {
        let next_width = used_columns.saturating_add(grapheme.width());
        if next_width > available {
            break;
        }
        used_columns = next_width;
        end = index + grapheme.len();
    }

    if start == prefix_end && end == visible.len() {
        return (Cow::Borrowed(line), skipped_chars);
    }
    (
        project_visible_range(line, prefix_end, start..end),
        skipped_chars,
    )
}

fn project_visible_range(line: &str, prefix_end: usize, content: Range<usize>) -> Cow<'_, str> {
    if !line.as_bytes().contains(&b'\x1b') {
        if prefix_end == 0 {
            return Cow::Borrowed(&line[content]);
        }
        if prefix_end == content.start {
            return Cow::Borrowed(&line[..content.end]);
        }
        return format!("{}{}", &line[..prefix_end], &line[content]).into();
    }

    let mut output = String::new();
    let mut byte_index = 0;
    let mut visible_byte_index = 0;
    while byte_index < line.len() {
        if let Some((sequence_end, _)) = ansi_sequence_end(line.as_bytes(), byte_index) {
            // Hidden text can contain the style or OSC 8 terminator needed by the visible range.
            output.push_str(&line[byte_index..sequence_end]);
            byte_index = sequence_end;
            continue;
        }
        let character = line[byte_index..]
            .chars()
            .next()
            .expect("byte index is within the line");
        if visible_byte_index < prefix_end || content.contains(&visible_byte_index) {
            output.push(character);
        }
        visible_byte_index += character.len_utf8();
        byte_index += character.len_utf8();
    }
    output.into()
}

#[cfg(test)]
mod tests {
    use super::horizontal_scroll_view;

    #[test]
    fn basic_view_no_line_numbers() {
        let line = "0123456789";
        let (view, skipped) = horizontal_scroll_view(line, 5, 2, 0);
        assert_eq!(view, "23456");
        assert_eq!(skipped, 2);
    }

    #[test]
    fn basic_view_with_line_numbers() {
        let prefix = format!(
            "{:5}{}10.{} ",
            "",
            crossterm::style::Attribute::Bold,
            crossterm::style::Attribute::Reset
        );
        let line = format!("{prefix}0123456789");
        for (columns, expected) in [(5, prefix.clone()), (14, format!("{prefix}23456"))] {
            let (view, skipped) = horizontal_scroll_view(&line, columns, 2, 9);
            assert_eq!(view, expected);
            assert_eq!(skipped, 2);
        }
    }

    #[test]
    fn non_ascii_view() {
        let line = "├── 0123456789";
        let (view, skipped) = horizontal_scroll_view(line, 5, 1, 0);
        assert_eq!(view, "── 01");
        assert_eq!(skipped, 1);
    }

    #[test]
    fn overflow_view() {
        let line = "0123";
        assert_eq!(horizontal_scroll_view(line, 10, 0, 0), (line.into(), 0));
        assert_eq!(
            horizontal_scroll_view(line, 10, usize::MAX, 0),
            ("".into(), 4)
        );
    }
}
