use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn process_styled_text_with_wrapping(
        &mut self,
        text: &str,
        highlighted: bool,
    ) -> Result<()> {
        let terminal_width = self.effective_text_width();

        // The effective width is the full terminal width since current_line_width
        // already includes any indentation that's been added to the current line
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters) while preserving formatting
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self
                .split_text_into_words_styled(text, self.word_wrap_content_width(effective_width)),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process each unit individually with formatting
        for (i, unit) in units.iter().enumerate() {
            if unit.trim().is_empty() && i > 0 {
                // Handle whitespace between units
                let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                } else {
                    crate::utils::strip_ansi(&self.output)
                };
                let current_line_width = crate::utils::display_width(&current_line_clean);
                let space_width = crate::utils::display_width(unit);
                if current_line_width + space_width > effective_width {
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
                let reference_index = if matches!(self.config.link_style, LinkStyle::InlineTable) {
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

            let would_exceed = current_line_width + unit_width + additional_width > effective_width;

            // Force line break if needed (but not for the first unit on a line)
            if would_exceed
                && current_line_width > 0
                && Self::line_has_visible_text(&current_line_clean)
                && wrap_mode != crate::utils::WrapMode::None
            {
                self.push_newline_with_context();
            }

            // Apply formatting and add to output
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

            // Don't add indentation if we're continuing on the same line OR
            // if we just processed a link (which may have wrapped URLs)
            if should_add_indent && !after_inline_content {
                self.push_indent_for_line_start();
            }

            self.output.push_str(&formatted_unit);
        }

        Ok(())
    }

    /// Split text into words for word-based wrapping (for styled text)
    pub(super) fn split_text_into_words_styled(&self, text: &str, max_width: usize) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut in_whitespace = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !in_whitespace && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
                in_whitespace = true;
            } else {
                if in_whitespace && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
                in_whitespace = false;
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
            .into_iter()
            .flat_map(|word| self.split_oversized_word_unit(word, max_width))
            .collect()
    }

    pub(super) fn split_oversized_word_unit(&self, unit: String, max_width: usize) -> Vec<String> {
        if unit.trim().is_empty() || crate::utils::display_width(&unit) <= max_width {
            return vec![unit];
        }

        crate::utils::wrap_text_with_mode(&unit, max_width, crate::utils::WrapMode::Character)
            .split('\n')
            .map(str::to_string)
            .collect()
    }

    pub(super) fn word_wrap_content_width(&self, effective_width: usize) -> usize {
        effective_width
            .saturating_sub(self.compute_line_start_context_width())
            .max(1)
    }

    /// Split text into characters for character-based wrapping (for styled text)
    pub(super) fn split_text_into_characters_styled(&self, text: &str) -> Vec<String> {
        text.chars().map(|c| c.to_string()).collect()
    }

    /// Calculate proper indentation for list content continuation lines
    pub(in crate::renderer::event) fn calculate_list_content_indent(&self) -> usize {
        let mut total_indent = 0;

        // Add heading content indentation
        total_indent += self.content_indent;

        // Add list nesting indentation (2 spaces per level)
        let indent_level = self.list_stack.len().saturating_sub(1);
        total_indent += indent_level * 2;

        // Add space for the list marker
        if let Some(list_state) = self.list_stack.last() {
            let marker_width = if list_state.is_ordered {
                // For ordered lists: "1. ", "2. ", etc. - typically 3 characters
                3
            } else {
                // For unordered lists: "- " - 2 characters
                2
            };
            total_indent += marker_width;
        }

        total_indent
    }

    pub(super) fn push_wrapped_inline_fragment<F>(
        &mut self,
        fragment: &str,
        render_fragment: &mut F,
    ) where
        F: FnMut(&Self, &str) -> String,
    {
        let content = fragment.trim_end();
        let trailing_whitespace = &fragment[content.len()..];

        if !content.is_empty() {
            let rendered = render_fragment(self, content);
            self.output.push_str(&rendered);
        }
        self.output.push_str(trailing_whitespace);
    }

    pub(in crate::renderer::event) fn process_wrapped_inline_fragments<F>(
        &mut self,
        text: &str,
        mut render_fragment: F,
    ) -> Result<()>
    where
        F: FnMut(&Self, &str) -> String,
    {
        let should_wrap = self.config.is_text_wrapping_enabled();

        if !should_wrap {
            let rendered = render_fragment(self, text);
            self.output.push_str(&rendered);
            return Ok(());
        }

        let effective_width = self.effective_text_width();
        let wrap_mode = self.config.text_wrap_mode();
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self
                .split_text_into_words_styled(text, self.word_wrap_content_width(effective_width)),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        let mut current_fragment = String::new();
        let initial_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };
        let mut fragment_start_line_width = crate::utils::display_width(&initial_line_clean);

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

            if is_ws && i > 0 {
                if would_exceed && !current_fragment.trim().is_empty() {
                    self.push_wrapped_inline_fragment(&current_fragment, &mut render_fragment);
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
                self.push_wrapped_inline_fragment(&current_fragment, &mut render_fragment);

                if wrap_mode != crate::utils::WrapMode::None {
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment = unit.clone();
            } else {
                if would_exceed {
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment.push_str(unit);
            }
        }

        if !current_fragment.is_empty() {
            self.push_wrapped_inline_fragment(&current_fragment, &mut render_fragment);
        }

        Ok(())
    }
}
