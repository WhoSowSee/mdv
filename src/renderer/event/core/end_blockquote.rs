use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_blockquote_end(&mut self) -> Result<()> {
        let callout_info = match self.callout_stack.last() {
            Some(CalloutState::Active(info)) => Some(info.clone()),
            _ => None,
        };
        let was_callout = callout_info.is_some();
        let spacing_element = if was_callout {
            BlockElement::Callout
        } else {
            BlockElement::Blockquote
        };
        let block_spacing = self.config.block_spacing.spacing(spacing_element);
        let callout_inline_links = callout_info
            .as_ref()
            .map(|info| info.inline_links.clone())
            .unwrap_or_default();
        let callout_level = self.blockquote_level;
        let closing_outer_blockquote = callout_level == 1;
        let start_index = self.blockquote_starts.pop().unwrap_or(self.output.len());
        let slice = if start_index <= self.output.len() {
            self.output[start_index..].to_string()
        } else {
            String::new()
        };
        let trimmed = strip_ansi(&slice);

        let mut has_visible_content = !trimmed.trim().is_empty();
        if was_callout {
            has_visible_content = true;
        }

        let use_pretty_callout =
            was_callout && matches!(self.config.callout_style.style, CalloutStyle::Pretty);

        if use_pretty_callout {
            if start_index <= self.output.len() {
                self.output.truncate(start_index);
            }

            self.callout_stack.pop();
            self.pending_callout_marker = false;
            self.pending_callout_marker_buffer.clear();
            self.pending_callout_label_override = false;
            self.pending_callout_label_buffer.clear();
            self.suppress_next_soft_break = false;
            self.blockquote_level = self.blockquote_level.saturating_sub(1);
            self.current_indent = self.current_indent.saturating_sub(2);
            if let Some((content_indent, heading_indent)) = self.blockquote_indent_stack.pop() {
                self.content_indent = content_indent;
                self.heading_indent = heading_indent;
            }
            self.active_blockquote_smart_indents.pop();

            if (has_visible_content || self.config.show_empty_elements)
                && let Some(info) = callout_info
            {
                let rendered = self.render_callout_pretty_block(
                    &slice,
                    callout_level,
                    info.kind,
                    &info.label,
                    info.label_override.as_deref(),
                    info.fold,
                );

                if !rendered {
                    self.output.push_str(&slice);
                    if !self.output.ends_with('\n') {
                        self.output.push('\n');
                    }
                }
            }

            if matches!(self.config.link_style, LinkStyle::InlineTable)
                && !callout_inline_links.is_empty()
            {
                self.trim_trailing_blank_lines();
                self.render_link_reference_blocks(&callout_inline_links, true, false, 0);
            }

            if (was_callout || closing_outer_blockquote)
                && (has_visible_content || self.config.show_empty_elements)
            {
                self.ensure_contextual_blank_lines(block_spacing.bottom);
            }

            return Ok(());
        }

        if !has_visible_content {
            if self.config.show_empty_elements {
                if start_index <= self.output.len() {
                    self.output.truncate(start_index);
                }
                if !self.output.ends_with('\n') && !self.output.is_empty() {
                    self.output.push('\n');
                }
                self.push_indent_for_line_start();
                if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            } else if start_index <= self.output.len() {
                self.output.truncate(start_index);
            }
        } else if !self.output.ends_with('\n') {
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
        if let Some((content_indent, heading_indent)) = self.blockquote_indent_stack.pop() {
            self.content_indent = content_indent;
            self.heading_indent = heading_indent;
        }
        self.active_blockquote_smart_indents.pop();

        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && !callout_inline_links.is_empty()
        {
            self.trim_trailing_blank_lines();
            self.render_link_reference_blocks(&callout_inline_links, true, false, 0);
        }

        if (was_callout || closing_outer_blockquote)
            && (has_visible_content || self.config.show_empty_elements)
        {
            self.ensure_contextual_blank_lines(block_spacing.bottom);
        }
        Ok(())
    }
}
