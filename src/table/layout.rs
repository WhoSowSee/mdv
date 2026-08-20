use super::*;

impl TableRenderer {
    pub fn new(
        theme: &Theme,
        no_colors: bool,
        terminal_width: usize,
        table_wrap: TableWrapMode,
    ) -> Self {
        Self {
            theme: theme.clone(),
            no_colors,
            terminal_width,
            table_wrap,
            pretty_table: false,
        }
    }

    pub fn with_pretty_table(mut self, pretty_table: bool) -> Self {
        self.pretty_table = pretty_table;
        self
    }

    pub(super) fn configure_table(&self, table: &mut Table) {
        if self.pretty_table {
            table.load_style(UTF8_FULL.with_rounded_corners());
        } else {
            table.load_style(COMPACT_TABLE_STYLE);
        }
        table.set_content_arrangement(ContentArrangement::Dynamic);
    }

    /// Create a cell with proper ANSI handling for width calculation
    pub(super) fn create_cell(&self, content: &str, reference_layout: &ReferenceLayout) -> Cell {
        let layout_content = if content.contains(TABLE_REFERENCE_WRAP_MARKER) {
            let replacement = if matches!(reference_layout, ReferenceLayout::ForcedBreak) {
                "\n"
            } else {
                ""
            };
            Cow::Owned(content.replace(TABLE_REFERENCE_WRAP_MARKER, replacement))
        } else {
            Cow::Borrowed(content)
        };

        if self.no_colors {
            Cell::new(strip_ansi(layout_content.as_ref()))
        } else {
            Cell::new(layout_content.as_ref())
        }
    }

    pub(super) fn reference_layout(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
    ) -> ReferenceLayout {
        let mut widths = vec![0usize; headers.len()];
        for row in std::iter::once(headers).chain(rows.iter().map(Vec::as_slice)) {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(reference_token_width(cell));
            }
        }
        if widths.iter().all(|width| *width == 0)
            || self.estimate_table_width(headers, rows) <= self.terminal_width
        {
            return ReferenceLayout::Natural;
        }

        let border_width = if self.pretty_table {
            headers.len().saturating_add(1)
        } else {
            headers.len().saturating_sub(1)
        };
        let required_width = widths
            .iter()
            .map(|width| (*width).max(1).saturating_add(2))
            .sum::<usize>()
            .saturating_add(border_width);

        if required_width <= self.terminal_width {
            ReferenceLayout::Constrained(widths)
        } else {
            ReferenceLayout::ForcedBreak
        }
    }

    pub(super) fn apply_reference_width_constraints(
        table: &mut Table,
        reference_layout: &ReferenceLayout,
    ) {
        let ReferenceLayout::Constrained(widths) = reference_layout else {
            return;
        };

        for (column_index, content_width) in widths.iter().copied().enumerate() {
            if content_width > 0
                && let Some(column) = table.column_mut(column_index)
            {
                let width_with_padding =
                    content_width.saturating_add(2).min(u16::MAX as usize) as u16;
                column.set_constraint(ColumnConstraint::Absolute(Width::Fixed(width_with_padding)));
            }
        }
        table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
    }

    fn maximum_column_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
        let mut max_widths = headers
            .iter()
            .map(|header| display_width(&strip_ansi(header)))
            .collect::<Vec<_>>();

        for row in rows {
            for (cell, max_width) in row.iter().zip(&mut max_widths) {
                *max_width = (*max_width).max(display_width(&strip_ansi(cell)));
            }
        }

        max_widths
    }

    /// Calculate estimated table width
    pub(super) fn estimate_table_width(&self, headers: &[String], rows: &[Vec<String>]) -> usize {
        let max_widths = Self::maximum_column_widths(headers, rows);

        // Add borders and padding: 3 chars per column (│ x │) + 1 for final border
        max_widths.iter().sum::<usize>() + (headers.len() * 3) + 1
    }

    /// Calculate column widths for each column
    fn calculate_column_widths(&self, headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
        Self::maximum_column_widths(headers, rows)
            .into_iter()
            .map(|width| width.max(3))
            .collect()
    }

    /// Split table into column blocks that fit terminal width
    pub(super) fn split_table_into_blocks(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Vec<TableBlock> {
        let column_widths = self.calculate_column_widths(headers, rows);
        let mut blocks = Vec::new();
        let mut current_block_start = 0;

        // Reserve space for table borders and separators
        let border_overhead = 4; // Minimum space for borders

        while current_block_start < headers.len() {
            let mut current_width = border_overhead;
            let mut current_block_end = current_block_start;

            // Always include at least one column
            if current_block_start < headers.len() {
                current_width += column_widths[current_block_start] + 3; // column + borders
                current_block_end = current_block_start + 1;
            }

            for (i, width) in column_widths
                .iter()
                .enumerate()
                .skip(current_block_start + 1)
            {
                let additional_width = *width + 3; // column width + borders

                if current_width + additional_width <= self.terminal_width {
                    current_width += additional_width;
                    current_block_end = i + 1;
                } else {
                    break;
                }
            }

            let block_headers: Vec<String> =
                headers[current_block_start..current_block_end].to_vec();
            let block_rows: Vec<Vec<String>> = rows
                .iter()
                .map(|row| {
                    if row.len() > current_block_start {
                        let end_idx = current_block_end.min(row.len());
                        row[current_block_start..end_idx].to_vec()
                    } else {
                        // If row doesn't have enough columns, fill with empty strings
                        vec!["".to_string(); block_headers.len()]
                    }
                })
                .collect();

            let block_alignments: Vec<Alignment> = if alignments.len() > current_block_start {
                let end_idx = current_block_end.min(alignments.len());
                alignments[current_block_start..end_idx].to_vec()
            } else {
                vec![Alignment::Left; block_headers.len()]
            };

            blocks.push((block_headers, block_rows, block_alignments));
            current_block_start = current_block_end;
        }

        blocks
    }
}

fn reference_token_width(cell: &str) -> usize {
    if !cell.contains(TABLE_REFERENCE_WRAP_MARKER) {
        return 0;
    }

    let clean = strip_ansi(cell);
    clean
        .match_indices(TABLE_REFERENCE_WRAP_MARKER)
        .map(|(marker_index, marker)| {
            let token_start = clean[..marker_index]
                .char_indices()
                .rev()
                .find(|(_, ch)| ch.is_whitespace())
                .map(|(index, ch)| index + ch.len_utf8())
                .unwrap_or(0);
            let reference_start = marker_index + marker.len();
            let reference_end = clean[reference_start..]
                .find(']')
                .map(|offset| reference_start + offset + 1)
                .unwrap_or(reference_start);

            display_width(&clean[token_start..marker_index])
                + display_width(&clean[reference_start..reference_end])
        })
        .max()
        .unwrap_or(0)
}
