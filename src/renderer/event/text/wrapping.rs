use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn process_text_units_with_wrapping(
        &mut self,
        text: &str,
        highlighted: bool,
    ) -> Result<()> {
        let effective_width = self.effective_text_width();
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters) while preserving formatting
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self
                .split_text_into_words_styled(text, self.word_wrap_content_width(effective_width)),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![Cow::Borrowed(text)],
        };

        for (i, unit) in units.iter().enumerate() {
            let current_line = self
                .output
                .rsplit_once('\n')
                .map_or(self.output.as_str(), |(_, line)| line);
            let current_line_width = crate::utils::display_width_ansi(current_line);
            if unit.trim().is_empty() && (i > 0 || self.formatting_stack.is_empty()) {
                let space_width = crate::utils::display_width(unit);
                if current_line_width + space_width > effective_width {
                    self.push_newline_with_context();
                } else {
                    let formatted_unit = if highlighted {
                        self.apply_formatting_with_highlight(unit, true)
                    } else {
                        Cow::Borrowed(unit.as_ref())
                    };
                    self.output.push_str(&formatted_unit);
                }
                continue;
            }

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
                && Self::line_has_visible_text(&crate::utils::strip_ansi(current_line))
                && wrap_mode != crate::utils::WrapMode::None
            {
                self.push_newline_with_context();
            }

            let formatted_unit = self.apply_formatting_with_highlight(unit, highlighted);

            // Add content indentation for new lines if needed
            // But don't add it if we're continuing text on the same line (like after inline links)
            let should_add_indent = (self.output.ends_with('\n') || self.output.is_empty())
                && !formatted_unit.trim().is_empty();

            if should_add_indent {
                self.push_indent_for_line_start();
            }

            self.output.push_str(&formatted_unit);
        }

        Ok(())
    }

    pub(super) fn split_text_into_words_styled<'t>(
        &self,
        text: &'t str,
        max_width: usize,
    ) -> Vec<Cow<'t, str>> {
        let mut words = Vec::new();
        let mut start = 0;
        let mut in_whitespace = false;
        for (index, ch) in text.char_indices() {
            let whitespace = ch.is_whitespace();
            if whitespace != in_whitespace && index > start {
                self.push_word_unit(&mut words, &text[start..index], max_width);
                start = index;
            }
            in_whitespace = whitespace;
        }
        if start < text.len() {
            self.push_word_unit(&mut words, &text[start..], max_width);
        }
        words
    }

    fn push_word_unit<'t>(&self, words: &mut Vec<Cow<'t, str>>, unit: &'t str, max_width: usize) {
        if unit.trim().is_empty() || crate::utils::display_width(unit) <= max_width {
            words.push(Cow::Borrowed(unit));
        } else {
            words.extend(
                crate::utils::wrap_text_with_mode(
                    unit,
                    max_width,
                    crate::utils::WrapMode::Character,
                )
                .split('\n')
                .map(|part| Cow::Owned(part.to_string())),
            );
        }
    }

    pub(super) fn word_wrap_content_width(&self, effective_width: usize) -> usize {
        effective_width
            .saturating_sub(self.compute_line_start_context_width())
            .max(1)
    }

    pub(super) fn split_text_into_characters_styled<'t>(&self, text: &'t str) -> Vec<Cow<'t, str>> {
        text.char_indices()
            .map(|(index, ch)| Cow::Borrowed(&text[index..index + ch.len_utf8()]))
            .collect()
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
        let should_wrap = self.should_wrap_inline_text();

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
            crate::utils::WrapMode::None => vec![Cow::Borrowed(text)],
        };

        let mut current_fragment = String::new();
        let initial_line = if let Some(last_newline) = self.output.rfind('\n') {
            &self.output[last_newline + 1..]
        } else {
            self.output.as_str()
        };
        let mut fragment_start_line_width = crate::utils::display_width_ansi(initial_line);

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

                current_fragment = unit.to_string();
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
