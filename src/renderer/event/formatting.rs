use super::{EventRenderer, ThemeElement, create_style};
use crate::terminal::AnsiStyle;
use crate::utils::strip_ansi;
use crossterm::style::Color as CrosstermColor;

impl<'a> EventRenderer<'a> {
    /// Apply current formatting stack to text
    ///
    /// Ensures consistent precedence when multiple styles are active at once
    /// (e.g. Strong + Emphasis). Color precedence: Strong > Emphasis > Strikethrough > Text.
    pub(super) fn apply_formatting(&self, text: &str) -> String {
        if self.formatting_stack.is_empty() {
            return text.to_string();
        }

        let has_strong = self.formatting_stack.contains(&ThemeElement::Strong);
        let has_emphasis = self.formatting_stack.contains(&ThemeElement::Emphasis);
        let has_strike = self.formatting_stack.contains(&ThemeElement::Strikethrough);

        // Choose base element to take the color from with deterministic precedence
        let base_element = if has_strong {
            ThemeElement::Strong
        } else if has_emphasis {
            ThemeElement::Emphasis
        } else if has_strike {
            ThemeElement::Strikethrough
        } else {
            ThemeElement::Text
        };

        let mut style = create_style(self.theme, base_element);

        // Add missing attributes without changing the chosen color
        if has_strong {
            style = style.bold();
        }
        if has_emphasis {
            style = style.italic();
        }
        if has_strike {
            style = style.strikethrough();
        }

        style.apply(text, self.config.no_colors)
    }

    /// Helper: add a newline and then indent for the current context
    /// - Adds heading content indent when present
    /// - Adds blockquote prefix if inside a quote
    /// - Aligns to list content when inside a list (and not in a blockquote)
    pub(super) fn push_newline_with_context(&mut self) {
        self.output.push('\n');

        if self.blockquote_level > 0 {
            if self.content_indent > 0 {
                self.output.push_str(&" ".repeat(self.content_indent));
            }
            let prefix = self.render_blockquote_prefix();
            self.output.push_str(&prefix);
            // If we are inside a list within a blockquote, also align to list content
            if !self.list_stack.is_empty() {
                let full_list_indent = self.calculate_list_content_indent();
                // Avoid double-counting heading indent already applied above
                let additional = full_list_indent.saturating_sub(self.content_indent);
                if additional > 0 {
                    self.output.push_str(&" ".repeat(additional));
                }
            }
        } else if !self.list_stack.is_empty() {
            let list_content_indent = self.calculate_list_content_indent();
            self.output.push_str(&" ".repeat(list_content_indent));
        } else if self.content_indent > 0 {
            self.output.push_str(&" ".repeat(self.content_indent));
        }
    }

    /// Helper: add only indentation/prefix for the current context (no newline)
    /// Used when we are already at a line start and need to insert the proper
    /// visual prefix (blockquote pipes) and alignment for list content.
    pub(super) fn current_line_prefix(&self) -> String {
        let mut prefix = String::new();
        if self.blockquote_level > 0 {
            if self.content_indent > 0 {
                prefix.push_str(&" ".repeat(self.content_indent));
            }
            prefix.push_str(&self.render_blockquote_prefix());
            if !self.list_stack.is_empty() {
                let full_list_indent = self.calculate_list_content_indent();
                let additional = full_list_indent.saturating_sub(self.content_indent);
                if additional > 0 {
                    prefix.push_str(&" ".repeat(additional));
                }
            }
        } else if !self.list_stack.is_empty() {
            let list_content_indent = self.calculate_list_content_indent();
            prefix.push_str(&" ".repeat(list_content_indent));
        } else if self.content_indent > 0 {
            prefix.push_str(&" ".repeat(self.content_indent));
        }
        prefix
    }

    pub(super) fn push_indent_for_line_start(&mut self) {
        let prefix = self.current_line_prefix();
        self.output.push_str(&prefix);
    }

    pub(super) fn ensure_contextual_blank_line(&mut self) {
        if self.output.is_empty() {
            return;
        }

        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        let prefix = self.current_line_prefix();
        if !self.trailing_blank_line_matches(&prefix) {
            self.output.push_str(&prefix);
            self.output.push('\n');
        }
    }

    pub(super) fn has_trailing_blank_line(&self) -> bool {
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

        let clean = strip_ansi(last_line);
        clean.chars().all(|ch| ch.is_whitespace() || ch == '│')
    }

    pub(super) fn normalize_trailing_blank_line(&mut self) {
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

        let clean = strip_ansi(last_line);
        if clean.chars().all(|ch| ch.is_whitespace() || ch == '│') {
            self.output.drain(start..len.saturating_sub(1));
        }
    }

    pub(super) fn trim_trailing_blank_lines(&mut self) {
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

    fn trailing_blank_line_matches(&self, prefix: &str) -> bool {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return false;
        }

        let without_last = &self.output[..self.output.len().saturating_sub(1)];
        let start = without_last
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let last_line = &without_last[start..];
        last_line == prefix
    }

    /// Helper: compute the visible width of indentation/prefix that would be
    /// added by `push_indent_for_line_start()` at the current position.
    /// This mirrors the logic used for headings, blockquotes and lists.
    pub(super) fn compute_line_start_context_width(&self) -> usize {
        if self.blockquote_level > 0 {
            let mut width = 0usize;
            // heading/content indent
            width += self.content_indent;
            // blockquote pipes + trailing space
            width += self.blockquote_level + 1;
            // inside a list, also align to list content (excluding heading indent already counted)
            if !self.list_stack.is_empty() {
                let full = self.calculate_list_content_indent();
                let additional = full.saturating_sub(self.content_indent);
                width += additional;
            }
            width
        } else if !self.list_stack.is_empty() {
            self.calculate_list_content_indent()
        } else {
            self.content_indent
        }
    }
    pub(super) fn render_blockquote_prefix(&self) -> String {
        if self.blockquote_level == 0 {
            return String::new();
        }

        let prefix = format!("{} ", "│".repeat(self.blockquote_level));

        if self.config.no_colors {
            prefix
        } else {
            let style = create_style(self.theme, ThemeElement::Quote);
            style.apply(&prefix, self.config.no_colors)
        }
    }

    pub(super) fn render_code_block_border(&self) -> String {
        self.render_pipe_prefix(1, Some(CrosstermColor::White))
    }

    fn render_pipe_prefix(&self, count: usize, color: Option<CrosstermColor>) -> String {
        if count == 0 {
            return String::new();
        }
        let prefix = format!("{} ", "│".repeat(count));
        if self.config.no_colors {
            return prefix;
        }
        if let Some(color) = color {
            let style = AnsiStyle::new().fg(color);
            style.apply(&prefix, self.config.no_colors)
        } else {
            prefix
        }
    }

    /// Helper: take a visible-width prefix from `s` that fits into `max_width`.
    /// Returns (prefix, rest). Uses display width and is unicode-safe.
    pub(super) fn take_prefix_by_width(&self, s: &str, max_width: usize) -> (String, String) {
        if max_width == 0 || s.is_empty() {
            return (String::new(), s.to_string());
        }

        let mut taken = String::new();
        let mut width = 0usize;
        let mut split_idx = 0usize;
        for (i, ch) in s.char_indices() {
            let ch_w = crate::utils::display_width(&ch.to_string());
            if width + ch_w > max_width {
                break;
            }
            taken.push(ch);
            width += ch_w;
            split_idx = i + ch.len_utf8();
        }
        let rest = s.get(split_idx..).unwrap_or("").to_string();
        (taken, rest)
    }
}
