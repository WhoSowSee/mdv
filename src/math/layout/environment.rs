use super::*;
use crate::math::ast::MathEnvironment;

pub(super) fn layout_environment(environment: &MathEnvironment, fonts: &[MathFont]) -> MathBox {
    if matches!(
        environment.name.as_str(),
        "gather" | "gather*" | "gathered" | "substack"
    ) {
        let rows = environment
            .rows
            .iter()
            .map(|row| {
                horizontal(
                    &row.iter()
                        .map(|node| layout_with_fonts(node, fonts))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        return vertical_rows(rows);
    }

    let table = layout_table(
        &environment.rows,
        environment.alignment.as_deref(),
        &environment.name,
        fonts,
    );
    match environment.name.as_str() {
        "pmatrix" => delimit(table, "(", ")"),
        "bmatrix" | "bmatrix*" => delimit(table, "[", "]"),
        "Bmatrix" | "Bmatrix*" => delimit(table, "{", "}"),
        "vmatrix" | "vmatrix*" => delimit(table, "|", "|"),
        "Vmatrix" | "Vmatrix*" => delimit(table, "‖", "‖"),
        "cases" => delimit(table, "{", ""),
        _ => table,
    }
}

fn layout_table(
    rows: &[Vec<MathNode>],
    alignment: Option<&str>,
    environment: &str,
    fonts: &[MathFont],
) -> MathBox {
    let cells = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|node| layout_with_fonts(node, fonts))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let columns = cells.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0usize; columns];
    for row in &cells {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.width);
        }
    }

    let formats = column_formats(alignment, columns, environment);
    let leading_separator = alignment.is_some_and(|value| value.trim_start().starts_with('|'));
    let trailing_separator = alignment.is_some_and(|value| value.trim_end().ends_with('|'));
    let separators_width = formats
        .iter()
        .take(columns.saturating_sub(1))
        .map(|format| if format.separator_after { 3 } else { 1 })
        .sum::<usize>()
        + usize::from(leading_separator) * 2
        + usize::from(trailing_separator) * 2;
    let width = widths.iter().sum::<usize>() + separators_width;
    let mut lines = Vec::new();
    let mut preserved_trailing = Vec::new();
    for row in cells {
        let baseline = row.iter().map(|cell| cell.baseline).max().unwrap_or(0);
        let below = row
            .iter()
            .map(|cell| cell.lines.len().saturating_sub(cell.baseline + 1))
            .max()
            .unwrap_or(0);
        for visual_row in 0..baseline + below + 1 {
            let mut line = String::new();
            let mut trailing = None;
            if leading_separator {
                line.push_str("│ ");
            }
            for (column, column_width) in widths.iter().copied().enumerate() {
                let cell_line = row.get(column).and_then(|cell| {
                    let index = visual_row
                        .checked_sub(baseline.saturating_sub(cell.baseline))
                        .filter(|index| *index < cell.lines.len())?;
                    Some((&cell.lines[index], &cell.preserved_trailing[index]))
                });
                let content = cell_line.map_or("", |(line, _)| line.as_str());
                if let Some((_, Some(source_trailing))) = cell_line {
                    trailing = Some(source_trailing.clone());
                } else if content.chars().any(|character| !character.is_whitespace()) {
                    trailing = None;
                }
                let format = formats[column];
                line.push_str(&align_cell(content, column_width, format.alignment));
                if column + 1 < columns {
                    if format.separator_after {
                        line.push_str(" │ ");
                    } else {
                        line.push(' ');
                    }
                }
            }
            if trailing_separator {
                line.push_str(" │");
                trailing = None;
            }
            lines.push(line);
            preserved_trailing.push(trailing);
        }
    }
    MathBox {
        baseline: lines.len() / 2,
        lines,
        preserved_trailing,
        width,
    }
}

#[derive(Clone, Copy)]
struct ColumnFormat {
    alignment: char,
    separator_after: bool,
}

fn column_formats(
    specification: Option<&str>,
    columns: usize,
    environment: &str,
) -> Vec<ColumnFormat> {
    let mut formats: Vec<ColumnFormat> = Vec::new();
    let mut pending_separator = false;
    for character in specification.unwrap_or("").chars() {
        match character {
            'l' | 'c' | 'r' => {
                if let Some(previous) = formats.last_mut() {
                    previous.separator_after = pending_separator;
                }
                formats.push(ColumnFormat {
                    alignment: character,
                    separator_after: false,
                });
                pending_separator = false;
            }
            '|' if !formats.is_empty() => pending_separator = true,
            _ => {}
        }
    }
    for column in formats.len()..columns {
        formats.push(ColumnFormat {
            alignment: default_column_alignment(environment, column),
            separator_after: false,
        });
    }
    formats
}

fn default_column_alignment(environment: &str, column: usize) -> char {
    match environment {
        "align" | "align*" | "aligned" | "split" => {
            if column.is_multiple_of(2) {
                'r'
            } else {
                'l'
            }
        }
        "eqnarray" => match column % 3 {
            0 => 'r',
            1 => 'c',
            _ => 'l',
        },
        "matrix" | "pmatrix" | "bmatrix" | "bmatrix*" | "Bmatrix" | "Bmatrix*" | "vmatrix"
        | "vmatrix*" | "Vmatrix" | "Vmatrix*" => 'c',
        _ => 'l',
    }
}

fn align_cell(text: &str, width: usize, alignment: char) -> String {
    let remaining = width.saturating_sub(display_width(text));
    match alignment {
        'r' => format!("{}{}", " ".repeat(remaining), text),
        'c' => center(text, width),
        _ => format!("{}{}", text, " ".repeat(remaining)),
    }
}
