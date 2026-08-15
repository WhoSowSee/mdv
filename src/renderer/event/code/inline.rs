use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_inline_code(&mut self, code: CowStr) -> Result<()> {
        self.close_inline_backticks();
        let inline_style = self.theme.inline_style.get(InlineStyleKind::Code);
        let mut style = AnsiStyle::new().fg(self.theme.code.clone().into());
        if let Some(background) = self.theme.inline_background(InlineStyleKind::Code) {
            style = style.bg(background.clone().into());
        }
        style = inline_style.apply_attributes(style);

        self.register_footnotes_in_text(&code);

        let raw_code = if inline_style.backticks {
            format!("`{}`", code)
        } else {
            code.to_string()
        };
        self.note_paragraph_content();

        // Table cells: let the table renderer decide about wrapping; just push styled.
        if let Some(ref mut table) = self.table_state {
            let styled_code = style.apply(&raw_code, self.config.no_colors);
            table.current_cell.push_str(&styled_code);
            return Ok(());
        }

        // If wrapping is disabled, just push styled text
        let should_wrap = self.config.is_text_wrapping_enabled();
        if !should_wrap {
            let styled_code = style.apply(&raw_code, self.config.no_colors);
            self.output.push_str(&styled_code);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(());
        }

        let terminal_width = self.config.get_content_width();
        let wrap_mode = self.config.text_wrap_mode();

        // Remaining visible text to place (without ANSI)
        let mut remaining = raw_code.clone();

        while !remaining.is_empty() {
            // Compute available width on the current visual line (without ANSI)
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let available = terminal_width.saturating_sub(current_line_width);

            // If there's no room left on this line, start a new one with proper indentation
            if available == 0 {
                self.push_newline_with_context();
                continue;
            }

            let line_indent_width = self.compute_line_start_context_width();
            let effective_indent = line_indent_width.min(current_line_width);
            let has_line_content = current_line_width > effective_indent;
            let remaining_width = crate::utils::display_width(&remaining);

            match wrap_mode {
                WrapMode::Word => {
                    if remaining_width <= available {
                        // Fits entirely on this line
                        let styled = style.apply(&remaining, self.config.no_colors);
                        self.output.push_str(&styled);
                        remaining.clear();
                    } else if has_line_content {
                        // Current line already has visible content; move the code span to the next line
                        self.push_newline_with_context();
                    } else {
                        // Too long even for a fresh line – fall back to character splitting
                        let (chunk, rest) = self.take_prefix_by_width(&remaining, available);
                        let styled = style.apply(&chunk, self.config.no_colors);
                        self.output.push_str(&styled);
                        remaining = rest;
                        if !remaining.is_empty() {
                            self.push_newline_with_context();
                        }
                    }
                }
                WrapMode::Character | WrapMode::None => {
                    // Fill current line up to available width
                    let (chunk, rest) = self.take_prefix_by_width(&remaining, available);
                    let styled = style.apply(&chunk, self.config.no_colors);
                    self.output.push_str(&styled);
                    remaining = rest;
                    if !remaining.is_empty() {
                        self.push_newline_with_context();
                    }
                }
            }
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
