use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn process_text_with_wrapping_and_formatting(&mut self, text: &str) -> Result<()> {
        if !text.contains("==") {
            return self.process_segment_with_wrapping_and_formatting(
                text,
                false,
                self.table_state.is_some(),
            );
        }

        for segment in self.split_highlight_segments(text) {
            if segment.text.is_empty() {
                if !segment.highlighted {
                    self.close_inline_backticks();
                }
                continue;
            }
            self.process_segment_with_wrapping_and_formatting(
                &segment.text,
                segment.highlighted,
                self.table_state.is_some(),
            )?;
        }

        Ok(())
    }

    pub(in crate::renderer::event) fn process_segment_with_wrapping_and_formatting(
        &mut self,
        text: &str,
        highlighted: bool,
        is_table_cell: bool,
    ) -> Result<()> {
        let prefixed_text;
        let text = if self.sync_inline_backticks(highlighted) {
            prefixed_text = format!("`{text}");
            prefixed_text.as_str()
        } else {
            text
        };

        // Check if this is for a table cell
        if is_table_cell {
            // For table cells, apply formatting directly without complex wrapping
            let formatted_text = self.apply_formatting_with_highlight(text, highlighted);
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(&formatted_text);
            }
            return Ok(());
        }

        // Add blockquote prefix if we're starting new content in a blockquote
        // Check if we're at the start of a line (after newline or any whitespace-only content)
        if self.blockquote_level > 0 {
            let after_newline = self.output.ends_with('\n');
            let at_start = self.output.is_empty();
            let at_line_start = if let Some(last_newline_pos) = self.output.rfind('\n') {
                // Check if everything after the last newline is just whitespace
                self.output[last_newline_pos + 1..].trim().is_empty()
            } else {
                // No newlines, check if entire output is just whitespace
                self.output.trim().is_empty()
            };

            if after_newline || at_start || at_line_start {
                let prefix = self.current_line_prefix();
                if !prefix.is_empty() {
                    self.output.push_str(&prefix);
                }
            }
        }

        // Check if we need to wrap text. When no explicit cols are provided,
        // wrap to the detected terminal width (unless --no-wrap is set).
        let should_wrap = self.config.is_text_wrapping_enabled() && !self.in_properties_callout();

        if should_wrap && !self.formatting_stack.is_empty() {
            // For styled text, prefer continuous decoration for strike-through
            if self.formatting_stack.contains(&ThemeElement::Strikethrough) {
                self.process_strikethrough_text_with_wrapping(text, highlighted)?;
            } else {
                // Default styled processing (per-unit formatting)
                self.process_styled_text_with_wrapping(text, highlighted)?;
            }
        } else {
            // Regular text processing
            self.process_regular_text(text, should_wrap, highlighted)?;
        }

        Ok(())
    }

    pub(super) fn split_highlight_segments(&self, text: &str) -> Vec<HighlightSegment> {
        let mut segments = Vec::with_capacity(4);
        let mut buffer = String::new();
        let mut highlighted = false;
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '=' && matches!(chars.peek(), Some('=')) {
                chars.next(); // consume second '='
                if !buffer.is_empty() {
                    segments.push(HighlightSegment {
                        text: std::mem::take(&mut buffer),
                        highlighted,
                    });
                }
                highlighted = !highlighted;
                continue;
            }
            buffer.push(ch);
        }

        segments.push(HighlightSegment {
            text: if highlighted {
                format!("=={}", buffer)
            } else {
                buffer
            },
            highlighted,
        });

        segments
    }
}
