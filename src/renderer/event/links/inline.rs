use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn finish_inline_link(&mut self) -> Result<()> {
        // For Inline mode, process link text with normal wrapping, then add URL
        let current_link_text = self.current_link_text.clone();
        let url = self
            .link_references
            .get(&format!("current_{}", self.link_counter))
            .cloned();

        if let Some(url) = url {
            // Check if we're in a table cell
            if let Some(ref mut table) = self.table_state {
                // custom_styling keeps the underline scoped through table wrapping.
                push_underlined_table_link(table, &current_link_text, self.config.no_colors);
                let url_part = format!("({})", url);
                let style = create_style(self.theme, ThemeElement::Link);
                let styled_url = style.apply(&url_part, self.config.no_colors);

                if matches!(self.config.link_truncation, LinkTruncationStyle::TableCut) {
                    let target = if table.in_header {
                        TableInlineUrlTarget::Header {
                            column_index: table.current_row.len(),
                        }
                    } else {
                        TableInlineUrlTarget::Row {
                            row_index: table.rows.len(),
                            column_index: table.current_row.len(),
                        }
                    };

                    table.inline_url_segments.push(TableInlineUrlSegment {
                        target,
                        url: url.clone(),
                        url_part: url_part.clone(),
                    });
                }

                table.current_cell.push_str(&styled_url);
            } else {
                // Process link text with underline formatting and normal wrapping logic
                self.process_underlined_text_with_wrapping(&current_link_text)?;
                // Safety: if last visual line overflowed by a single dangling character, fix it
                self.enforce_width_on_current_line();

                // Now add the URL part
                let url_part = format!("({})", url);

                // Check if URL needs wrapping or truncation
                let should_wrap = self.config.is_text_wrapping_enabled();

                if should_wrap {
                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };

                    let terminal_width = self.effective_text_width();
                    let current_line_width = crate::utils::display_width(&current_line_clean);
                    let url_part_width = crate::utils::display_width(&url_part);

                    // Check truncation style for Inline mode
                    match self.config.link_truncation {
                        LinkTruncationStyle::Cut | LinkTruncationStyle::TableCut => {
                            // Precisely fit the URL display into the remaining space on the current line.
                            let available_width = terminal_width.saturating_sub(current_line_width);

                            if available_width >= url_part_width {
                                // URL fits entirely on the current line
                                let style = create_style(self.theme, ThemeElement::Link);
                                let styled_url = style.apply(&url_part, self.config.no_colors);
                                let clickable_url = self.make_clickable_link(&styled_url, &url);
                                self.output.push_str(&clickable_url);
                                self.enforce_width_on_current_line();
                            } else if available_width > 2 {
                                // Space available only for a truncated form inside parentheses
                                let available_for_url = available_width.saturating_sub(2); // -2 for parentheses
                                let truncated_display =
                                    self.truncate_url_with_ellipsis(&url, available_for_url);
                                let truncated_url_part = format!("({})", truncated_display);
                                let style = create_style(self.theme, ThemeElement::Link);
                                let styled_truncated =
                                    style.apply(&truncated_url_part, self.config.no_colors);
                                let clickable_truncated =
                                    self.make_clickable_link(&styled_truncated, &url);
                                self.output.push_str(&clickable_truncated);
                            } else {
                                // Not enough space left on this visual line – break and place URL at the start
                                // of the next line with proper indentation, then fit it there.
                                self.output.push('\n');
                                self.push_indent_for_line_start();

                                // Effective width for the new line considering indentation
                                let mut effective_width_for_url = terminal_width;
                                if self.content_indent > 0 {
                                    effective_width_for_url =
                                        effective_width_for_url.saturating_sub(self.content_indent);
                                }
                                if self.blockquote_level > 0 {
                                    let prefix_width = self.blockquote_level + 1; // │ + space
                                    effective_width_for_url =
                                        effective_width_for_url.saturating_sub(prefix_width);
                                }

                                let available_for_url = effective_width_for_url.saturating_sub(2);
                                let truncated_display =
                                    self.truncate_url_with_ellipsis(&url, available_for_url);
                                let truncated_url_part = format!("({})", truncated_display);
                                let style = create_style(self.theme, ThemeElement::Link);
                                let styled_truncated =
                                    style.apply(&truncated_url_part, self.config.no_colors);
                                let clickable_truncated =
                                    self.make_clickable_link(&styled_truncated, &url);
                                self.output.push_str(&clickable_truncated);
                            }
                        }
                        LinkTruncationStyle::None => {
                            // No truncation - make URL clickable even if it overflows
                            let style = create_style(self.theme, ThemeElement::Link);
                            let styled_url = style.apply(&url_part, self.config.no_colors);
                            let clickable_url = self.make_clickable_link(&styled_url, &url);
                            self.output.push_str(&clickable_url);
                        }
                        LinkTruncationStyle::Wrap => {
                            // Flexible wrapping: place as much as fits on the current line,
                            // then continue on the next line with proper indentation.
                            if current_line_width + url_part_width <= terminal_width {
                                // Fits entirely on the current line
                                let style = create_style(self.theme, ThemeElement::Link);
                                let styled_url = style.apply(&url_part, self.config.no_colors);
                                let clickable_url = self.make_clickable_link(&styled_url, &url);
                                self.output.push_str(&clickable_url);
                            } else {
                                // Split URL text into two parts: the remainder that fits on this line,
                                // and the rest that goes to the next line(s).
                                let mut taken = String::new();
                                let mut remaining = String::new();
                                let mut acc = 0usize;
                                for ch in url_part.chars() {
                                    let w = crate::utils::display_width(&ch.to_string());
                                    if acc + w <= terminal_width.saturating_sub(current_line_width)
                                    {
                                        taken.push(ch);
                                        acc += w;
                                    } else {
                                        remaining.push(ch);
                                    }
                                }

                                // Add the part that fits to the current line
                                if !taken.is_empty() {
                                    let style = create_style(self.theme, ThemeElement::Link);
                                    let styled_taken = style.apply(&taken, self.config.no_colors);
                                    let clickable_taken =
                                        self.make_clickable_link(&styled_taken, &url);
                                    self.output.push_str(&clickable_taken);
                                }

                                // If anything remains, break the line and render the rest with indentation
                                if !remaining.is_empty() {
                                    // New visual line for the rest of the URL
                                    self.push_newline_with_context();

                                    // Wrap the remaining part for subsequent lines
                                    let style = create_style(self.theme, ThemeElement::Link);
                                    let styled_remaining =
                                        style.apply(&remaining, self.config.no_colors);
                                    let wrapped_url =
                                        self.wrap_url_with_indentation(&styled_remaining);
                                    let clickable_wrapped =
                                        self.make_clickable_wrapped_url(&url, &wrapped_url);
                                    self.output.push_str(&clickable_wrapped);
                                    self.enforce_width_on_current_line();
                                }
                            }
                        }
                    }
                } else {
                    // No wrapping, but still ensure we do not exceed terminal width
                    match self.config.link_truncation {
                        LinkTruncationStyle::Cut | LinkTruncationStyle::TableCut => {
                            let terminal_width = self.effective_text_width();
                            let current_line_clean =
                                if let Some(last_newline) = self.output.rfind('\n') {
                                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                                } else {
                                    crate::utils::strip_ansi(&self.output)
                                };
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let available_width = terminal_width.saturating_sub(current_line_width);
                            let url_part_width = crate::utils::display_width(&url_part);

                            if available_width >= url_part_width {
                                let style = create_style(self.theme, ThemeElement::Link);
                                let styled_url = style.apply(&url_part, self.config.no_colors);
                                let clickable_url = self.make_clickable_link(&styled_url, &url);
                                self.output.push_str(&clickable_url);
                                self.enforce_width_on_current_line();
                            } else if available_width > 2 {
                                let available_for_url = available_width.saturating_sub(2);
                                let truncated_display =
                                    self.truncate_url_with_ellipsis(&url, available_for_url);
                                let truncated_url_part = format!("({})", truncated_display);
                                let style = create_style(self.theme, ThemeElement::Link);
                                let styled_truncated =
                                    style.apply(&truncated_url_part, self.config.no_colors);
                                let clickable_truncated =
                                    self.make_clickable_link(&styled_truncated, &url);
                                self.output.push_str(&clickable_truncated);
                                self.enforce_width_on_current_line();
                            } else {
                                // Not enough space even for parentheses; show minimal clickable marker if possible
                                if available_width > 0 {
                                    let style = create_style(self.theme, ThemeElement::Link);
                                    let marker = style.apply("…", self.config.no_colors);
                                    let clickable_marker = self.make_clickable_link(&marker, &url);
                                    self.output.push_str(&clickable_marker);
                                }
                            }
                        }
                        _ => {
                            // Just add clickable URL without wrapping or truncation
                            let style = create_style(self.theme, ThemeElement::Link);
                            let styled_url = style.apply(&url_part, self.config.no_colors);
                            let clickable_url = self.make_clickable_link(&styled_url, &url);
                            self.output.push_str(&clickable_url);
                            self.enforce_width_on_current_line();
                        }
                    }
                }
            }
        }
        self.in_link = false;
        self.current_link_text.clear();

        Ok(())
    }
}
