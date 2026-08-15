use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_media(&mut self, element: ElementRef<'_>) -> Result<bool> {
        let Some(source) = html_media_source(&element) else {
            return Ok(false);
        };

        let label = html_media_label(&element, &source);
        if let Some(ref mut table) = self.table_state {
            let marker = media_marker(&source);
            let separator = media_marker_leading_separator(&table.current_cell);
            table.current_cell.push_str(separator);
            table.current_cell.push_str(marker);
            table.current_cell.push_str(&label);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(true);
        }

        let marker = media_marker(&source);
        self.note_paragraph_content();
        self.prepare_html_media_line(marker, &label);

        let style = create_style(self.theme, ThemeElement::Link);
        let styled_marker = style.apply(marker, self.config.no_colors);
        let separator = media_marker_leading_separator(&self.output);
        self.output.push_str(separator);
        self.output.push_str(&styled_marker);
        if !label.is_empty() {
            self.process_segment_with_wrapping_and_formatting(&label, false, false)?;
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(true)
    }

    pub(super) fn prepare_html_media_line(&mut self, marker: &str, label: &str) {
        let line_start_idx = self.output.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
        let current_line = &self.output[line_start_idx..];
        if current_line.trim().is_empty() {
            self.output.truncate(line_start_idx);
            self.push_indent_for_line_start();
            return;
        }

        if !self.config.is_text_wrapping_enabled() {
            return;
        }

        let current_line_clean = strip_ansi(current_line);
        let current_width = display_width(&current_line_clean);
        let media_width = display_width(media_marker_leading_separator(&self.output))
            + display_width(marker)
            + display_width(label);
        let would_exceed = current_width + media_width > self.effective_text_width();
        let has_visible_text = current_line_clean
            .chars()
            .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃');

        if would_exceed && current_width > 0 && has_visible_text {
            self.push_newline_with_context();
        }
    }
}
