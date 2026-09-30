use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_link_end(&mut self) -> Result<()> {
        let link = self.links.current.take();
        if self.pending_callout_label_override {
            let link =
                link.ok_or_else(|| anyhow::anyhow!("Callout link end without a matching start"))?;
            self.pending_callout_label_buffer.push_str(&link.text);
            return Ok(());
        }
        if matches!(self.config.link_style, LinkStyle::Hide) {
            self.commit_pending_heading_placeholder_if_content();
            return Ok(());
        }
        let link = link.ok_or_else(|| anyhow::anyhow!("Link end without a matching start"))?;
        if self.table_state.is_none() && !matches!(self.config.link_style, LinkStyle::Hide) {
            self.note_paragraph_content();
        }

        match self.config.link_style {
            LinkStyle::Clickable | LinkStyle::ClickableForced => {
                self.finish_clickable_link(link)?
            }
            LinkStyle::Hide => {}
            LinkStyle::Inline => self.finish_inline_link(link)?,
            LinkStyle::InlineTable => self.finish_inline_table_link(link.text)?,
            LinkStyle::EndTable => self.finish_end_table_link(link.text)?,
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn finish_clickable_link(&mut self, link: CurrentLink) -> Result<()> {
        let (link_text, link_url) = link.into_direct()?;
        let force_underline = matches!(self.config.link_style, LinkStyle::ClickableForced);

        if let Some(ref mut table) = self.table_state {
            push_clickable_table_link(
                table,
                &link_text,
                Some(link_url.as_str()),
                self.output_style,
            );
        } else {
            self.process_clickable_text_with_wrapping(&link_text, &link_url, force_underline)?;
        }
        Ok(())
    }

    pub(super) fn finish_inline_table_link(&mut self, link_text: String) -> Result<()> {
        // InlineTable needs special handling in tables to avoid duplicating the
        // link text outside the table. If we're inside a table cell, write the
        // entire link (underlined text + reference) directly into the cell and
        // skip any rendering to the main output buffer.
        let reference_index = if self.table_state.is_some() {
            self.links.paragraph_counter
        } else {
            match self.callout_stack.last() {
                Some(CalloutState::Active(info)) => info.inline_link_counter,
                _ => self.links.paragraph_counter,
            }
        };
        let reference_text = format!("[{}]", reference_index);

        if let Some(ref mut table) = self.table_state {
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.output_style);

            push_underlined_table_link(table, &link_text, self.output_style);

            push_wrappable_table_reference(&mut table.current_cell, &styled_reference);
        } else {
            // 1) Render the link text underlined with proper wrapping
            let link_text = link_text.trim();
            if !link_text.is_empty() {
                self.process_underlined_text_with_wrapping(link_text)?;
            }

            // 2) Append the reference number after the text (wrap if needed)
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.output_style);

            // Decide if reference fits on current line
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let terminal_width = self.effective_text_width();
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let reference_width = crate::utils::display_width(&reference_text);

            if self.should_wrap_inline_text()
                && current_line_width + reference_width > terminal_width
            {
                self.push_newline_with_context();
            }
            self.output.push_str(&styled_reference);
        }

        Ok(())
    }

    pub(super) fn finish_end_table_link(&mut self, link_text: String) -> Result<()> {
        // Behave like InlineTable for inline markers but collect references for document-level table.
        if let Some(ref mut table) = self.table_state {
            let reference_text = format!("[{}]", self.links.paragraph_counter);
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.output_style);

            push_underlined_table_link(table, &link_text, self.output_style);

            push_wrappable_table_reference(&mut table.current_cell, &styled_reference);
        } else {
            let link_text = link_text.trim();
            if !link_text.is_empty() {
                self.process_underlined_text_with_wrapping(link_text)?;
            }

            let reference_text = format!("[{}]", self.links.paragraph_counter);
            let style = create_style(self.theme, ThemeElement::Link);
            let styled_reference = style.apply(&reference_text, self.output_style);

            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let terminal_width = self.effective_text_width();
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let reference_width = crate::utils::display_width(&reference_text);

            if self.should_wrap_inline_text()
                && current_line_width + reference_width > terminal_width
            {
                self.push_newline_with_context();
            }
            self.output.push_str(&styled_reference);
        }

        Ok(())
    }
}
