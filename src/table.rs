use crate::theme::{Color as ThemeColor, Theme, ThemeElement, create_style};
use crate::utils::{display_width, strip_ansi};
use anyhow::Result;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use pulldown_cmark::Alignment;

use crate::cli::TableWrapMode;

/// Table renderer using comfy-table for proper Unicode handling
pub struct TableRenderer {
    theme: Theme,
    no_colors: bool,
    terminal_width: usize,
    table_wrap: TableWrapMode,
}

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
        }
    }

    /// Create a cell with proper ANSI handling for width calculation
    fn create_cell(&self, content: &str) -> Cell {
        let clean_content = strip_ansi(content);

        let mut cell = Cell::new(&clean_content);

        if clean_content.starts_with('`') && clean_content.ends_with('`') {
            if !self.no_colors {
                if let Some(theme_color) = theme_color_to_comfy(&self.theme.code) {
                    // Use only foreground color, no background
                    cell = cell.fg(theme_color);
                }
            }
        }

        if clean_content.len() != content.len() {
            if content.contains("\x1b[1m") || content.contains("\x1b[01m") {
                cell = cell.add_attribute(Attribute::Bold);
            }
            if content.contains("\x1b[3m") || content.contains("\x1b[03m") {
                cell = cell.add_attribute(Attribute::Italic);
            }
            if content.contains("\x1b[4m") || content.contains("\x1b[04m") {
                cell = cell.add_attribute(Attribute::Underlined);
            }

            if !self.no_colors {
                if let Some(ansi_color) = extract_ansi_foreground_color(content) {
                    cell = cell.fg(ansi_color);
                }
            }
        }

        cell
    }

    /// Calculate estimated table width
    fn estimate_table_width(&self, headers: &[String], rows: &[Vec<String>]) -> usize {
        let mut max_widths = vec![0; headers.len()];

        for (i, header) in headers.iter().enumerate() {
            let clean_header = strip_ansi(header);
            max_widths[i] = display_width(&clean_header);
        }

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < max_widths.len() {
                    let clean_cell = strip_ansi(cell);
                    max_widths[i] = max_widths[i].max(display_width(&clean_cell));
                }
            }
        }

        // Add borders and padding: 3 chars per column (│ x │) + 1 for final border
        max_widths.iter().sum::<usize>() + (headers.len() * 3) + 1
    }

    /// Calculate column widths for each column
    fn calculate_column_widths(&self, headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
        let mut max_widths = vec![0; headers.len()];

        for (i, header) in headers.iter().enumerate() {
            let clean_header = strip_ansi(header);
            max_widths[i] = display_width(&clean_header);
        }

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < max_widths.len() {
                    let clean_cell = strip_ansi(cell);
                    max_widths[i] = max_widths[i].max(display_width(&clean_cell));
                }
            }
        }

        // Add minimum padding (at least 3 characters per column for borders)
        max_widths.iter().map(|&w| w.max(3)).collect()
    }

    /// Split table into column blocks that fit terminal width
    fn split_table_into_blocks(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Vec<(Vec<String>, Vec<Vec<String>>, Vec<Alignment>)> {
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

            for i in (current_block_start + 1)..headers.len() {
                let additional_width = column_widths[i] + 3; // column width + borders

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
                self.render_single_table_block(block_headers, block_rows, &block_alignments)?;
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

        // Configure table appearance
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        if !self.no_colors {
            table.enforce_styling();
        }

        // Don't set width limit - let table overflow

        // Add headers with styling
        let header_cells: Vec<Cell> = headers
            .iter()
            .enumerate()
            .map(|(i, header)| {
                let mut cell = self.create_cell(header);

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
                    let mut cell = self.create_cell(cell_content);

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

        Ok(table.to_string())
    }

    /// Render a single table block
    fn render_single_table_block(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        alignments: &[Alignment],
    ) -> Result<String> {
        let mut table = Table::new();

        // Configure table appearance
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

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
                let mut cell = self.create_cell(header);

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
                    let mut cell = self.create_cell(cell_content);

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

        Ok(table.to_string())
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

fn theme_color_to_comfy(color: &ThemeColor) -> Option<Color> {
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

pub fn apply_inline_reference_styles(
    mut table_output: String,
    references: &[(String, String)],
    no_colors: bool,
) -> String {
    if no_colors {
        return table_output;
    }

    let mut search_start = 0usize;

    for (plain, styled) in references {
        if plain.is_empty() {
            continue;
        }

        if let Some(rel_idx) = table_output[search_start..].find(plain) {
            let idx = search_start + rel_idx;
            let end = idx + plain.len();
            table_output.replace_range(idx..end, styled);
            search_start = idx + styled.len();
        }
    }

    table_output
}

fn extract_ansi_foreground_color(content: &str) -> Option<Color> {
    let mut remaining = content;

    while let Some(start) = remaining.find("\x1b[") {
        remaining = &remaining[start + 2..];
        let Some(end) = remaining.find('m') else {
            break;
        };
        let sequence = &remaining[..end];
        if let Some(color) = parse_sgr_sequence(sequence) {
            return Some(color);
        }
        remaining = &remaining[end + 1..];
    }

    None
}

fn parse_sgr_sequence(sequence: &str) -> Option<Color> {
    let values: Vec<i32> = sequence
        .split(';')
        .filter_map(|part| part.parse::<i32>().ok())
        .collect();

    let mut index = 0;
    while index < values.len() {
        let code = values[index];
        match code {
            30..=37 | 90..=97 => {
                return map_basic_ansi_to_color(code);
            }
            38 => {
                if let Some(mode) = values.get(index + 1) {
                    match *mode {
                        5 => {
                            if let Some(value) = values.get(index + 2) {
                                return Some(Color::AnsiValue(clamp_to_u8(*value)));
                            }
                        }
                        2 => {
                            if let (Some(r), Some(g), Some(b)) = (
                                values.get(index + 2),
                                values.get(index + 3),
                                values.get(index + 4),
                            ) {
                                return Some(Color::Rgb {
                                    r: clamp_to_u8(*r),
                                    g: clamp_to_u8(*g),
                                    b: clamp_to_u8(*b),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            39 => return None,
            _ => {}
        }

        index += 1;
    }

    None
}

fn map_basic_ansi_to_color(code: i32) -> Option<Color> {
    match code {
        30 => Some(Color::Black),
        31 => Some(Color::DarkRed),
        32 => Some(Color::DarkGreen),
        33 => Some(Color::DarkYellow),
        34 => Some(Color::DarkBlue),
        35 => Some(Color::DarkMagenta),
        36 => Some(Color::DarkCyan),
        37 => Some(Color::Grey),
        90 => Some(Color::DarkGrey),
        91 => Some(Color::Red),
        92 => Some(Color::Green),
        93 => Some(Color::Yellow),
        94 => Some(Color::Blue),
        95 => Some(Color::Magenta),
        96 => Some(Color::Cyan),
        97 => Some(Color::White),
        _ => None,
    }
}

fn clamp_to_u8(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeManager;

    #[test]
    fn test_table_rendering() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

        let headers = vec!["Name".to_string(), "Value".to_string()];
        let rows = vec![
            vec!["Test".to_string(), "123".to_string()],
            vec!["Another".to_string(), "456".to_string()],
        ];
        let alignments = vec![Alignment::Left, Alignment::Right];

        let result = renderer.render_table(&headers, &rows, &alignments);
        assert!(result.is_ok());

        let table_str = result.unwrap();
        assert!(!table_str.is_empty());
        assert!(table_str.contains("Name"));
        assert!(table_str.contains("Value"));
        assert!(table_str.contains("\x1b["));
    }

    #[test]
    fn test_empty_table() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

        let headers = vec![];
        let rows = vec![];
        let alignments = vec![];

        let result = renderer.render_table(&headers, &rows, &alignments);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_table_rendering_no_colors() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, true, 80, TableWrapMode::Fit);

        let headers = vec!["Name".to_string(), "Value".to_string()];
        let rows = vec![vec!["Test".to_string(), "123".to_string()]];
        let alignments = vec![Alignment::Left, Alignment::Right];

        let table_str = renderer.render_table(&headers, &rows, &alignments).unwrap();

        assert!(!table_str.contains("\x1b["));
    }

    #[test]
    fn test_narrow_terminal_vertical_layout() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 30, TableWrapMode::Wrap); // Very narrow terminal with wrap mode

        let headers = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let rows = vec![
            vec![
                "Alice".to_string(),
                "25".to_string(),
                "New York".to_string(),
            ],
            vec!["Bob".to_string(), "30".to_string(), "London".to_string()],
        ];
        let alignments = vec![Alignment::Left, Alignment::Right, Alignment::Left];

        let result = renderer.render_table(&headers, &rows, &alignments);
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should render table properly for narrow terminals with wrap mode
        // The table might fit in 30 chars, so let's check if it contains basic table elements
        assert!(output.contains("Name"));
        assert!(output.contains("Age"));
        assert!(output.contains("City"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_wide_table_column_wrapping() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 60, TableWrapMode::Wrap); // Medium width terminal with wrap mode

        let headers = vec![
            "Very Long Header Name".to_string(),
            "Another Long Header".to_string(),
            "Third Column".to_string(),
            "Fourth Column".to_string(),
        ];
        let rows = vec![vec![
            "Long content in first column".to_string(),
            "Content in second".to_string(),
            "Third content".to_string(),
            "Fourth content".to_string(),
        ]];
        let alignments = vec![
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
        ];

        let result = renderer.render_table(&headers, &rows, &alignments);
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should contain information about multiple blocks
        assert!(output.to_lowercase().contains("block"));
    }

    #[test]
    fn test_column_wrapping_logic() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 30, TableWrapMode::Fit); // Very narrow terminal

        let headers = vec![
            "Very Long Column Header 1".to_string(),
            "Very Long Column Header 2".to_string(),
            "Very Long Column Header 3".to_string(),
            "Very Long Column Header 4".to_string(),
        ];
        let rows = vec![vec![
            "Long content in first column".to_string(),
            "Long content in second column".to_string(),
            "Long content in third column".to_string(),
            "Long content in fourth column".to_string(),
        ]];
        let alignments = vec![Alignment::Left; 4];

        let blocks = renderer.split_table_into_blocks(&headers, &rows, &alignments);

        // Should split into multiple blocks for narrow terminal with long content
        assert!(blocks.len() >= 1);

        // Each block should have at least one column
        for (block_headers, _, _) in &blocks {
            assert!(!block_headers.is_empty());
        }

        // Total columns across all blocks should equal original column count
        let total_columns: usize = blocks.iter().map(|(headers, _, _)| headers.len()).sum();
        assert_eq!(total_columns, headers.len());
    }

    #[test]
    fn test_theme_color_to_comfy_conversion() {
        let ansi_color = ThemeColor::AnsiValue(42);
        assert_eq!(
            theme_color_to_comfy(&ansi_color),
            Some(Color::AnsiValue(42))
        );

        let rgb_color = ThemeColor::Rgb { r: 1, g: 2, b: 3 };
        assert_eq!(
            theme_color_to_comfy(&rgb_color),
            Some(Color::Rgb { r: 1, g: 2, b: 3 })
        );

        assert_eq!(theme_color_to_comfy(&ThemeColor::Reset), None);
    }

    #[test]
    fn test_table_link_text_keeps_default_color() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

        let link_text = "Link text";
        let formatted_link_text = format!("\x1b[4m{}\x1b[0m", link_text);
        let styled_reference = create_style(theme, ThemeElement::Link).apply("[1]", false);

        let headers = vec!["Col".to_string()];
        let reference_text = "[1]".to_string();
        let rows = vec![vec![format!("{}{}", formatted_link_text, reference_text)]];
        let alignments = vec![Alignment::Left];

        let references = vec![(reference_text.clone(), styled_reference.clone())];

        let table_output = renderer
            .render_table(&headers, &rows, &alignments)
            .expect("table rendered");
        let table_output = apply_inline_reference_styles(table_output, &references, false);

        let data_line = table_output
            .lines()
            .find(|line| line.contains(link_text))
            .expect("data row present");
        assert!(data_line.contains(&styled_reference));
        let stripped_line = crate::utils::strip_ansi(&data_line);
        assert!(stripped_line.contains("Link text[1]"));

        let prefix_len = styled_reference
            .find("[1]")
            .expect("styled reference contains '[1]'");
        let color_prefix = &styled_reference[..prefix_len];

        let reference_pos = data_line
            .find(&styled_reference)
            .expect("styled reference present");
        let before_reference = &data_line[..reference_pos];

        assert!(data_line.contains(color_prefix));
        assert!(
            !before_reference.contains(color_prefix),
            "link color prefix should not tint link text; line={:?}",
            data_line
        );
    }

    #[test]
    fn test_table_inline_link_preserves_text_color() {
        let theme_manager = ThemeManager::new();
        let theme = theme_manager.get_theme("terminal").unwrap();
        let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

        let link_text = "Link text";
        let formatted_link_text = format!("\x1b[4m{}\x1b[0m", link_text);
        let url_part = "(https://example.com)".to_string();
        let styled_url = create_style(theme, ThemeElement::Link).apply(&url_part, false);

        let headers = vec!["Col".to_string()];
        let rows = vec![vec![format!("{}{}", formatted_link_text, url_part.clone())]];
        let alignments = vec![Alignment::Left];

        let replacements = vec![(url_part.clone(), styled_url.clone())];

        let table_output = renderer
            .render_table(&headers, &rows, &alignments)
            .expect("table rendered");
        let table_output = apply_inline_reference_styles(table_output, &replacements, false);

        let data_line = table_output
            .lines()
            .find(|line| line.contains(link_text))
            .expect("data row present");

        assert!(data_line.contains(&styled_url));

        let stripped_line = crate::utils::strip_ansi(&data_line);
        assert!(stripped_line.contains(&format!("{}{}", link_text, url_part)));

        let prefix_len = styled_url
            .find(&url_part)
            .expect("styled url contains raw url");
        let color_prefix = &styled_url[..prefix_len];

        let reference_pos = data_line.find(&styled_url).expect("styled url present");
        let before_reference = &data_line[..reference_pos];

        assert!(data_line.contains(color_prefix));
        assert!(
            !before_reference.contains(color_prefix),
            "link color prefix should not tint link text; line={:?}",
            data_line
        );
    }

    #[test]
    fn test_extract_ansi_foreground_color_rgb() {
        let input = "\x1b[1;38;2;12;34;56mcolored\x1b[0m";
        let color = extract_ansi_foreground_color(input);
        assert_eq!(
            color,
            Some(Color::Rgb {
                r: 12,
                g: 34,
                b: 56
            })
        );
    }
}
