use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_list_end(&mut self) -> Result<()> {
        let Some(closed_list) = self.list_stack.pop() else {
            return Ok(());
        };
        let closed_top_level_list = self.list_stack.is_empty();
        let spacing_element = closed_list.spacing_element;
        let list_has_visible_items = closed_list.has_visible_items;

        if !list_has_visible_items {
            self.output
                .truncate(closed_list.block_start.min(self.output.len()));
            return Ok(());
        }

        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && closed_top_level_list
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references();
        } else if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        if closed_top_level_list {
            let spacing = self.config.block_spacing.spacing(spacing_element);
            self.ensure_contextual_blank_lines(spacing.bottom);
        }
        Ok(())
    }

    pub(super) fn handle_item_end(&mut self) -> Result<()> {
        let mut start_index = self.output.len();
        let mut has_content = false;
        let mut was_ordered = false;

        if let Some(list_state) = self.list_stack.last_mut() {
            start_index = list_state
                .current_item_start
                .unwrap_or(self.output.len())
                .min(self.output.len());
            let marker_end = list_state
                .current_item_marker_end
                .unwrap_or(start_index)
                .min(self.output.len());
            let slice = &self.output[marker_end..];
            has_content = !strip_ansi(slice).trim().is_empty();
            was_ordered = list_state.is_ordered;
        }

        if matches!(self.config.footnote_style, FootnoteStyle::Attached)
            && !self.current_inline_footnotes.is_empty()
        {
            self.finalize_inline_footnotes(true, true)?;
        }

        if self.pending_task_marker
            && self.config.show_empty_elements
            && self.is_custom_task_marker(&self.pending_task_marker_buffer)
        {
            self.strip_bullet_for_checkbox_item();
            let state = self
                .pending_task_marker_buffer
                .chars()
                .nth(1)
                .unwrap_or(' ');
            let marker = if self.config.pretty_checkbox.is_some() {
                self.styled_checkbox_marker(state)
            } else {
                let style = create_style(self.theme, ThemeElement::ListMarker);
                style.apply(&format!("[{state}]"), self.config.no_colors)
            };
            self.output.push_str(&marker);
            self.output.push(' ');
            has_content = true;
        }
        if let Some(list_state) = self.list_stack.last_mut() {
            if has_content || self.config.show_empty_elements {
                list_state.has_visible_items = true;
                if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            } else {
                self.output.truncate(start_index);
                if was_ordered {
                    list_state.counter = list_state.counter.saturating_sub(1);
                }
            }

            list_state.current_item_start = None;
            list_state.current_item_marker_start = None;
            list_state.current_item_marker_end = None;
        } else if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.pending_task_marker = false;
        self.pending_task_marker_buffer.clear();
        Ok(())
    }
}
