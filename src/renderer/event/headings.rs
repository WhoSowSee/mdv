use super::core::CalloutState;
use super::{EventRenderer, HeadingLevel, Result, ThemeElement, create_style};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_header_start(&mut self, level: HeadingLevel) -> Result<()> {
        self.finalize_pending_heading_placeholder();

        // Calculate indentation depending on layout and smart-indent flag
        if matches!(self.config.heading_layout, crate::cli::HeadingLayout::Level)
            && self.config.smart_indent
        {
            let planned_indent = if self.blockquote_level > 0 {
                self.active_blockquote_smart_indents
                    .last()
                    .and_then(|map| map.get(&level))
                    .copied()
            } else {
                self.smart_level_indents.get(&level).copied()
            };

            if let Some(planned_indent) = planned_indent {
                self.heading_indent = planned_indent;
                self.content_indent = planned_indent + 1;
            } else {
                // Fallback: use default mapping if level was not precomputed.
                self.heading_indent = self.get_heading_indent(level);
                self.content_indent = self.get_content_indent(level);
            }
        } else {
            // Non-smart or non-level layout: use the standard mapping
            self.heading_indent = self.get_heading_indent(level);
            self.content_indent = self.get_content_indent(level);
        }
        if let Some(callout_info) =
            self.callout_stack
                .iter_mut()
                .rev()
                .find_map(|state| match state {
                    CalloutState::Active(info) => Some(info),
                    _ => None,
                })
        {
            let base = callout_info
                .min_heading_indent
                .map(|current| current.min(self.heading_indent))
                .unwrap_or(self.heading_indent);
            callout_info.min_heading_indent = Some(base);
            if base > 0 {
                self.heading_indent = self.heading_indent.saturating_sub(base);
                self.content_indent = self.content_indent.saturating_sub(base);
            }
        }
        // Update trackers after computing indentation
        self.current_heading_level = Some(level);
        self.last_header_level = level;

        self.trim_trailing_blank_lines();
        let use_heading_prefix = self.blockquote_level > 0
            && matches!(
                self.config.callout_style.style,
                crate::cli::CalloutStyle::Simple
            )
            && self
                .callout_stack
                .iter()
                .any(|state| matches!(state, CalloutState::Active(_)));

        if use_heading_prefix {
            let mut prefix = self.render_blockquote_prefix();
            if !self.list_stack.is_empty() {
                let list_indent = self
                    .calculate_list_content_indent()
                    .saturating_sub(self.content_indent);
                if list_indent > 0 {
                    prefix.push_str(&" ".repeat(list_indent));
                }
            }
            self.ensure_contextual_blank_line_with_prefix(&prefix);
        } else {
            self.ensure_contextual_blank_line();
        }

        if self.has_trailing_blank_line() && !use_heading_prefix {
            self.normalize_trailing_blank_line();
        }

        if self.blockquote_level > 0 {
            let prefix = self.render_blockquote_prefix();
            self.output.push_str(&prefix);
            if self.heading_indent > 0 {
                self.output.push_str(&" ".repeat(self.heading_indent));
            }
            if !self.list_stack.is_empty() {
                let list_indent = self
                    .calculate_list_content_indent()
                    .saturating_sub(self.content_indent);
                if list_indent > 0 {
                    self.output.push_str(&" ".repeat(list_indent));
                }
            }
        } else if self.heading_indent > 0 {
            // Add heading indentation
            self.output.push_str(&" ".repeat(self.heading_indent));
        }

        self.current_heading_start = Some(self.output.len());

        Ok(())
    }

    /// Calculate indentation for a heading level depending on layout mode
    /// Level: H1:0, H2:1, ..., H6:5; Center/Flat/None: 0
    fn get_heading_indent(&self, level: HeadingLevel) -> usize {
        use crate::cli::HeadingLayout;
        match self.config.heading_layout {
            HeadingLayout::Level => match level {
                HeadingLevel::H1 => 0,
                HeadingLevel::H2 => 1,
                HeadingLevel::H3 => 2,
                HeadingLevel::H4 => 3,
                HeadingLevel::H5 => 4,
                HeadingLevel::H6 => 5,
            },
            HeadingLayout::Center | HeadingLayout::Flat | HeadingLayout::None => 0,
        }
    }

    /// Calculate indentation for content under a heading level
    /// Level: +1 relative to heading; Center: 0; Flat: 1; None: 0
    fn get_content_indent(&self, level: HeadingLevel) -> usize {
        use crate::cli::HeadingLayout;
        match self.config.heading_layout {
            HeadingLayout::Level => match level {
                HeadingLevel::H1 => 1,
                HeadingLevel::H2 => 2,
                HeadingLevel::H3 => 3,
                HeadingLevel::H4 => 4,
                HeadingLevel::H5 => 5,
                HeadingLevel::H6 => 6,
            },
            HeadingLayout::Center => 0,
            HeadingLayout::Flat => 1,
            HeadingLayout::None => 0,
        }
    }

    pub(super) fn handle_header_end(&mut self, level: HeadingLevel) -> Result<()> {
        let element = match level {
            HeadingLevel::H1 => ThemeElement::H1,
            HeadingLevel::H2 => ThemeElement::H2,
            HeadingLevel::H3 => ThemeElement::H3,
            HeadingLevel::H4 => ThemeElement::H4,
            HeadingLevel::H5 => ThemeElement::H5,
            HeadingLevel::H6 => ThemeElement::H6,
        };

        if let Some(start) = self.current_heading_start.take() {
            let is_empty_heading = {
                let slice = if start <= self.output.len() {
                    &self.output[start..]
                } else {
                    ""
                };
                crate::utils::strip_ansi(slice).trim().is_empty()
            };

            if is_empty_heading {
                let marker_count = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let placeholder = "#".repeat(marker_count);

                if self.output.len() == start {
                    self.output.push_str(&placeholder);
                } else {
                    self.output.insert_str(start, &placeholder);
                }

                if self.config.show_empty_elements {
                    self.pending_heading_placeholder = None;
                } else {
                    self.pending_heading_placeholder = Some((start, placeholder.len()));
                }
            } else {
                self.pending_heading_placeholder = None;
            }

            // Apply header styling based on the exact heading span.
            let line_start = self.output[..start]
                .rfind('\n')
                .map(|idx| idx + 1)
                .unwrap_or(0);
            let line_prefix = self.output[line_start..start].to_string();
            let before = self.output[..line_start].to_string();
            let header_text = self.output[start..].to_string();

            let clean_header_text = header_text.trim();
            if !clean_header_text.is_empty() {
                let wrapped_header = if !self.config.is_text_wrapping_enabled() {
                    clean_header_text.to_string()
                } else {
                    self.wrap_text_for_output(clean_header_text)
                };

                let style = create_style(self.theme, element);
                let styled_header = match self.config.heading_layout {
                    crate::cli::HeadingLayout::Center => {
                        let terminal_width = self.effective_text_width();
                        wrapped_header
                            .lines()
                            .map(|line| {
                                let clean = crate::utils::strip_ansi(line);
                                let line_width = crate::utils::display_width(&clean);
                                let pad = if terminal_width > line_width {
                                    (terminal_width - line_width) / 2
                                } else {
                                    0
                                };
                                let styled = style.apply(line, self.config.no_colors);
                                format!("{}{}", " ".repeat(pad), styled)
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                    _ => style.apply(&wrapped_header, self.config.no_colors),
                };

                let final_header = styled_header
                    .lines()
                    .map(|line| format!("{}{}", line_prefix, line))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.output = format!("{}{}", before, final_header);
            } else {
                self.output = format!("{}{}", before, line_prefix);
            }
        }

        self.output.push('\n');
        if matches!(
            self.config.heading_layout,
            crate::cli::HeadingLayout::Center
        ) {
            // Centered headings look better with an empty line before content.
            self.output.push('\n');
        }

        // Keep the current heading level for subsequent content indentation
        // Don't reset it here as content under this heading should have the same indent

        Ok(())
    }

    pub(super) fn commit_pending_heading_placeholder_if_content(&mut self) {
        if let Some((start, len)) = self.pending_heading_placeholder {
            let end = start.saturating_add(len);
            if end <= self.output.len() {
                let slice = &self.output[end..];
                if !crate::utils::strip_ansi(slice).trim().is_empty() {
                    self.pending_heading_placeholder = None;
                }
            }
        }
    }

    pub(super) fn finalize_pending_heading_placeholder(&mut self) {
        if let Some((start, len)) = self.pending_heading_placeholder.take() {
            let end = start.saturating_add(len);
            let end = end.min(self.output.len());
            let slice = if end <= self.output.len() {
                &self.output[end..]
            } else {
                ""
            };
            if crate::utils::strip_ansi(slice).trim().is_empty()
                && start <= end
                && end <= self.output.len()
            {
                self.output.replace_range(start..end, "");
            }
        }
    }
}
