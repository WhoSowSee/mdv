use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_blockquote(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_table_blockquote(element, context);
        }

        self.blockquote_indent_stack
            .push((self.content_indent, self.heading_indent));
        self.blockquote_starts.push(self.output.len());
        self.active_blockquote_smart_indents
            .push(Default::default());
        self.blockquote_level += 1;
        self.current_indent += 2;
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.callout_stack.push(CalloutState::None);

        let result = self.render_html_children(element, context);
        if result.is_ok() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.callout_stack.pop();
        self.pending_callout_marker = false;
        self.pending_callout_marker_buffer.clear();
        self.pending_callout_label_override = false;
        self.pending_callout_label_buffer.clear();
        self.suppress_next_soft_break = false;
        self.blockquote_level = self.blockquote_level.saturating_sub(1);
        self.current_indent = self.current_indent.saturating_sub(2);
        self.blockquote_starts.pop();
        if let Some((content_indent, heading_indent)) = self.blockquote_indent_stack.pop() {
            self.content_indent = content_indent;
            self.heading_indent = heading_indent;
        }
        self.active_blockquote_smart_indents.pop();
        self.flush_html_inline_table_references();
        result
    }
}
