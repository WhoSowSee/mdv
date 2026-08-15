use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn apply_table_inline_url_truncation(
        &self,
        table: &mut TableState,
        table_width: usize,
    ) {
        if table.inline_url_segments.is_empty() {
            return;
        }

        let column_width_limits =
            Self::estimate_table_column_width_limits(&table.headers, &table.rows, table_width);
        if column_width_limits.is_empty() {
            return;
        }

        let segments = table.inline_url_segments.clone();

        for segment in segments {
            let (column_index, cell) = match segment.target {
                TableInlineUrlTarget::Header { column_index } => {
                    let cell = table.headers.get_mut(column_index);
                    (column_index, cell)
                }
                TableInlineUrlTarget::Row {
                    row_index,
                    column_index,
                } => {
                    let cell = table
                        .rows
                        .get_mut(row_index)
                        .and_then(|row| row.get_mut(column_index));
                    (column_index, cell)
                }
            };

            let Some(cell) = cell else {
                continue;
            };
            let Some(&column_limit) = column_width_limits.get(column_index) else {
                continue;
            };

            let cell_width = display_width(&strip_ansi(cell));
            let url_part_width = display_width(&segment.url_part);
            if url_part_width == 0 {
                continue;
            }

            let other_content_width = cell_width.saturating_sub(url_part_width);
            let allowed_url_width = column_limit.saturating_sub(other_content_width);

            if allowed_url_width >= url_part_width {
                continue;
            }

            let truncated_url_part =
                self.truncate_table_inline_url_part(&segment.url, allowed_url_width);

            if truncated_url_part == segment.url_part {
                continue;
            }

            if let Some(start_idx) = cell.find(&segment.url_part) {
                let end_idx = start_idx + segment.url_part.len();
                cell.replace_range(start_idx..end_idx, &truncated_url_part);
            } else {
                continue;
            }
        }
    }

    pub(super) fn truncate_table_inline_url_part(&self, url: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }

        if max_width <= 2 {
            return ".".repeat(max_width);
        }

        let inner_width = max_width.saturating_sub(2);
        let truncated = self.truncate_url_with_ellipsis(url, inner_width);
        format!("({})", truncated)
    }

    pub(super) fn estimate_table_column_width_limits(
        headers: &[String],
        rows: &[Vec<String>],
        table_width: usize,
    ) -> Vec<usize> {
        let columns = headers
            .len()
            .max(rows.iter().map(Vec::len).max().unwrap_or(0))
            .max(1);
        let mut widths = vec![1usize; columns];

        for (idx, header) in headers.iter().enumerate() {
            widths[idx] = widths[idx].max(display_width(&strip_ansi(header)).max(1));
        }

        for row in rows {
            for (idx, cell) in row.iter().enumerate().take(columns) {
                widths[idx] = widths[idx].max(display_width(&strip_ansi(cell)).max(1));
            }
        }

        let border_width = columns
            .saturating_mul(TABLE_COLUMN_OVERHEAD)
            .saturating_add(TABLE_BORDER_OVERHEAD);

        let content_budget = table_width.saturating_sub(border_width);
        if content_budget == 0 {
            return vec![1; columns];
        }

        let mut limits = widths;
        let mut total_width: usize = limits.iter().sum();

        while total_width > content_budget {
            let Some((widest_index, _)) = limits
                .iter()
                .enumerate()
                .filter(|(_, width)| **width > 1)
                .max_by_key(|(_, width)| *width)
            else {
                break;
            };

            limits[widest_index] = limits[widest_index].saturating_sub(1);
            total_width = total_width.saturating_sub(1);
        }

        limits
    }

    pub(super) fn compute_table_indent(
        &self,
        terminal_width: usize,
        headers: &[String],
        rows: &[Vec<String>],
    ) -> usize {
        if !self.config.table_smart_indent {
            return 0;
        }

        let base_indent = self.content_indent;
        if base_indent == 0 {
            return 0;
        }

        if matches!(self.config.table_wrap, crate::cli::TableWrapMode::None) {
            return base_indent;
        }

        let min_table_width = Self::minimum_table_width(headers, rows);
        if terminal_width <= min_table_width {
            return 0;
        }

        let max_indent = terminal_width.saturating_sub(min_table_width);
        base_indent.min(max_indent)
    }

    pub(super) fn minimum_table_width(headers: &[String], rows: &[Vec<String>]) -> usize {
        let columns = headers
            .len()
            .max(rows.iter().map(Vec::len).max().unwrap_or(0))
            .max(1);

        let mut tokens_per_column: Vec<Vec<usize>> = vec![Vec::new(); columns];

        for (idx, header) in headers.iter().enumerate() {
            Self::collect_token_widths(header, &mut tokens_per_column[idx]);
        }

        for row in rows {
            for (idx, cell) in row.iter().enumerate().take(columns) {
                Self::collect_token_widths(cell, &mut tokens_per_column[idx]);
            }
        }

        let content_width_sum: usize = tokens_per_column
            .iter()
            .map(|widths| Self::upper_quartile(widths))
            .sum();

        content_width_sum
            .saturating_add(columns.saturating_mul(TABLE_COLUMN_OVERHEAD))
            .saturating_add(TABLE_BORDER_OVERHEAD)
    }

    pub(super) fn collect_token_widths(cell: &str, out: &mut Vec<usize>) {
        let clean = strip_ansi(cell);
        let mut collected_any = false;

        for token in clean.split_whitespace() {
            let width = display_width(token);
            if width > 0 {
                out.push(width);
                collected_any = true;
            }
        }

        if !collected_any {
            let fallback_width = display_width(clean.trim());
            if fallback_width > 0 {
                out.push(fallback_width);
            }
        }
    }

    pub(super) fn upper_quartile(widths: &[usize]) -> usize {
        if widths.is_empty() {
            return 1;
        }

        let mut sorted = widths.to_vec();
        sorted.sort_unstable();
        let index = (sorted.len().saturating_sub(1) * 3) / 4;
        sorted[index].max(1)
    }

    pub(super) fn indent_table_block(table: String, indent: usize) -> String {
        if indent == 0 || table.is_empty() {
            return table;
        }

        let prefix = " ".repeat(indent);
        table
            .lines()
            .map(|line| format!("{}{}", prefix, line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) fn prefix_table_block(table: String, prefix: &str) -> String {
        if prefix.is_empty() || table.is_empty() {
            return table;
        }

        table
            .lines()
            .map(|line| format!("{}{}", prefix, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
