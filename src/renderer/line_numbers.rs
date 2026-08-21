use crate::cli::{LineNumberOptions, LineNumberTarget};
use crate::markdown::{SourceLineMarker, source_line_from_event};
use crate::terminal::AnsiStyle;
use pulldown_cmark::Event;

// unicode-width counts C0 controls in strings, so metadata uses zero-width default-ignorable code points.
const INTERNAL_MARKER_START: char = '\u{2063}';
const INTERNAL_MARKER_END: char = '\u{2064}';
const INTERNAL_MARKER_ZERO: char = '\u{200c}';
const INTERNAL_MARKER_ONE: char = '\u{200d}';
const PLAIN_GUTTER_SUFFIX_WIDTH: usize = 1;
const SEPARATOR_GUTTER_SUFFIX_WIDTH: usize = 3;

pub(super) fn max_source_line(events: &[Event<'_>]) -> Option<usize> {
    events
        .iter()
        .filter_map(source_line_from_event)
        .map(|marker| match marker {
            SourceLineMarker::Content(line) | SourceLineMarker::Blank(line) => line,
        })
        .max()
}

pub(super) fn gutter_width(max_line: usize, options: LineNumberOptions) -> usize {
    gutter_width_for_number_width(max_line.to_string().len(), options)
}

pub(super) fn gutter_width_for_number_width(
    number_width: usize,
    options: LineNumberOptions,
) -> usize {
    let suffix_width = if options.separator {
        SEPARATOR_GUTTER_SUFFIX_WIDTH
    } else {
        PLAIN_GUTTER_SUFFIX_WIDTH
    };
    number_width + suffix_width
}

pub(super) fn rendered_line_count(output: &str) -> usize {
    if output.is_empty() {
        0
    } else {
        output.split_inclusive('\n').count()
    }
}

pub(super) fn encode_internal_marker(line: usize) -> String {
    let highest_bit = usize::BITS - line.max(1).leading_zeros();
    let mut marker = String::with_capacity(highest_bit as usize + 2);
    marker.push(INTERNAL_MARKER_START);
    for shift in (0..highest_bit).rev() {
        let bit = (line >> shift) & 1;
        marker.push(if bit == 0 {
            INTERNAL_MARKER_ZERO
        } else {
            INTERNAL_MARKER_ONE
        });
    }
    marker.push(INTERNAL_MARKER_END);
    marker
}

pub(super) fn apply_line_numbers(
    output: &str,
    max_line: usize,
    number_style: &AnsiStyle,
    separator_style: &AnsiStyle,
    options: LineNumberOptions,
    no_colors: bool,
) -> String {
    if output.is_empty() {
        return String::new();
    }

    let number_width = max_line.to_string().len();
    let mut numbered = String::with_capacity(
        output.len() + rendered_line_count(output) * gutter_width(max_line, options),
    );
    for (rendered_index, rendered_line) in output.split_inclusive('\n').enumerate() {
        let (content, newline) = rendered_line
            .strip_suffix('\n')
            .map_or((rendered_line, ""), |content| (content, "\n"));
        let (content, source_line) = strip_internal_markers(content);
        let line_number = match options.target {
            LineNumberTarget::Rendered => Some(rendered_index + 1),
            LineNumberTarget::Source => source_line,
        };
        numbered.push_str(&format_gutter(
            line_number,
            number_width,
            number_style,
            separator_style,
            options,
            no_colors,
        ));
        numbered.push_str(&content);
        numbered.push_str(newline);
    }
    numbered
}

pub(super) fn format_gutter(
    line_number: Option<usize>,
    number_width: usize,
    number_style: &AnsiStyle,
    separator_style: &AnsiStyle,
    options: LineNumberOptions,
    no_colors: bool,
) -> String {
    let number = match line_number {
        Some(line) => format!("{line:>number_width$}"),
        None => format!("{:number_width$}", ""),
    };
    let mut gutter = number_style.apply(&number, no_colors);
    if options.separator {
        gutter.push_str(&separator_style.apply(" │ ", no_colors));
    } else {
        gutter.push(' ');
    }
    gutter
}

pub(super) fn strip_internal_markers(line: &str) -> (String, Option<usize>) {
    let mut cleaned = String::with_capacity(line.len());
    let mut source_line = None;
    let mut cursor = 0usize;

    while cursor < line.len() {
        let Some(relative_start) = line[cursor..].find(INTERNAL_MARKER_START) else {
            cleaned.push_str(&line[cursor..]);
            break;
        };
        let marker_start = cursor + relative_start;
        cleaned.push_str(&line[cursor..marker_start]);

        let payload_start = marker_start + INTERNAL_MARKER_START.len_utf8();
        let Some(relative_end) = line[payload_start..].find(INTERNAL_MARKER_END) else {
            cleaned.push_str(&line[marker_start..]);
            break;
        };
        let marker_end = payload_start + relative_end;
        let mut value = 0usize;
        let mut has_payload = false;
        let mut valid = true;
        for ch in line[payload_start..marker_end].chars() {
            match ch {
                INTERNAL_MARKER_ZERO => {
                    value <<= 1;
                    has_payload = true;
                }
                INTERNAL_MARKER_ONE => {
                    value = (value << 1) | 1;
                    has_payload = true;
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        }

        if valid && has_payload {
            source_line.get_or_insert(value);
            cursor = marker_end + INTERNAL_MARKER_END.len_utf8();
        } else {
            cleaned.push(INTERNAL_MARKER_START);
            cursor = payload_start;
        }
    }

    (cleaned, source_line)
}

#[cfg(test)]
mod tests {
    use super::{apply_line_numbers, encode_internal_marker, strip_internal_markers};
    use crate::cli::{LineNumberOptions, LineNumberTarget};
    use crate::terminal::AnsiStyle;
    use crate::utils::display_width;
    use crossterm::style::Color;

    #[test]
    fn internal_marker_round_trips_without_visible_content() {
        let marker = encode_internal_marker(42);
        let (cleaned, source_line) = strip_internal_markers(&format!("prefix{marker}text"));

        assert_eq!(cleaned, "prefixtext");
        assert_eq!(source_line, Some(42));
    }

    #[test]
    fn internal_marker_occupies_no_display_columns() {
        assert_eq!(display_width(&encode_internal_marker(42)), 0);
    }

    #[test]
    fn number_and_separator_use_independent_styles() {
        let output = format!("{}text\n", encode_internal_marker(1));
        let number_style = AnsiStyle::new().fg(Color::Rgb { r: 1, g: 2, b: 3 });
        let separator_style = AnsiStyle::new().fg(Color::Rgb { r: 4, g: 5, b: 6 });

        assert_eq!(
            apply_line_numbers(
                &output,
                1,
                &number_style,
                &separator_style,
                LineNumberOptions {
                    target: LineNumberTarget::Source,
                    separator: true,
                },
                false,
            ),
            "\x1b[38;2;1;2;3m1\x1b[0m\x1b[38;2;4;5;6m │ \x1b[0mtext\n"
        );
    }
}
