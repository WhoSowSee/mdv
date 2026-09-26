use super::*;
use crate::terminal::AnsiStyle;
use regex::regex;
use unicode_segmentation::UnicodeSegmentation;

impl TableRenderer {
    pub(super) fn colorize_borders(&self, rendered: String) -> String {
        if self.output_style.is_disabled() {
            return rendered;
        }
        if !self.theme.table_border_overridden {
            return rendered;
        }

        let lines = rendered.split('\n').collect::<Vec<_>>();
        let columns = self.border_columns(&lines);
        let style = create_style(&self.theme, ThemeElement::TableBorder);
        let mut output = String::with_capacity(rendered.len());

        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            if is_horizontal_border(line) {
                output.push_str(&style.apply(line, self.output_style));
            } else {
                append_colored_verticals(&mut output, line, &columns, &style, self.output_style);
            }
        }
        output
    }

    fn border_columns(&self, lines: &[&str]) -> Vec<usize> {
        let reference = if self.table_borders {
            lines.first().copied()
        } else {
            lines
                .iter()
                .copied()
                .find(|line| is_horizontal_border(line))
        };
        let Some(reference) = reference else {
            return Vec::new();
        };
        strip_ansi(reference)
            .chars()
            .enumerate()
            .filter_map(|(column, character)| {
                let is_boundary = if self.table_borders {
                    matches!(character, '╭' | '┬' | '╮')
                } else {
                    character == '┼'
                };
                is_boundary.then_some(column)
            })
            .collect()
    }
}

fn is_horizontal_border(line: &str) -> bool {
    !line.contains('\x1b')
        && line
            .chars()
            .any(|character| matches!(character, '─' | '═' | '╌'))
        && line.chars().all(|character| {
            matches!(
                character,
                '─' | '═'
                    | '╌'
                    | '┼'
                    | '┬'
                    | '┴'
                    | '╭'
                    | '╮'
                    | '╰'
                    | '╯'
                    | '╞'
                    | '╪'
                    | '╡'
                    | '├'
                    | '┤'
            )
        })
}

fn append_colored_verticals(
    output: &mut String,
    line: &str,
    columns: &[usize],
    style: &AnsiStyle,
    output_style: OutputStyle,
) {
    let mut column = 0;
    let mut start = 0;
    for escape in regex!(r"\x1b\[[0-9;]*m|\x1b\]8;;[^\x1b]*\x1b\\").find_iter(line) {
        append_segment(
            output,
            &line[start..escape.start()],
            columns,
            &mut column,
            style,
            output_style,
        );
        output.push_str(escape.as_str());
        start = escape.end();
    }
    append_segment(
        output,
        &line[start..],
        columns,
        &mut column,
        style,
        output_style,
    );
}

fn append_segment(
    output: &mut String,
    segment: &str,
    columns: &[usize],
    column: &mut usize,
    style: &AnsiStyle,
    output_style: OutputStyle,
) {
    for grapheme in segment.graphemes(true) {
        if columns.contains(column) && matches!(grapheme, "│" | "┆") {
            output.push_str(&style.apply(grapheme, output_style));
        } else {
            output.push_str(grapheme);
        }
        *column += display_width(grapheme);
    }
}
