use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_paragraph_end(&mut self) -> Result<()> {
        self.close_inline_backticks();
        self.finalize_pending_callout_label_override();
        let paragraph_start = self.current_paragraph_start.take();
        let paragraph_has_content = self.current_paragraph_has_content;
        let paragraph_has_leading_break = self.current_paragraph_has_leading_break;
        self.current_paragraph_has_content = false;
        self.current_paragraph_has_leading_break = false;
        self.suppress_next_soft_break = false;
        let suppress_break = self.suppress_next_paragraph_break;
        self.suppress_next_paragraph_break = false;

        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references();
        }

        let inline_footnotes_rendered =
            matches!(self.config.footnote_style, FootnoteStyle::Attached)
                && self.has_renderable_footnotes(&self.current_inline_footnotes)
                && !self.suppress_footnote_output;

        self.finalize_inline_footnotes(true, !self.list_stack.is_empty())?;

        let has_visible_content = paragraph_has_content
            || paragraph_start.is_some_and(|start| {
                let slice = if start <= self.output.len() {
                    &self.output[start..]
                } else {
                    ""
                };
                let clean = strip_ansi(slice);
                clean
                    .chars()
                    .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃')
            });

        if paragraph_has_leading_break && !has_visible_content {
            self.trim_trailing_blank_lines();
            self.ensure_contextual_blank_line();
            return Ok(());
        }

        if !has_visible_content && self.blockquote_level > 0 {
            self.ensure_contextual_blank_line();
            return Ok(());
        }

        let skip_blank_line = self.blockquote_level > 0
            && self.trailing_blank_line_matches(&self.current_line_prefix());

        if self.list_stack.is_empty() && !self.in_definition_description() {
            if !inline_footnotes_rendered && !suppress_break && !skip_blank_line {
                if has_visible_content && self.blockquote_level == 0 {
                    let spacing = self.config.block_spacing.spacing(BlockElement::Paragraph);
                    self.ensure_contextual_blank_lines(spacing.bottom);
                } else {
                    self.output.push('\n');
                }
            }
        } else if has_visible_content && !suppress_break && !self.output.ends_with('\n') {
            // Paragraph boundaries inside list items must end with a line break
            // so continuation blocks (e.g. indented images/media) start on next line.
            self.output.push('\n');
        }
        Ok(())
    }
}
