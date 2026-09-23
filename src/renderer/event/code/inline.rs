use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_inline_code(&mut self, code: CowStr) -> Result<()> {
        if self.pending_callout_label_override {
            if self.in_link {
                self.current_link_text.push_str(&code);
            } else {
                self.pending_callout_label_buffer.push_str(&code);
            }
            return Ok(());
        }
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
            let styled_code = style.apply(&raw_code, self.output_style);
            table.current_cell.push_str(&styled_code);
            return Ok(());
        }

        let terminal_width = self.config.get_content_width();
        self.push_styled_inline_atom(&raw_code, &style, terminal_width);
        Ok(())
    }
}
