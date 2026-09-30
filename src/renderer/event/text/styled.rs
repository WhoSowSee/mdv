use super::*;

impl<'a> EventRenderer<'a> {
    fn push_strikethrough_fragment(&mut self, fragment: &str, highlighted: bool) {
        let content = if highlighted {
            fragment
        } else {
            fragment.trim_end()
        };
        let formatted = self.apply_formatting_with_highlight(content, highlighted);
        self.output.push_str(&formatted);
        self.output.push_str(&fragment[content.len()..]);
    }

    pub(in crate::renderer::event) fn process_underlined_text_with_wrapping(
        &mut self,
        text: &str,
    ) -> Result<()> {
        self.process_wrapped_inline_fragments(text, |renderer, fragment| {
            if renderer.output_style.is_disabled() {
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
        let effective_width = self.effective_text_width();

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters)
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self
                .split_text_into_words_styled(text, self.word_wrap_content_width(effective_width)),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![Cow::Borrowed(text)],
        };

        // Process units in groups - each group becomes one continuous struck fragment
        let mut current_fragment = String::new();

        // Initial line width (without ANSI)
        let initial_line = if let Some(last_newline) = self.output.rfind('\n') {
            &self.output[last_newline + 1..]
        } else {
            self.output.as_str()
        };
        let mut fragment_start_line_width = crate::utils::display_width_ansi(initial_line);

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
                    self.push_strikethrough_fragment(&current_fragment, highlighted);

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
                self.push_strikethrough_fragment(&current_fragment, highlighted);

                self.push_newline_with_context();
                fragment_start_line_width = self.compute_line_start_context_width();

                current_fragment = unit.to_string();
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
            self.push_strikethrough_fragment(&current_fragment, highlighted);
        }

        Ok(())
    }
    pub(super) fn process_regular_text(
        &mut self,
        text: &str,
        should_wrap: bool,
        highlighted: bool,
    ) -> Result<()> {
        if should_wrap {
            self.process_text_units_with_wrapping(text, highlighted)?;
        } else {
            let formatted = self.apply_formatting_with_highlight(text, highlighted);
            if (self.output.ends_with('\n') || self.output.is_empty())
                && !formatted.trim().is_empty()
            {
                self.push_indent_for_line_start();
            }
            self.output.push_str(&formatted);
        }
        Ok(())
    }
}
