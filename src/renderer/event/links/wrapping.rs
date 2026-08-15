use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn wrap_url_with_reference(
        &self,
        url: &str,
        first_line_width: usize,
        continuation_width: usize,
        reference_width: usize,
    ) -> String {
        if crate::utils::display_width(url) <= first_line_width {
            return url.to_string();
        }

        let mut result = String::new();
        let mut current_line = String::new();
        let mut current_width = 0;
        let mut is_first_line = true;

        // Characters that are good breaking points in URLs
        let good_break_chars = ['/', '?', '&', '=', '-', '_', '.', ':', '#'];

        // Calculate the indent for continuation lines based on the actual reference width
        // This creates the exact same indentation as the reference part
        let continuation_indent = " ".repeat(reference_width);

        let chars: Vec<char> = url.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            let char_width = crate::utils::display_width(&ch.to_string());
            let max_width = if is_first_line {
                first_line_width
            } else {
                continuation_width.saturating_sub(reference_width)
            };

            // Check if adding this character would exceed the line width
            if current_width + char_width > max_width && !current_line.is_empty() {
                // Look for a good breaking point in the current line
                if let Some(break_pos) = self.find_url_break_point(&current_line, &good_break_chars)
                {
                    // Break at the good point
                    let (line_part, remaining) = current_line.split_at(break_pos);
                    result.push_str(line_part);
                    result.push('\n');

                    // Add indent for continuation line and start with remaining characters plus current character
                    result.push_str(&continuation_indent);
                    current_line = format!("{}{}", remaining, ch);
                    current_width = crate::utils::display_width(&current_line);
                } else {
                    // No good breaking point found, force break
                    result.push_str(&current_line);
                    result.push('\n');

                    // Add indent for continuation line and start with current character
                    result.push_str(&continuation_indent);
                    current_line = ch.to_string();
                    current_width = crate::utils::display_width(&current_line);
                }
                is_first_line = false;
            } else {
                // Add character to current line
                current_line.push(ch);
                current_width += char_width;
            }

            i += 1;
        }

        // Add remaining characters
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }

        result
    }

    /// Find the best breaking point in a URL segment
    pub(in crate::renderer::event) fn wrap_text_for_output(&self, text: &str) -> String {
        // Check if wrapping is disabled
        if !self.config.is_text_wrapping_enabled() {
            return text.to_string();
        }

        let terminal_width = self.effective_text_width();

        // Don't wrap if width is too small or text is very short
        if terminal_width < 20 || text.trim().len() < 10 {
            return text.to_string();
        }

        // Calculate effective width considering heading indentation and blockquote prefix
        let mut effective_width = terminal_width;

        // Account for heading indentation
        if self.content_indent > 0 {
            effective_width = effective_width.saturating_sub(self.content_indent);
        }

        // Account for blockquote prefix if in blockquote
        if self.current_indent > 0 {
            // Each level adds one │ character plus one space
            let prefix_width = self.blockquote_level + 1; // │ symbols + space
            effective_width = effective_width.saturating_sub(prefix_width);
        }

        // Only wrap if we have a reasonable width (minimum 10 characters for deep nesting)
        if effective_width < 10 {
            return text.to_string();
        }

        // Determine wrapping mode
        let wrap_mode = self.config.text_wrap_mode();

        // Use our text wrapping utility
        // For blockquotes, don't add indentation here - we'll add the │ prefix manually
        wrap_text_with_mode(text, effective_width, wrap_mode)
    }

    /// Wrap URL text with proper indentation for each line
    pub(in crate::renderer::event) fn wrap_url_with_indentation(&self, text: &str) -> String {
        let wrapped = self.wrap_text_for_output(text);

        // If the text wasn't actually wrapped (no newlines), return as is
        if !wrapped.contains('\n') {
            return wrapped;
        }

        // Split into lines and add indentation to continuation lines
        let lines: Vec<&str> = wrapped.split('\n').collect();
        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');
                let prefix = self.current_line_prefix();
                if !prefix.is_empty() {
                    result.push_str(&prefix);
                }
            }
            result.push_str(line);
        }

        result
    }

    pub(in crate::renderer::event) fn find_url_break_point(
        &self,
        line: &str,
        good_break_chars: &[char],
    ) -> Option<usize> {
        // Look for good breaking points from right to left (prefer breaking later)
        for (i, ch) in line.char_indices().rev() {
            if good_break_chars.contains(&ch) {
                // Break after the special character (not before)
                return Some(i + ch.len_utf8());
            }
        }
        None
    }

    /// Truncate URL with ellipsis if it doesn't fit in available width
    pub(in crate::renderer::event) fn truncate_url_with_ellipsis(
        &self,
        url: &str,
        available_width: usize,
    ) -> String {
        // Always ensure the returned string's display width is <= available_width.
        // Use three-dot ellipsis when possible, otherwise fit the number of dots
        // that can be displayed (including zero when there is no space at all).
        if available_width == 0 {
            return String::new();
        }

        // When very little space remains, prefer a minimal visual indicator that fits.
        if available_width <= 2 {
            return ".".repeat(available_width);
        }

        let ellipsis = "...";
        let ellipsis_width = 3; // display width of "..."

        // If URL already fits, return as is
        if crate::utils::display_width(url) <= available_width {
            return url.to_string();
        }

        // Calculate maximum width for URL content (leaving space for ellipsis)
        let max_url_width = available_width.saturating_sub(ellipsis_width);

        // Find the best truncation point
        let mut truncated = String::new();
        let mut current_width = 0;

        for ch in url.chars() {
            let char_width = crate::utils::display_width(&ch.to_string());
            if current_width + char_width > max_url_width {
                break;
            }
            truncated.push(ch);
            current_width += char_width;
        }

        // Add ellipsis
        format!("{}{}", truncated, ellipsis)
    }
}
