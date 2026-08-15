use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn process_underlined_text_with_wrapping(
        &mut self,
        text: &str,
    ) -> Result<()> {
        self.process_wrapped_inline_fragments(text, |renderer, fragment| {
            if renderer.config.no_colors {
                fragment.to_string()
            } else {
                format!("\x1b[4m{}\x1b[0m", fragment)
            }
        })
    }

    /// Process text with strikethrough formatting applied as a continuous run (includes spaces)
    pub(super) fn process_strikethrough_text_with_wrapping(
        &mut self,
        text: &str,
        highlighted: bool,
    ) -> Result<()> {
        let should_wrap = self.config.is_text_wrapping_enabled();

        if !should_wrap {
            // No wrapping - apply full formatting (including strikethrough) to entire text
            let formatted_text = self.apply_formatting_with_highlight(text, highlighted);
            self.output.push_str(&formatted_text);
            return Ok(());
        }

        let terminal_width = self.effective_text_width();
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters)
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self
                .split_text_into_words_styled(text, self.word_wrap_content_width(effective_width)),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process units in groups - each group becomes one continuous struck fragment
        let mut current_fragment = String::new();

        // Initial line width (without ANSI)
        let initial_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };
        let mut fragment_start_line_width = crate::utils::display_width(&initial_line_clean);

        // If little space left on the current line, move to a new one before adding any struck text
        if effective_width.saturating_sub(fragment_start_line_width) <= 1 && !text.trim().is_empty()
        {
            self.push_newline_with_context();
            fragment_start_line_width = self.compute_line_start_context_width();
        }

        for (i, unit) in units.iter().enumerate() {
            let is_ws = unit.trim().is_empty();
            let unit_width = crate::utils::display_width(unit);
            let current_fragment_width = crate::utils::display_width(&current_fragment);
            let would_exceed =
                fragment_start_line_width + current_fragment_width + unit_width > effective_width;

            // Whitespace handling: keep inside fragment unless it would overflow the line
            if is_ws && i > 0 {
                if would_exceed && !current_fragment.trim().is_empty() {
                    // Flush current fragment and break line; drop whitespace at new line start
                    let fragment_to_format = current_fragment.trim_end();
                    let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                    // Apply full formatting (includes strike) to the fragment; keep spaces highlighted when needed
                    let formatted_fragment = if highlighted {
                        self.apply_formatting_with_highlight(&current_fragment, true)
                    } else {
                        format!(
                            "{}{}",
                            self.apply_formatting(fragment_to_format),
                            trailing_spaces
                        )
                    };
                    self.output.push_str(&formatted_fragment);

                    // Start new visual line with correct context indentation
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();

                    current_fragment.clear();
                    continue;
                } else {
                    current_fragment.push_str(unit);
                    continue;
                }
            }

            if would_exceed && !current_fragment.trim().is_empty() {
                // Break: output current fragment first
                let fragment_to_format = current_fragment.trim_end();
                let trailing_spaces = &current_fragment[fragment_to_format.len()..];
                let formatted_fragment = if highlighted {
                    self.apply_formatting_with_highlight(&current_fragment, true)
                } else {
                    format!(
                        "{}{}",
                        self.apply_formatting(fragment_to_format),
                        trailing_spaces
                    )
                };
                self.output.push_str(&formatted_fragment);

                if wrap_mode != crate::utils::WrapMode::None {
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment = unit.clone();
            } else {
                if would_exceed {
                    // Nothing in fragment yet, but unit would exceed -> break line first
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment.push_str(unit);
            }
        }

        // Output remaining fragment if any
        if !current_fragment.is_empty() {
            let fragment_to_format = current_fragment.trim_end();
            let trailing_spaces = &current_fragment[fragment_to_format.len()..];
            let formatted_fragment = if highlighted {
                self.apply_formatting_with_highlight(&current_fragment, true)
            } else {
                format!(
                    "{}{}",
                    self.apply_formatting(fragment_to_format),
                    trailing_spaces
                )
            };
            self.output.push_str(&formatted_fragment);
        }

        Ok(())
    }
    pub(super) fn process_regular_text(
        &mut self,
        text: &str,
        should_wrap: bool,
        highlighted: bool,
    ) -> Result<()> {
        // Use the same word-by-word logic as styled text for consistent behavior
        if should_wrap {
            let terminal_width = self.effective_text_width();

            // Use full terminal width as effective width since current_line_width already includes indents
            let effective_width = terminal_width;

            // Determine wrap mode based on config
            let wrap_mode = self.config.text_wrap_mode();

            // Split text into wrappable units (words or characters)
            let units = match wrap_mode {
                crate::utils::WrapMode::Word => self.split_text_into_words_styled(
                    text,
                    self.word_wrap_content_width(effective_width),
                ),
                crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
                crate::utils::WrapMode::None => vec![text.to_string()],
            };

            // Process each unit individually
            for unit in units.iter() {
                if unit.trim().is_empty() {
                    // Handle whitespace cautiously: don't let a trailing space overflow the line
                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };
                    let current_line_width = crate::utils::display_width(&current_line_clean);
                    let space_width = crate::utils::display_width(unit);
                    if current_line_width + space_width > effective_width {
                        // Break visual line and skip adding whitespace at start of next line
                        self.push_newline_with_context();
                    } else {
                        let formatted_unit = if highlighted {
                            self.apply_formatting_with_highlight(unit, true)
                        } else {
                            unit.to_string()
                        };
                        self.output.push_str(&formatted_unit);
                    }
                    continue;
                }

                // Check if adding this unit would exceed line width
                let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                } else {
                    crate::utils::strip_ansi(&self.output)
                };

                let current_line_width = crate::utils::display_width(&current_line_clean);
                let unit_width = crate::utils::display_width(unit);

                // For InlineTable links, account for the reference number that will be added
                let additional_width = if self.in_link
                    && matches!(
                        self.config.link_style,
                        LinkStyle::InlineTable | LinkStyle::EndTable
                    ) {
                    // Calculate the width of the reference number like [1], [2], etc.
                    let reference_index =
                        if matches!(self.config.link_style, LinkStyle::InlineTable) {
                            match self.callout_stack.last() {
                                Some(CalloutState::Active(info)) => info.inline_link_counter,
                                _ => self.paragraph_link_counter,
                            }
                        } else {
                            self.paragraph_link_counter
                        };
                    let ref_num_str = format!("[{}]", reference_index);
                    crate::utils::display_width(&ref_num_str)
                } else {
                    0
                };

                let would_exceed =
                    current_line_width + unit_width + additional_width > effective_width;

                // Force line break if needed (but not for the first unit on a line)
                if would_exceed
                    && current_line_width > 0
                    && Self::line_has_visible_text(&current_line_clean)
                    && wrap_mode != crate::utils::WrapMode::None
                {
                    self.push_newline_with_context();
                }

                // Apply formatting (no-op for regular text) and add to output
                let formatted_unit = self.apply_formatting_with_highlight(unit, highlighted);

                // Add content indentation for new lines if needed
                // But don't add it if we're continuing text on the same line (like after inline links)
                let should_add_indent = (self.output.ends_with('\n') || self.output.is_empty())
                    && !formatted_unit.trim().is_empty();

                // Check if we're immediately after content that shouldn't get extra indentation
                let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                    let line_content = &self.output[last_newline + 1..];
                    // If the line has content (not just whitespace), we're continuing on the same line
                    !line_content.trim().is_empty()
                } else {
                    // No newlines, check if we have any content
                    !self.output.trim().is_empty()
                };

                if should_add_indent && !after_inline_content {
                    self.push_indent_for_line_start();
                }

                self.output.push_str(&formatted_unit);
            }
        } else {
            // No wrapping - still ensure correct indentation at visual line starts
            let final_text = self.apply_formatting_with_highlight(text, highlighted);

            // Add content indentation for new visual lines when appropriate
            if (self.output.ends_with('\n') || self.output.is_empty())
                && !final_text.trim().is_empty()
            {
                // If the current line (after the last newline) already contains
                // non-whitespace content, we are continuing on the same line and
                // must not add extra indentation.
                let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                    let line_content = &self.output[last_newline + 1..];
                    !line_content.trim().is_empty()
                } else {
                    !self.output.trim().is_empty()
                };

                if !after_inline_content {
                    self.push_indent_for_line_start();
                }
            }

            self.output.push_str(&final_text);
        }

        Ok(())
    }
}
