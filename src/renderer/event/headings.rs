use super::{EventRenderer, HeadingLevel, Result, ThemeElement, create_style};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_header_start(&mut self, level: HeadingLevel) -> Result<()> {
        self.finalize_pending_heading_placeholder();

        // Calculate indentation depending on layout and smart-indent flag
        if matches!(self.config.heading_layout, crate::cli::HeadingLayout::Level)
            && self.config.smart_indent
        {
            if let Some(&planned_indent) = self.smart_level_indents.get(&level) {
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
        // Update trackers after computing indentation
        self.current_heading_level = Some(level);
        self.last_header_level = level;

        self.trim_trailing_blank_lines();
        self.ensure_contextual_blank_line();

        if self.has_trailing_blank_line() {
            self.normalize_trailing_blank_line();
        }

        // Add heading indentation
        if self.heading_indent > 0 {
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
        }

        // Apply header styling to the last line(s) in output
        // This is a simplified approach - we style the header after it's been added
        let style = create_style(self.theme, element);
        let indent_str = " ".repeat(self.heading_indent);

        // Find the last header content (everything after the last double newline)
        if let Some(last_newline_pos) = self.output.rfind("\n\n") {
            let (before, after) = self.output.split_at(last_newline_pos + 2);
            let header_text = after.trim();
            if !header_text.is_empty() {
                // Remove the existing indentation from header text to avoid double indentation
                let clean_header_text = if header_text.starts_with(&indent_str) {
                    &header_text[indent_str.len()..]
                } else {
                    header_text
                };

                // Wrap header text if needed
                let wrapped_header = if !self.config.is_text_wrapping_enabled() {
                    clean_header_text.to_string()
                } else {
                    self.wrap_text_for_output(clean_header_text)
                };

                // Optionally center each line depending on layout
                let final_header = match self.config.heading_layout {
                    crate::cli::HeadingLayout::Center => {
                        let terminal_width = self.config.get_terminal_width();
                        let centered = wrapped_header
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
                            .join("\n");
                        centered
                    }
                    _ => {
                        let styled_header = style.apply(&wrapped_header, self.config.no_colors);
                        if self.heading_indent > 0 {
                            format!("{}{}", indent_str, styled_header)
                        } else {
                            styled_header
                        }
                    }
                };
                self.output = format!("{}{}", before, final_header);
            }
        } else {
            // Header is at the beginning, style the entire output so far
            let header_text = self.output.trim();
            if !header_text.is_empty() {
                // Remove the existing indentation from header text to avoid double indentation
                let clean_header_text = if header_text.starts_with(&indent_str) {
                    &header_text[indent_str.len()..]
                } else {
                    header_text
                };

                let wrapped_header = if !self.config.is_text_wrapping_enabled() {
                    clean_header_text.to_string()
                } else {
                    self.wrap_text_for_output(clean_header_text)
                };

                let final_header = match self.config.heading_layout {
                    crate::cli::HeadingLayout::Center => {
                        let terminal_width = self.config.get_terminal_width();
                        let centered = wrapped_header
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
                            .join("\n");
                        centered
                    }
                    _ => {
                        let styled_header = style.apply(&wrapped_header, self.config.no_colors);
                        if self.heading_indent > 0 {
                            format!("{}{}", indent_str, styled_header)
                        } else {
                            styled_header
                        }
                    }
                };
                self.output = final_header;
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
            if crate::utils::strip_ansi(slice).trim().is_empty() {
                if start <= end && end <= self.output.len() {
                    self.output.replace_range(start..end, "");
                }
            }
        }
    }
}
