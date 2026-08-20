use unicode_segmentation::UnicodeSegmentation;

use super::*;

pub(super) fn arranged_column_content_widths(
    rendered: &str,
    column_count: usize,
    pretty_table: bool,
) -> Option<Vec<usize>> {
    let clean = strip_ansi(rendered);
    let total_widths = if pretty_table {
        let border = clean
            .lines()
            .find(|line| line.starts_with('╭') && line.ends_with('╮'))?;
        border
            .strip_prefix('╭')?
            .strip_suffix('╮')?
            .split('┬')
            .map(display_width)
            .collect::<Vec<_>>()
    } else {
        let separator = clean.lines().find(|line| {
            line.contains('─') && line.chars().all(|character| matches!(character, '─' | '┼'))
        })?;
        separator.split('┼').map(display_width).collect::<Vec<_>>()
    };

    (total_widths.len() == column_count).then(|| {
        total_widths
            .into_iter()
            .map(|width| width.saturating_sub(2).max(1))
            .collect()
    })
}

pub(super) fn remove_wrapped_boundary_spaces(
    headers: &[String],
    rows: &[Vec<String>],
    widths: &[usize],
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut changed = false;
    let mut normalize = |cell: &String, column_index: usize| {
        let Some(width) = widths.get(column_index) else {
            return cell.clone();
        };
        if let Some(normalized) = remove_cell_boundary_spaces(cell, *width) {
            changed = true;
            normalized
        } else {
            cell.clone()
        }
    };

    let normalized_headers = headers
        .iter()
        .enumerate()
        .map(|(index, cell)| normalize(cell, index))
        .collect();
    let normalized_rows = rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(index, cell)| normalize(cell, index))
                .collect()
        })
        .collect();

    changed.then_some((normalized_headers, normalized_rows))
}

fn remove_cell_boundary_spaces(content: &str, width: usize) -> Option<String> {
    if !content.contains(' ') {
        return None;
    }

    let clean = strip_ansi(content);
    let mut current_width = 0usize;
    let mut automatic_continuation = false;
    let mut space_ordinal = 0usize;
    let mut removed_spaces = Vec::new();

    for grapheme in clean.graphemes(true) {
        if grapheme.contains('\n') {
            current_width = 0;
            automatic_continuation = false;
            continue;
        }

        let grapheme_width = display_width(grapheme);
        if current_width > 0 && current_width + grapheme_width > width {
            current_width = 0;
            automatic_continuation = true;
        }

        if grapheme == " " {
            if current_width == 0 && automatic_continuation {
                removed_spaces.push(space_ordinal);
                space_ordinal += 1;
                continue;
            }
            space_ordinal += 1;
        }
        current_width += grapheme_width;
    }

    if removed_spaces.is_empty() {
        return None;
    }

    let mut normalized = String::with_capacity(content.len() - removed_spaces.len());
    let mut ordinal = 0usize;
    let mut removed = removed_spaces.into_iter().peekable();
    for character in content.chars() {
        if character == ' ' {
            if removed.next_if_eq(&ordinal).is_none() {
                normalized.push(character);
            }
            ordinal += 1;
        } else {
            normalized.push(character);
        }
    }
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_spaces_at_automatic_grapheme_boundaries() {
        assert_eq!(
            remove_cell_boundary_spaces("1234567 abcdefgh", 7).as_deref(),
            Some("1234567abcdefgh")
        );
        assert_eq!(
            remove_cell_boundary_spaces("12345 👨‍👩‍👧‍👦x", 5).as_deref(),
            Some("12345👨‍👩‍👧‍👦x")
        );
        assert_eq!(
            remove_cell_boundary_spaces("\x1b[31m1234567 abcdefgh\x1b[0m", 7).as_deref(),
            Some("\x1b[31m1234567abcdefgh\x1b[0m")
        );
    }

    #[test]
    fn extracts_pretty_and_compact_content_widths() {
        assert_eq!(
            arranged_column_content_widths("╭─────────┬──────╮\n", 2, true),
            Some(vec![7, 4])
        );
        assert_eq!(
            arranged_column_content_widths("─────────┼──────\n", 2, false),
            Some(vec![7, 4])
        );
    }
}
