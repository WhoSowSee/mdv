use super::{CowStr, EventRenderer, PRETTY_ACCENT_COLOR, Result, ThemeElement, create_style};
use crate::terminal::AnsiStyle;
use crate::utils::{display_width, strip_ansi};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_html(&mut self, html: CowStr) -> Result<()> {
        if html.as_ref().trim() == crate::markdown::BLANK_LINE_MARKER {
            self.handle_explicit_blank_line();
            return Ok(());
        }
        self.render_html_fragment(&html)
    }

    pub(super) fn handle_inline_html(&mut self, html: CowStr) -> Result<()> {
        if html.as_ref().trim() == crate::markdown::BLANK_LINE_MARKER {
            self.handle_explicit_blank_line();
            return Ok(());
        }
        self.render_html_fragment(&html)
    }

    fn render_html_fragment(&mut self, html: &CowStr) -> Result<()> {
        if self.config.render_html {
            return self.render_html_fragment_buffering_blocks(html.as_ref());
        }

        let html_str = html.as_ref();
        let trimmed = html_str.trim();

        let is_comment = trimmed.starts_with("<!--") && trimmed.ends_with("-->");
        if is_comment && self.config.hide_comments {
            return Ok(());
        }

        self.note_paragraph_content();

        let mut followup_prefix: Option<String> = None;
        let line_start = self
            .output
            .rfind('\n')
            .map(|idx| idx.saturating_add(1))
            .unwrap_or(0);
        let current_line = &self.output[line_start..];

        if current_line.is_empty() {
            let prefix = self.current_line_prefix();
            if !prefix.is_empty() {
                self.push_indent_for_line_start();
                followup_prefix = Some(prefix);
            }
        } else {
            let prefix = self.current_line_prefix();
            if !prefix.is_empty() && current_line == prefix {
                followup_prefix = Some(prefix);
            }
        }

        let mut segments = html_str.split('\n').peekable();
        let mut first_segment = true;

        while let Some(segment) = segments.next() {
            if !first_segment {
                self.output.push('\n');
                if let Some(prefix) = followup_prefix.as_ref()
                    && !prefix.is_empty()
                    && (!segment.is_empty() || segments.peek().is_some())
                {
                    self.output.push_str(prefix);
                }
            } else {
                first_segment = false;
            }

            if segment.is_empty() {
                continue;
            }

            self.render_wrapped_html_segment(segment)?;
        }

        self.commit_pending_heading_placeholder_if_content();

        Ok(())
    }

    fn render_wrapped_html_segment(&mut self, segment: &str) -> Result<()> {
        let formatting_stack =
            std::mem::replace(&mut self.formatting_stack, vec![ThemeElement::Text]);
        let result = self.process_segment_with_wrapping_and_formatting(
            segment,
            false,
            self.table_state.is_some(),
        );
        self.formatting_stack = formatting_stack;
        result
    }

    pub(super) fn handle_horizontal_rule(&mut self) -> Result<()> {
        self.reset_explicit_blank_line_streak();
        let prefix = self.current_rule_prefix();
        let prefix_width = display_width(&strip_ansi(&prefix));
        let width = self
            .effective_text_width()
            .saturating_sub(prefix_width)
            .max(1);
        let rule = if width >= 2 {
            format!("◈{}◈", "─".repeat(width.saturating_sub(2)))
        } else {
            "─".repeat(width)
        };
        let styled_rule = AnsiStyle::new()
            .fg(PRETTY_ACCENT_COLOR)
            .apply(&rule, self.config.no_colors);

        if !self.output.is_empty() {
            self.ensure_contextual_blank_line_with_prefix(&prefix);
        }

        if !prefix.is_empty() {
            self.output.push_str(&prefix);
        }
        self.output.push_str(&styled_rule);
        self.output.push('\n');
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_footnote_reference(&mut self, name: CowStr) -> Result<()> {
        self.note_paragraph_content();
        self.register_footnote_reference(name.as_ref());

        let marker = format!("[^{}]", name);
        let should_highlight = self.should_highlight_footnote_reference(name.as_ref());
        if let Some(ref mut table) = self.table_state {
            if should_highlight {
                let style = create_style(self.theme, ThemeElement::Link);
                let styled_marker = style.apply(&marker, self.config.no_colors);
                table
                    .inline_references
                    .push((marker.clone(), styled_marker));
            }
            table.current_cell.push_str(&marker);
        } else {
            let rendered_marker = if should_highlight {
                let style = create_style(self.theme, ThemeElement::Link);
                style.apply(&marker, self.config.no_colors)
            } else {
                marker
            };
            self.output.push_str(&rendered_marker);
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_task_list_marker(&mut self, checked: bool) -> Result<()> {
        self.note_paragraph_content();
        self.pending_task_marker = false;
        self.pending_task_marker_buffer.clear();
        if self.config.pretty_checkbox.is_some() {
            self.strip_bullet_for_checkbox_item();
            if let Some(list_state) = self.list_stack.last_mut() {
                list_state.current_item_marker_end = Some(self.output.len());
            }
        }
        let marker = if self.config.pretty_checkbox.is_some() {
            self.styled_checkbox_marker(if checked { 'x' } else { ' ' })
        } else if checked {
            let style = create_style(self.theme, ThemeElement::ListMarker);
            style.apply("[✓]", self.config.no_colors)
        } else {
            let style = create_style(self.theme, ThemeElement::ListMarker);
            style.apply("[ ]", self.config.no_colors)
        };
        self.output.push_str(&marker);
        self.output.push(' ');
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    /// Returns the checkbox icon. Callers add the separating space.
    pub(super) fn styled_checkbox_marker(&self, state: char) -> String {
        let shape = self
            .config
            .pretty_checkbox
            .expect("pretty checkbox rendering is enabled");

        let (icon, custom_color) = match self.config.checkbox_overrides.get(&state) {
            Some(ov) => {
                let icon = ov.icon.clone().or_else(|| {
                    crate::checkbox::default_icon(shape, state)
                        .or_else(|| crate::checkbox::default_icon(shape, ' '))
                        .map(|ch| ch.to_string())
                });
                (icon, ov.color.clone())
            }
            None => (
                crate::checkbox::default_icon(shape, state).map(|ch| ch.to_string()),
                None,
            ),
        };

        let style = match custom_color {
            Some(color) => create_style(self.theme, ThemeElement::ListMarker).fg(color.into()),
            None => create_style(self.theme, ThemeElement::ListMarker),
        };

        match icon {
            Some(glyph) => style.apply(&glyph, self.config.no_colors),
            None => style.apply(&format!("[{state}]"), self.config.no_colors),
        }
    }

    pub(super) fn strip_bullet_for_checkbox_item(&mut self) {
        let Some(list_state) = self.list_state_for_strip() else {
            return;
        };
        let (start, marker_end) = list_state;
        if start >= marker_end || marker_end > self.output.len() {
            return;
        }
        let segment = &self.output[start..marker_end];
        let stripped = strip_ansi(segment);
        if let Some(pos) = stripped.rfind("- ") {
            let byte_pos = Self::ansi_aware_byte_offset(segment, pos);
            let keep_until = start + byte_pos;
            let after = self.output[marker_end..].to_string();
            self.output.truncate(keep_until);
            self.output.push_str(&after);
        }
    }

    fn list_state_for_strip(&self) -> Option<(usize, usize)> {
        let list_state = self.list_stack.last()?;
        if list_state.is_ordered {
            return None;
        }
        Some((
            list_state.current_item_start?,
            list_state.current_item_marker_end?,
        ))
    }

    fn ansi_aware_byte_offset(original: &str, char_offset: usize) -> usize {
        let stripped = strip_ansi(original);
        let prefix = stripped[..char_offset.min(stripped.len())].to_string();
        let mut consumed = 0usize;
        let mut byte_idx = 0usize;
        let original_bytes = original.as_bytes();
        while byte_idx < original_bytes.len() && consumed < prefix.len() {
            if original_bytes[byte_idx] == 0x1b {
                byte_idx += 1;
                while byte_idx < original_bytes.len() && original_bytes[byte_idx] != b'm' {
                    byte_idx += 1;
                }
                byte_idx = byte_idx.saturating_add(1);
                continue;
            }
            let ch_len = std::str::from_utf8(&original_bytes[byte_idx..])
                .ok()
                .and_then(|s| s.chars().next())
                .map(|ch| ch.len_utf8())
                .unwrap_or(1);
            consumed += ch_len;
            byte_idx += ch_len;
        }
        byte_idx
    }
}
