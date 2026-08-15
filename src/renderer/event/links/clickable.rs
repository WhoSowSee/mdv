use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn process_clickable_text_with_wrapping(
        &mut self,
        text: &str,
        url: &str,
        force_underline: bool,
    ) -> Result<()> {
        self.process_wrapped_inline_fragments(text, |renderer, fragment| {
            let formatted = renderer.apply_formatting(fragment);
            let clickable = renderer.make_clickable_link(&formatted, url);

            if force_underline && !renderer.config.no_colors {
                format!("\x1b[4m{}\x1b[0m", clickable)
            } else {
                clickable
            }
        })
    }

    /// Make a text line clickable by wrapping it in terminal hyperlink escape sequences
    pub(in crate::renderer::event) fn make_clickable_link(&self, text: &str, url: &str) -> String {
        if self.config.no_colors {
            // If colors are disabled, don't add hyperlink sequences
            return text.to_string();
        }

        // Use OSC 8 hyperlink escape sequence to make text clickable
        // Format: \e]8;;URL\e\\TEXT\e]8;;\e\\
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
    }

    /// Create a clickable wrapped URL where each part opens the full original URL
    pub(in crate::renderer::event) fn make_clickable_wrapped_url(
        &self,
        original_url: &str,
        styled_wrapped_url: &str,
    ) -> String {
        if self.config.no_colors {
            return styled_wrapped_url.to_string();
        }

        // Split the wrapped URL by newlines and make each part clickable
        let lines: Vec<&str> = styled_wrapped_url.split('\n').collect();
        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }

            // Get clean text to check if line is empty
            let clean_line = crate::utils::strip_ansi(line);
            if !clean_line.trim().is_empty() {
                // Apply link styling to clean text first
                let style = create_style(self.theme, crate::theme::ThemeElement::Link);
                let styled_clean_line = style.apply(&clean_line, self.config.no_colors);
                // Then make the styled text clickable
                let clickable_line = self.make_clickable_link(&styled_clean_line, original_url);
                result.push_str(&clickable_line);
            } else {
                result.push_str(line);
            }
        }

        result
    }
    /// Ensure the last visual line does not exceed the terminal width.
    /// If it does, break the line at the last space and add proper indentation/prefixes.
    pub(in crate::renderer::event) fn enforce_width_on_current_line(&mut self) {
        let terminal_width = self.effective_text_width();
        let start = self.output.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let current_line_raw = &self.output[start..];
        let clean = crate::utils::strip_ansi(current_line_raw);
        let width = crate::utils::display_width(&clean);

        if width <= terminal_width {
            return;
        }

        // Find last space to break at; avoid breaking at the very first
        // leading space (indentation) which would produce a blank line.
        if let Some(space_rel_idx) = current_line_raw.rfind(' ') {
            if space_rel_idx == 0 {
                return;
            }
            // Build indentation for continuation line
            let indent = self.current_line_prefix();

            // Replace the space with a newline + indent
            let insert = format!("\n{}", indent);
            let abs_idx = start + space_rel_idx;
            self.output.replace_range(abs_idx..abs_idx + 1, &insert);
        }
    }
}
