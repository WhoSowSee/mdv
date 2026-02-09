use super::{
    EventRenderer, LinkStyle, LinkTruncationStyle, Result, TableInlineUrlTarget, TableRenderer,
    TableState, ThemeElement, create_style,
};
use crate::utils::{display_width, strip_ansi};
use pulldown_cmark::Alignment;

const TABLE_COLUMN_OVERHEAD: usize = 3;
const TABLE_BORDER_OVERHEAD: usize = 1;
const TABLE_REFERENCE_WRAP_DELIMITER: char = '\u{200B}';

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_table_end(&mut self) -> Result<()> {
        let mut table_indent = 0usize;
        if let Some(table) = self.table_state.take() {
            table_indent = self.render_table(table)?;
        }

        // Add accumulated link references for InlineTable mode at the end of the table
        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references_for_table(table_indent);
        }

        Ok(())
    }

    pub(super) fn render_table(&mut self, mut table: TableState) -> Result<usize> {
        let headers_empty = table
            .headers
            .iter()
            .all(|header| strip_ansi(header).trim().is_empty());
        let rows_empty = table
            .rows
            .iter()
            .all(|row| row.iter().all(|cell| strip_ansi(cell).trim().is_empty()));

        if table.headers.is_empty() && !self.config.show_empty_elements {
            return Ok(0);
        }

        if headers_empty && rows_empty && !self.config.show_empty_elements {
            return Ok(0);
        }

        if self.config.show_empty_elements {
            if table.headers.is_empty() {
                table.headers.push(" ".to_string());
            } else if headers_empty {
                for header in table.headers.iter_mut() {
                    if strip_ansi(header).trim().is_empty() {
                        *header = " ".to_string();
                    }
                }
            }

            if table.alignments.len() < table.headers.len() {
                table.alignments.extend(
                    std::iter::repeat(Alignment::Left)
                        .take(table.headers.len().saturating_sub(table.alignments.len())),
                );
            }

            if rows_empty {
                if table.rows.is_empty() {
                    table
                        .rows
                        .push(vec![" ".to_string(); table.headers.len().max(1)]);
                } else {
                    for row in table.rows.iter_mut() {
                        if row.len() < table.headers.len() {
                            row.extend(
                                std::iter::repeat(String::new())
                                    .take(table.headers.len().saturating_sub(row.len())),
                            );
                        }
                        for cell in row.iter_mut() {
                            if strip_ansi(cell).trim().is_empty() {
                                *cell = " ".to_string();
                            }
                        }
                    }
                }
            }
        } else if table.headers.is_empty() {
            return Ok(0);
        }

        let terminal_width = self.config.get_terminal_width();
        let line_prefix = if self.blockquote_level > 0 {
            self.current_line_prefix()
        } else {
            String::new()
        };
        let prefix_width = display_width(&strip_ansi(&line_prefix));
        let table_indent = if self.blockquote_level > 0 {
            0
        } else {
            self.compute_table_indent(terminal_width, &table.headers, &table.rows)
        };
        let available_width = terminal_width
            .saturating_sub(prefix_width)
            .saturating_sub(table_indent)
            .max(1);

        if matches!(self.config.link_style, LinkStyle::Inline)
            && matches!(self.config.link_truncation, LinkTruncationStyle::TableCut)
        {
            self.apply_table_inline_url_truncation(&mut table, available_width);
        }

        let table_renderer = TableRenderer::new(
            self.theme,
            self.config.no_colors,
            available_width,
            self.config.table_wrap,
        );

        let mut rendered_table =
            table_renderer.render_table(&table.headers, &table.rows, &table.alignments)?;
        rendered_table = rendered_table.replace(TABLE_REFERENCE_WRAP_DELIMITER, "");

        if !table.inline_references.is_empty() {
            rendered_table = crate::table::apply_inline_reference_styles(
                rendered_table,
                &table.inline_references,
                self.config.no_colors,
            );
        }
        rendered_table = Self::indent_table_block(rendered_table, table_indent);
        rendered_table = Self::prefix_table_block(rendered_table, &line_prefix);

        self.ensure_contextual_blank_line();

        self.output.push_str(&rendered_table);
        self.output.push('\n');
        self.commit_pending_heading_placeholder_if_content();

        Ok(table_indent)
    }

    fn apply_table_inline_url_truncation(&self, table: &mut TableState, table_width: usize) {
        if table.inline_url_segments.is_empty() {
            return;
        }

        let column_width_limits =
            Self::estimate_table_column_width_limits(&table.headers, &table.rows, table_width);
        if column_width_limits.is_empty() {
            return;
        }

        let link_style = create_style(self.theme, ThemeElement::Link);
        let mut reference_cursor = 0usize;
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

            if let Some((idx, _)) = table
                .inline_references
                .iter()
                .enumerate()
                .skip(reference_cursor)
                .find(|(_, (plain, _))| plain == &segment.url_part)
            {
                let styled = link_style.apply(&truncated_url_part, self.config.no_colors);
                table.inline_references[idx] = (truncated_url_part.clone(), styled);
                reference_cursor = idx.saturating_add(1);
            }
        }
    }

    fn truncate_table_inline_url_part(&self, url: &str, max_width: usize) -> String {
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

    fn estimate_table_column_width_limits(
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

    fn compute_table_indent(
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

    fn minimum_table_width(headers: &[String], rows: &[Vec<String>]) -> usize {
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

    fn collect_token_widths(cell: &str, out: &mut Vec<usize>) {
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

    fn upper_quartile(widths: &[usize]) -> usize {
        if widths.is_empty() {
            return 1;
        }

        let mut sorted = widths.to_vec();
        sorted.sort_unstable();
        let index = (sorted.len().saturating_sub(1) * 3) / 4;
        sorted[index].max(1)
    }

    fn indent_table_block(table: String, indent: usize) -> String {
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

    fn prefix_table_block(table: String, prefix: &str) -> String {
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
