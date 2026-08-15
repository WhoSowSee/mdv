use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn push_newline_with_context(&mut self) {
        self.output.push('\n');
        let prefix = self.current_line_prefix();
        if !prefix.is_empty() {
            self.output.push_str(&prefix);
        }
    }

    /// Helper: add only indentation/prefix for the current context (no newline)
    /// Used when we are already at a line start and need to insert the proper
    /// visual prefix (blockquote pipes) and alignment for list content.
    pub(in crate::renderer::event) fn current_line_prefix(&self) -> String {
        self.current_line_prefix_for_blockquote_level_with_options(self.blockquote_level, true)
    }

    /// Prefix for fenced/indented code blocks.
    /// Code blocks should not inherit list continuation indentation.
    pub(in crate::renderer::event) fn current_code_block_prefix(&self) -> String {
        self.current_line_prefix_for_blockquote_level_with_options(self.blockquote_level, false)
    }

    pub(in crate::renderer::event) fn current_rule_prefix(&self) -> String {
        self.current_rule_prefix_for_blockquote_level(self.blockquote_level)
    }

    pub(in crate::renderer::event) fn push_indent_for_line_start(&mut self) {
        let prefix = self.current_line_prefix();
        self.output.push_str(&prefix);
    }

    pub(in crate::renderer::event) fn push_code_block_indent_for_line_start(&mut self) {
        let prefix = self.current_code_block_prefix();
        self.output.push_str(&prefix);
    }

    pub(in crate::renderer::event) fn ensure_contextual_blank_line(&mut self) {
        self.ensure_contextual_blank_line_for_blockquote_level(self.blockquote_level);
    }

    pub(in crate::renderer::event) fn ensure_contextual_blank_lines(&mut self, count: usize) {
        let prefix = self.current_line_prefix_for_blockquote_level(self.blockquote_level);
        self.ensure_contextual_blank_lines_with_prefix(count, &prefix);
    }

    pub(in crate::renderer::event) fn ensure_contextual_blank_lines_with_prefix(
        &mut self,
        count: usize,
        prefix: &str,
    ) {
        if self.output.is_empty() {
            return;
        }
        if count == 0 {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            return;
        }

        self.ensure_contextual_blank_line_with_prefix(prefix);
        let existing = self.trailing_blank_line_count();
        for _ in existing..count {
            self.output.push_str(prefix);
            self.output.push('\n');
        }
    }

    pub(in crate::renderer::event) fn effective_text_width(&self) -> usize {
        let mut width = self.config.get_content_width();
        if self.should_reserve_callout_padding() {
            width = width.saturating_sub(2);
        }
        if self.active_backtick_style.is_some() {
            width = width.saturating_sub(1);
        }
        width
    }

    pub(in crate::renderer::event) fn ensure_contextual_blank_line_for_blockquote_level(
        &mut self,
        level: usize,
    ) {
        let prefix = self.current_line_prefix_for_blockquote_level(level);
        self.ensure_contextual_blank_line_with_prefix(&prefix);
    }

    pub(in crate::renderer::event) fn ensure_contextual_blank_line_with_prefix(
        &mut self,
        prefix: &str,
    ) {
        if self.output.is_empty() {
            return;
        }

        if self.has_trailing_blank_line() {
            if self.trailing_blank_line_matches(prefix) {
                return;
            }
            if self.retarget_source_blank_line(prefix) {
                return;
            }
            self.trim_trailing_blank_lines();
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            self.output.push_str(prefix);
            self.output.push('\n');
            return;
        }

        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.output.push_str(prefix);
        self.output.push('\n');
    }

    pub(in crate::renderer::event) fn has_trailing_blank_line(&self) -> bool {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return false;
        }

        let without_last = &self.output[..self.output.len().saturating_sub(1)];
        let start = without_last
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let last_line = &without_last[start..];

        if last_line.is_empty() {
            return true;
        }

        let clean = strip_layout_metadata(last_line);
        clean
            .chars()
            .all(|ch| ch.is_whitespace() || is_quote_prefix_char(ch))
    }

    pub(in crate::renderer::event) fn trailing_blank_line_count(&self) -> usize {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return 0;
        }

        self.output[..self.output.len().saturating_sub(1)]
            .rsplit('\n')
            .take_while(|line| {
                let clean = strip_layout_metadata(line);
                clean
                    .chars()
                    .all(|ch| ch.is_whitespace() || is_quote_prefix_char(ch))
            })
            .count()
    }

    pub(in crate::renderer::event) fn normalize_trailing_blank_line(&mut self) {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return;
        }

        let len = self.output.len();
        let without_last = &self.output[..len.saturating_sub(1)];
        let start = without_last
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let last_line = &without_last[start..];

        if last_line.is_empty() {
            return;
        }

        let (clean, source_line) = crate::renderer::line_numbers::strip_internal_markers(last_line);
        let clean = strip_ansi(&clean);
        if clean
            .chars()
            .all(|ch| ch.is_whitespace() || is_quote_prefix_char(ch))
        {
            if let Some(source_line) = source_line {
                let marker = crate::renderer::line_numbers::encode_internal_marker(source_line);
                self.output
                    .replace_range(start..len.saturating_sub(1), &marker);
            } else {
                self.output.drain(start..len.saturating_sub(1));
            }
        }
    }

    pub(in crate::renderer::event) fn retarget_source_blank_line(&mut self, prefix: &str) -> bool {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return false;
        }

        let len = self.output.len();
        let without_last = &self.output[..len.saturating_sub(1)];
        let start = without_last
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let last_line = &without_last[start..];
        let (clean, source_line) = crate::renderer::line_numbers::strip_internal_markers(last_line);
        let Some(source_line) = source_line else {
            return false;
        };

        let clean = strip_ansi(&clean);
        if !clean
            .chars()
            .all(|ch| ch.is_whitespace() || is_quote_prefix_char(ch))
        {
            return false;
        }

        let mut replacement = crate::renderer::line_numbers::encode_internal_marker(source_line);
        replacement.push_str(prefix);
        self.output
            .replace_range(start..len.saturating_sub(1), &replacement);
        true
    }

    pub(in crate::renderer::event) fn trim_trailing_blank_lines(&mut self) {
        while self.output.ends_with('\n') {
            let len = self.output.len();
            let without_last = &self.output[..len.saturating_sub(1)];
            let start = without_last
                .rfind('\n')
                .map_or(0, |idx| idx.saturating_add(1));
            let last_line = &without_last[start..];

            if last_line.is_empty() {
                self.output.truncate(start);
                continue;
            }

            let clean = strip_ansi(last_line);
            if clean.trim().is_empty() {
                self.output.truncate(start);
            } else {
                break;
            }
        }
    }

    pub(in crate::renderer::event) fn trailing_blank_line_matches(&self, prefix: &str) -> bool {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return false;
        }

        let without_last = &self.output[..self.output.len().saturating_sub(1)];
        let start = without_last
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let last_line = &without_last[start..];
        let clean = strip_layout_metadata(last_line);
        if clean == strip_layout_metadata(prefix) {
            return true;
        }

        if prefix.is_empty() {
            return clean.trim().is_empty();
        }

        false
    }
}
