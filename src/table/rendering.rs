use super::*;

impl TableRenderer {
    /// Render table with column wrapping
    fn render_wrapped_table(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Result<String> {
        let blocks = self.split_table_into_blocks(headers, rows, alignments);
        let mut result = String::new();

        for (block_idx, (block_headers, block_rows, block_alignments)) in blocks.iter().enumerate()
        {
            // Add block separator and info for all blocks except the first
            if block_idx > 0 {
                result.push('\n');

                let separator_width = self.terminal_width.min(80);
                let inner_separator = "═".repeat(separator_width.saturating_sub(3));

                let full_separator_text = format!("{}", inner_separator);

                let separator = if self.no_colors {
                    full_separator_text
                } else {
                    let border_style = create_style(&self.theme, ThemeElement::TableBorder);
                    border_style.apply(&full_separator_text, self.no_colors)
                };
                result.push_str(&separator);
                result.push('\n');
            }

            // Add block number indicator for ALL blocks (including first)
            let block_style = create_style(&self.theme, ThemeElement::Quote);
            let block_info = block_style.apply(
                &format!("Block {} of {}", block_idx + 1, blocks.len()),
                self.no_colors,
            );
            result.push_str(&block_info);
            result.push('\n');

            // Render this block as a regular table
            let block_table =
                self.render_single_table_block(block_headers, block_rows, block_alignments)?;
            result.push_str(&block_table);
        }

        // Informational note about column wrapping removed for cleaner output

        Ok(result)
    }

    /// Render a single table block without width limit (for --table-no-wrap)
    fn render_single_table_block_no_width_limit(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Result<String> {
        let mut table = Table::new();
        let reference_layout = ReferenceLayout::Natural;

        self.configure_table(&mut table);

        if !self.no_colors {
            table.enforce_styling();
        }

        // Don't set width limit - let table overflow

        // Add headers with styling
        let header_cells: Vec<Cell> = headers
            .iter()
            .enumerate()
            .map(|(i, header)| {
                let mut cell = self.create_cell(header, &reference_layout);

                if !self.no_colors {
                    if let Some(color) = theme_color_to_comfy(&self.theme.table_header) {
                        cell = cell.fg(color);
                    }

                    cell = cell.add_attribute(Attribute::Bold);
                }

                if i < alignments.len() {
                    let alignment = match alignments[i] {
                        Alignment::Left => CellAlignment::Left,
                        Alignment::Center => CellAlignment::Center,
                        Alignment::Right => CellAlignment::Right,
                        Alignment::None => CellAlignment::Left,
                    };
                    cell = cell.set_alignment(alignment);
                } else {
                    cell = cell.set_alignment(CellAlignment::Center);
                }

                cell
            })
            .collect();

        table.set_header(header_cells);

        // Add data rows
        for row in rows {
            let row_cells: Vec<Cell> = row
                .iter()
                .enumerate()
                .map(|(i, cell_content)| {
                    let mut cell = self.create_cell(cell_content, &reference_layout);

                    if i < alignments.len() {
                        let alignment = match alignments[i] {
                            Alignment::Left => CellAlignment::Left,
                            Alignment::Center => CellAlignment::Center,
                            Alignment::Right => CellAlignment::Right,
                            Alignment::None => CellAlignment::Left,
                        };
                        cell = cell.set_alignment(alignment);
                    }

                    cell
                })
                .collect();

            table.add_row(row_cells);
        }

        let rendered = table.to_string();
        Ok(Self::collapse_header_only_separator(rendered, rows))
    }

    /// Render a single table block
    fn render_single_table_block(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Result<String> {
        let mut table = Table::new();
        let reference_layout = self.reference_layout(headers, rows);

        self.configure_table(&mut table);

        if !self.no_colors {
            table.enforce_styling();
        }

        // Set table width to fit terminal
        if self.terminal_width > 10 {
            table.set_width(self.terminal_width as u16);
        }

        // Add headers with styling
        let header_cells: Vec<Cell> = headers
            .iter()
            .enumerate()
            .map(|(i, header)| {
                let mut cell = self.create_cell(header, &reference_layout);

                if !self.no_colors {
                    if let Some(color) = theme_color_to_comfy(&self.theme.table_header) {
                        cell = cell.fg(color);
                    }

                    cell = cell.add_attribute(Attribute::Bold);
                }

                if i < alignments.len() {
                    let alignment = match alignments[i] {
                        Alignment::Left => CellAlignment::Left,
                        Alignment::Center => CellAlignment::Center,
                        Alignment::Right => CellAlignment::Right,
                        Alignment::None => CellAlignment::Left,
                    };
                    cell = cell.set_alignment(alignment);
                } else {
                    cell = cell.set_alignment(CellAlignment::Center);
                }

                cell
            })
            .collect();

        table.set_header(header_cells);

        // Add data rows
        for row in rows {
            let row_cells: Vec<Cell> = row
                .iter()
                .enumerate()
                .map(|(i, cell_content)| {
                    let mut cell = self.create_cell(cell_content, &reference_layout);

                    if i < alignments.len() {
                        let alignment = match alignments[i] {
                            Alignment::Left => CellAlignment::Left,
                            Alignment::Center => CellAlignment::Center,
                            Alignment::Right => CellAlignment::Right,
                            Alignment::None => CellAlignment::Left,
                        };
                        cell = cell.set_alignment(alignment);
                    }

                    cell
                })
                .collect();

            table.add_row(row_cells);
        }

        Self::apply_reference_width_constraints(&mut table, &reference_layout);

        let rendered = table.to_string();
        Ok(Self::collapse_header_only_separator(rendered, rows))
    }

    fn collapse_header_only_separator(rendered: String, rows: &[Vec<String>]) -> String {
        if !rows.is_empty() {
            return rendered;
        }

        let mut lines: Vec<&str> = rendered.lines().collect();
        if lines.len() < 4 {
            return rendered;
        }

        lines.remove(lines.len() - 2);
        lines.join("\n")
    }

    pub fn render_table(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Result<String> {
        if headers.is_empty() {
            return Ok(String::new());
        }

        match self.table_wrap {
            TableWrapMode::None => {
                // No wrapping: tables overflow horizontally (like --no-wrap for text)
                self.render_single_table_block_no_width_limit(headers, rows, alignments)
            }
            TableWrapMode::Wrap => {
                // Column wrapping: split table into blocks when too wide
                // Estimate table width
                let estimated_width = self.estimate_table_width(headers, rows);

                // If table fits in terminal width, render normally
                if estimated_width <= self.terminal_width {
                    self.render_single_table_block(headers, rows, alignments)
                } else {
                    // If table is too wide, use column wrapping (horizontal split)
                    self.render_wrapped_table(headers, rows, alignments)
                }
            }
            TableWrapMode::Fit => {
                // Fit behavior: wrap text within table cells, fit to terminal width
                self.render_single_table_block(headers, rows, alignments)
            }
        }
    }
}

pub(super) fn theme_color_to_comfy(color: &ThemeColor) -> Option<Color> {
    match color {
        ThemeColor::Black => Some(Color::Black),
        ThemeColor::DarkRed => Some(Color::DarkRed),
        ThemeColor::DarkGreen => Some(Color::DarkGreen),
        ThemeColor::DarkYellow => Some(Color::DarkYellow),
        ThemeColor::DarkBlue => Some(Color::DarkBlue),
        ThemeColor::DarkMagenta => Some(Color::DarkMagenta),
        ThemeColor::DarkCyan => Some(Color::DarkCyan),
        ThemeColor::Grey => Some(Color::Grey),
        ThemeColor::DarkGrey => Some(Color::DarkGrey),
        ThemeColor::Red => Some(Color::Red),
        ThemeColor::Green => Some(Color::Green),
        ThemeColor::Yellow => Some(Color::Yellow),
        ThemeColor::Blue => Some(Color::Blue),
        ThemeColor::Magenta => Some(Color::Magenta),
        ThemeColor::Cyan => Some(Color::Cyan),
        ThemeColor::White => Some(Color::White),
        ThemeColor::AnsiValue(value) => Some(Color::AnsiValue(*value)),
        ThemeColor::Rgb { r, g, b } => Some(Color::Rgb {
            r: *r,
            g: *g,
            b: *b,
        }),
        ThemeColor::Reset => None,
    }
}
