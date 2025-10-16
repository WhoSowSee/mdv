use super::{CowStr, EventRenderer, Result, ThemeElement, create_style};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_image_start(&mut self, _dest_url: CowStr) -> Result<()> {
        // If we are inside a table, write the marker into the current cell
        if let Some(ref mut table) = self.table_state {
            let style = create_style(self.theme, ThemeElement::Link);
            let image_marker = style.apply("[IMAGE] ", self.config.no_colors);
            table.current_cell.push_str(&image_marker);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(());
        }

        // Ensure correct indentation/prefix when an image starts a visual line.
        // Paragraph start may have added spaces, but when inside lists/quotes
        // there may be no prefix yet. If the current line contains only
        // whitespace, normalize it and insert the proper context-aware prefix.
        let line_start_idx = self.output.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let current_line = &self.output[line_start_idx..];
        if current_line.trim().is_empty() {
            // Drop any existing leading spaces on the current visual line
            // (e.g. content indent added at paragraph start) to avoid double
            // indentation, then re-apply consistent prefix/indent.
            self.output.truncate(line_start_idx);
            self.push_indent_for_line_start();
        }

        let style = create_style(self.theme, ThemeElement::Link);
        let image_marker = style.apply("[IMAGE] ", self.config.no_colors);
        self.output.push_str(&image_marker);
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_image_end(&mut self) -> Result<()> {
        // Image handling is completed in start
        Ok(())
    }
}
