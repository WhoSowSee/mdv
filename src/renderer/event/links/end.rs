use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_link_end(&mut self) -> Result<()> {
        if self.table_state.is_none() && !matches!(self.config.link_style, LinkStyle::Hide) {
            self.note_paragraph_content();
        }

        match self.config.link_style {
            LinkStyle::Clickable | LinkStyle::ClickableForced => self.finish_clickable_link()?,
            LinkStyle::Hide => {}
            LinkStyle::Inline => self.finish_inline_link()?,
            LinkStyle::InlineTable => self.finish_inline_table_link()?,
            LinkStyle::EndTable => self.finish_end_table_link()?,
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn finish_clickable_link(&mut self) -> Result<()> {
        let link_text = self.current_link_text.clone();
        let current_link_key = format!("current_{}", self.link_counter);
        let link_url = self.link_references.get(&current_link_key).cloned();
        let force_underline = matches!(self.config.link_style, LinkStyle::ClickableForced);

        if let Some(ref mut table) = self.table_state {
            push_clickable_table_link(
                table,
                &link_text,
                link_url.as_deref(),
                self.config.no_colors,
            );
        } else if let Some(url) = link_url.as_deref() {
            self.process_clickable_text_with_wrapping(&link_text, url, force_underline)?;
        }
        self.in_link = false;
        self.current_link_text.clear();
        Ok(())
    }

    pub(super) fn finish_inline_table_link(&mut self) -> Result<()> {
        // InlineTable needs special handling in tables to avoid duplicating the
        // link text outside the table. If we're inside a table cell, write the
        // entire link (underlined text + reference) directly into the cell and
        // skip any rendering to the main output buffer.
        let reference_index = if self.table_state.is_some() {
            self.paragraph_link_counter
        } else {
            match self.callout_stack.last() {
                Some(CalloutState::Active(info)) => info.inline_link_counter,
                _ => self.paragraph_link_counter,
            }
        };
        let reference_text = format!("[{}]", reference_index);

        if let Some(ref mut table) = self.table_state {
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.config.no_colors);

            push_underlined_table_link(table, &self.current_link_text, self.config.no_colors);

            push_wrappable_table_reference(&mut table.current_cell, &styled_reference);
        } else {
            // 1) Render the link text underlined with proper wrapping
            let link_text = self.current_link_text.trim().to_string();
            if !link_text.is_empty() {
                self.process_underlined_text_with_wrapping(&link_text)?;
            }

            // 2) Append the reference number after the text (wrap if needed)
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.config.no_colors);

            // Decide if reference fits on current line
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let terminal_width = self.effective_text_width();
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let reference_width = crate::utils::display_width(&reference_text);

            if self.config.is_text_wrapping_enabled()
                && current_line_width + reference_width > terminal_width
            {
                self.push_newline_with_context();
            }
            self.output.push_str(&styled_reference);
        }

        self.in_link = false;
        self.current_link_text.clear();
        Ok(())
    }

    pub(super) fn finish_end_table_link(&mut self) -> Result<()> {
        // Behave like InlineTable for inline markers but collect references for document-level table.
        if let Some(ref mut table) = self.table_state {
            let reference_text = format!("[{}]", self.paragraph_link_counter);
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.config.no_colors);

            push_underlined_table_link(table, &self.current_link_text, self.config.no_colors);

            push_wrappable_table_reference(&mut table.current_cell, &styled_reference);
        } else {
            let link_text = self.current_link_text.trim().to_string();
            if !link_text.is_empty() {
                self.process_underlined_text_with_wrapping(&link_text)?;
            }

            let reference_text = format!("[{}]", self.paragraph_link_counter);
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.config.no_colors);

            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let terminal_width = self.effective_text_width();
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let reference_width = crate::utils::display_width(&reference_text);

            if self.config.is_text_wrapping_enabled()
                && current_line_width + reference_width > terminal_width
            {
                self.push_newline_with_context();
            }
            self.output.push_str(&styled_reference);
        }

        self.in_link = false;
        self.current_link_text.clear();
        Ok(())
    }
}
