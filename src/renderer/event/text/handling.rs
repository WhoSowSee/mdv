use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn line_has_visible_text(line: &str) -> bool {
        line.chars()
            .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃')
    }

    pub(in crate::renderer::event) fn handle_text(&mut self, text: CowStr) -> Result<()> {
        if !self.in_code_block && !self.in_link {
            self.scan_footnotes_in_text_stream(&text);
        }

        if self.in_code_block {
            self.pending_task_marker = false;
            self.pending_task_marker_buffer.clear();
            self.code_block_content.push_str(&text);
            return Ok(());
        } else if self.in_link {
            self.current_link_text.push_str(&text);
            return Ok(());
        }

        let raw_text = text.as_ref();
        if self.pending_callout_label_override {
            if self.pending_callout_label_buffer.is_empty() {
                let mut remaining = raw_text;

                if let Some(first) = remaining.chars().next()
                    && matches!(first, '+' | '-')
                {
                    if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut()
                        && info.fold.is_none()
                    {
                        info.fold = Some(match first {
                            '+' => CalloutFold::Expanded,
                            '-' => CalloutFold::Collapsed,
                            _ => unreachable!(),
                        });
                    }
                    remaining = &remaining[first.len_utf8()..];
                    if remaining.is_empty() {
                        return Ok(());
                    }
                }

                let starts_with_ws = remaining
                    .chars()
                    .next()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(false);

                if !starts_with_ws {
                    if self.finalize_pending_callout_label_override() {
                        self.suppress_next_soft_break = true;
                    }
                    return Ok(());
                }

                self.pending_callout_label_buffer.push_str(remaining);
                return Ok(());
            }

            self.pending_callout_label_buffer.push_str(raw_text);
            return Ok(());
        }
        let mut callout_decision = None;
        if self.blockquote_level > 0
            && self.list_stack.is_empty()
            && let Some(state) = self.callout_stack.last_mut()
            && matches!(state, CalloutState::Pending)
        {
            if self.pending_callout_marker {
                self.pending_callout_marker_buffer.push_str(raw_text);
                let evaluation = Self::evaluate_callout_buffer(&self.pending_callout_marker_buffer);
                callout_decision = Some(Self::apply_callout_buffer_evaluation(
                    state,
                    evaluation,
                    &self.pending_callout_marker_buffer,
                ));
            } else if raw_text.trim().is_empty() {
                // Keep pending until we see meaningful content.
                return Ok(());
            } else if raw_text.trim_start().starts_with('[') {
                self.pending_callout_marker = true;
                self.pending_callout_marker_buffer.clear();
                self.pending_callout_marker_buffer.push_str(raw_text);
                let evaluation = Self::evaluate_callout_buffer(&self.pending_callout_marker_buffer);
                callout_decision = Some(Self::apply_callout_buffer_evaluation(
                    state,
                    evaluation,
                    &self.pending_callout_marker_buffer,
                ));
            } else {
                *state = CalloutState::None;
            }
        }

        if let Some(decision) = callout_decision {
            match decision {
                CalloutDecision::RenderHeader {
                    kind,
                    label,
                    label_override,
                    fold,
                    trailing,
                    suppress_paragraph_break,
                } => {
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    if matches!(self.config.callout_style.style, CalloutStyle::Pretty) {
                        self.content_indent = 0;
                        self.heading_indent = 0;
                    }
                    self.note_paragraph_content();
                    self.render_callout_header(kind, &label, label_override.as_deref(), fold);
                    if suppress_paragraph_break {
                        self.suppress_next_paragraph_break = true;
                    }
                    if let Some(trailing) = trailing {
                        if !trailing.trim().is_empty() {
                            self.note_paragraph_content();
                        }
                        self.process_text_with_wrapping_and_formatting(&trailing)?;
                        self.commit_pending_heading_placeholder_if_content();
                        self.suppress_next_soft_break = false;
                    } else {
                        self.suppress_next_soft_break = true;
                    }
                    return Ok(());
                }
                CalloutDecision::AwaitLabelOverride => {
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    if matches!(self.config.callout_style.style, CalloutStyle::Pretty) {
                        self.content_indent = 0;
                        self.heading_indent = 0;
                    }
                    self.pending_callout_label_override = true;
                    self.pending_callout_label_buffer.clear();
                    return Ok(());
                }
                CalloutDecision::FlushBuffer(buffer) => {
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    if !buffer.trim().is_empty() {
                        self.note_paragraph_content();
                    }
                    self.process_text_with_wrapping_and_formatting(&buffer)?;
                    self.commit_pending_heading_placeholder_if_content();
                    return Ok(());
                }
                CalloutDecision::Pending => {
                    return Ok(());
                }
            }
        }

        self.maybe_render_callout_header();
        if self.pending_task_marker && !self.list_stack.is_empty() {
            if self.pending_task_marker_buffer.is_empty() && !raw_text.starts_with('[') {
                self.pending_task_marker = false;
                self.pending_task_marker_buffer.clear();
            } else {
                self.pending_task_marker_buffer.push_str(raw_text);
                if self.pending_task_marker_buffer.chars().count() < 4 {
                    return Ok(());
                }

                self.pending_task_marker = false;
                let buffer = std::mem::take(&mut self.pending_task_marker_buffer);
                if let Some((marker, remainder)) = self.split_custom_task_marker_prefix(&buffer) {
                    self.note_paragraph_content();
                    self.strip_bullet_for_checkbox_item();
                    let rendered_marker = if self.config.pretty_checkbox.is_some() {
                        let state = marker.chars().nth(1).unwrap_or(' ');
                        self.styled_checkbox_marker(state)
                    } else {
                        let style = create_style(self.theme, ThemeElement::ListMarker);
                        style.apply(marker, self.config.no_colors)
                    };
                    self.output.push_str(&rendered_marker);
                    if let Some(list_state) = self.list_stack.last_mut() {
                        list_state.current_item_marker_end = Some(self.output.len());
                    }
                    if !remainder.is_empty() {
                        self.process_text_with_wrapping_and_formatting(remainder)?;
                    }
                } else {
                    // Process text with wrapping and formatting
                    if !buffer.trim().is_empty() {
                        self.note_paragraph_content();
                    }
                    self.process_text_with_wrapping_and_formatting(&buffer)?;
                }
                self.commit_pending_heading_placeholder_if_content();
                return Ok(());
            }
        }

        // Process text with wrapping and formatting
        if !raw_text.trim().is_empty() {
            self.note_paragraph_content();
        }
        self.process_text_with_wrapping_and_formatting(raw_text)?;
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
