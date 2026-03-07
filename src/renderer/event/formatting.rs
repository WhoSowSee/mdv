use super::core::{CalloutFold, CalloutKind, CalloutState};
use super::{EventRenderer, PRETTY_ACCENT_COLOR, ThemeElement, create_style};
use crate::terminal::AnsiStyle;
use crate::utils::{WrapMode, display_width, strip_ansi, wrap_text_with_mode};
use crossterm::style::Color as CrosstermColor;

fn is_quote_prefix_char(ch: char) -> bool {
    matches!(ch, '│' | '┃')
}

const DEFAULT_UNKNOWN_CALLOUT_ICON: &str = "";

impl<'a> EventRenderer<'a> {
    pub(super) fn reset_explicit_blank_line_streak(&mut self) {
        self.explicit_blank_line_streak = 0;
    }

    pub(super) fn handle_explicit_blank_line(&mut self) {
        let prefix = self.current_line_prefix();
        let use_prefix = !prefix.is_empty();

        if self.has_trailing_blank_line() {
            if self.explicit_blank_line_streak > 0 {
                if use_prefix {
                    self.output.push('\n');
                    self.output.push_str(&prefix);
                    self.output.push('\n');
                } else {
                    self.output.push('\n');
                }
            }
        } else {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            if use_prefix {
                self.output.push_str(&prefix);
            }
            self.output.push('\n');
        }
        self.explicit_blank_line_streak = self.explicit_blank_line_streak.saturating_add(1);
    }

    pub(super) fn note_paragraph_content(&mut self) {
        if self.table_state.is_some() || self.current_paragraph_start.is_none() {
            return;
        }

        self.reset_explicit_blank_line_streak();

        if !self.current_paragraph_has_content {
            if self.current_paragraph_has_leading_break {
                self.trim_trailing_blank_lines();
                self.ensure_contextual_blank_line();
                self.current_paragraph_has_leading_break = false;
            }
            self.current_paragraph_has_content = true;
        }
    }

    /// Apply current formatting stack to text
    ///
    /// Ensures consistent precedence when multiple styles are active at once
    /// (e.g. Strong + Emphasis). Color precedence: Strong > Emphasis > Strikethrough > Text.
    pub(super) fn apply_formatting(&self, text: &str) -> String {
        self.apply_formatting_with_highlight(text, false)
    }

    /// Apply formatting stack plus optional background highlight (==text==)
    pub(super) fn apply_formatting_with_highlight(&self, text: &str, highlighted: bool) -> String {
        if self.formatting_stack.is_empty() && !highlighted {
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

        if highlighted {
            style = style.bg(self.theme.highlight_background.clone().into());
        }

        style.apply(text, self.config.no_colors)
    }

    /// Helper: add a newline and then indent for the current context
    /// - Adds heading content indent when present
    /// - Adds blockquote prefix if inside a quote
    /// - Aligns to list content when inside a list (and not in a blockquote)
    pub(super) fn push_newline_with_context(&mut self) {
        self.output.push('\n');
        let prefix = self.current_line_prefix();
        if !prefix.is_empty() {
            self.output.push_str(&prefix);
        }
    }

    /// Helper: add only indentation/prefix for the current context (no newline)
    /// Used when we are already at a line start and need to insert the proper
    /// visual prefix (blockquote pipes) and alignment for list content.
    pub(super) fn current_line_prefix(&self) -> String {
        self.current_line_prefix_for_blockquote_level_with_options(self.blockquote_level, true)
    }

    /// Prefix for fenced/indented code blocks.
    /// Code blocks should not inherit list continuation indentation.
    pub(super) fn current_code_block_prefix(&self) -> String {
        self.current_line_prefix_for_blockquote_level_with_options(self.blockquote_level, false)
    }

    pub(super) fn current_rule_prefix(&self) -> String {
        self.current_rule_prefix_for_blockquote_level(self.blockquote_level)
    }

    pub(super) fn push_indent_for_line_start(&mut self) {
        let prefix = self.current_line_prefix();
        self.output.push_str(&prefix);
    }

    pub(super) fn push_code_block_indent_for_line_start(&mut self) {
        let prefix = self.current_code_block_prefix();
        self.output.push_str(&prefix);
    }

    pub(super) fn ensure_contextual_blank_line(&mut self) {
        self.ensure_contextual_blank_line_for_blockquote_level(self.blockquote_level);
    }

    pub(super) fn effective_text_width(&self) -> usize {
        let mut width = self.config.get_terminal_width();
        if self.should_reserve_callout_padding() {
            width = width.saturating_sub(2);
        }
        width
    }

    pub(super) fn ensure_contextual_blank_line_for_blockquote_level(&mut self, level: usize) {
        let prefix = self.current_line_prefix_for_blockquote_level(level);
        self.ensure_contextual_blank_line_with_prefix(&prefix);
    }

    pub(super) fn ensure_contextual_blank_line_with_prefix(&mut self, prefix: &str) {
        if self.output.is_empty() {
            return;
        }

        if self.has_trailing_blank_line() {
            if self.trailing_blank_line_matches(prefix) {
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
        clean
            .chars()
            .all(|ch| ch.is_whitespace() || is_quote_prefix_char(ch))
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
        if clean
            .chars()
            .all(|ch| ch.is_whitespace() || is_quote_prefix_char(ch))
        {
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

    pub(super) fn trailing_blank_line_matches(&self, prefix: &str) -> bool {
        if self.output.is_empty() || !self.output.ends_with('\n') {
            return false;
        }

        let without_last = &self.output[..self.output.len().saturating_sub(1)];
        let start = without_last
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let last_line = &without_last[start..];
        if last_line == prefix {
            return true;
        }

        if prefix.is_empty() {
            let clean = strip_ansi(last_line);
            return clean.trim().is_empty();
        }

        false
    }

    /// Helper: compute the visible width of indentation/prefix that would be
    /// added by `push_indent_for_line_start()` at the current position.
    /// This mirrors the logic used for headings, blockquotes and lists.
    pub(super) fn compute_line_start_context_width(&self) -> usize {
        let prefix = self.current_line_prefix();
        display_width(&strip_ansi(&prefix))
    }

    pub(super) fn compute_code_block_context_width(&self) -> usize {
        let prefix = self.current_code_block_prefix();
        display_width(&strip_ansi(&prefix))
    }
    pub(super) fn render_blockquote_prefix(&self) -> String {
        self.render_blockquote_prefix_for_level(self.blockquote_level)
    }

    pub(super) fn render_blockquote_prefix_for_level(&self, level: usize) -> String {
        if level == 0 {
            return String::new();
        }

        let mut prefix = String::new();
        for idx in 0..level {
            let symbol = match self.callout_stack.get(idx) {
                Some(CalloutState::Active(_)) => '┃',
                _ => '│',
            };
            prefix.push(symbol);
        }
        prefix.push(' ');

        if self.config.no_colors {
            prefix
        } else {
            let style = create_style(self.theme, ThemeElement::Quote);
            style.apply(&prefix, self.config.no_colors)
        }
    }

    pub(super) fn should_indent_after_blockquote_prefix(&self, level: usize) -> bool {
        if level == 0 {
            return false;
        }

        matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) && self
            .callout_stack
            .iter()
            .take(level)
            .any(|state| matches!(state, CalloutState::Active(_)))
    }

    fn current_line_prefix_for_blockquote_level(&self, level: usize) -> String {
        self.current_line_prefix_for_blockquote_level_with_options(level, true)
    }

    fn current_line_prefix_for_blockquote_level_with_options(
        &self,
        level: usize,
        include_list_indent: bool,
    ) -> String {
        let mut prefix = String::new();
        if level > 0 {
            let base_indent = if self.current_heading_start.is_some() {
                self.heading_indent
            } else {
                self.content_indent
            };
            let indent_after_prefix = self.should_indent_after_blockquote_prefix(level);
            if base_indent > 0 && !indent_after_prefix {
                prefix.push_str(&" ".repeat(base_indent));
            }
            prefix.push_str(&self.render_blockquote_prefix_for_level(level));
            if base_indent > 0 && indent_after_prefix {
                prefix.push_str(&" ".repeat(base_indent));
            }
            if include_list_indent && !self.list_stack.is_empty() {
                let list_indent = self
                    .calculate_list_content_indent()
                    .saturating_sub(self.content_indent);
                if list_indent > 0 {
                    prefix.push_str(&" ".repeat(list_indent));
                }
            }
        } else if include_list_indent && !self.list_stack.is_empty() {
            let list_content_indent = self.calculate_list_content_indent();
            prefix.push_str(&" ".repeat(list_content_indent));
        } else {
            let base_indent = if self.current_heading_start.is_some() {
                self.heading_indent
            } else {
                self.content_indent
            };
            if base_indent > 0 {
                prefix.push_str(&" ".repeat(base_indent));
            }
        }
        prefix
    }

    fn current_rule_prefix_for_blockquote_level(&self, level: usize) -> String {
        let mut prefix = String::new();
        if level > 0 {
            prefix.push_str(&self.render_blockquote_prefix_for_level(level));
            if !self.list_stack.is_empty() {
                let list_indent = self.calculate_list_content_indent();
                if list_indent > 0 {
                    prefix.push_str(&" ".repeat(list_indent));
                }
            }
        } else if !self.list_stack.is_empty() {
            let list_content_indent = self.calculate_list_content_indent();
            if list_content_indent > 0 {
                prefix.push_str(&" ".repeat(list_content_indent));
            }
        }
        prefix
    }

    fn should_reserve_callout_padding(&self) -> bool {
        matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Pretty
        ) && self
            .callout_stack
            .iter()
            .any(|state| matches!(state, CalloutState::Active(_)))
    }

    fn callout_label_style(&self, kind: CalloutKind, label: &str) -> AnsiStyle {
        let color = if let Some(custom) = self.config.custom_callouts.get(label) {
            custom
                .color
                .clone()
                .unwrap_or_else(|| self.unknown_callout_color())
        } else {
            self.callout_palette
                .get(&kind)
                .cloned()
                .unwrap_or_else(|| self.theme.text.clone())
        };

        AnsiStyle::new().fg(color.into()).bold()
    }

    fn unknown_callout_color(&self) -> crate::theme::Color {
        self.callout_palette
            .get(&CalloutKind::Tip)
            .cloned()
            .unwrap_or_else(|| self.theme.text.clone())
    }

    fn callout_label_text(
        &self,
        label: &str,
        label_override: Option<&str>,
        fold: Option<CalloutFold>,
        icon_spacing: usize,
    ) -> String {
        let base = self.callout_display_label(label, label_override);
        if !self.config.callout_style.show_icons {
            return base;
        }

        let icon = match self.callout_icon_for_label(label) {
            Some(icon) => icon,
            None => return base,
        };

        let mut text = String::new();
        text.push_str(icon);
        if icon_spacing > 0 {
            text.push_str(&" ".repeat(icon_spacing));
        }
        text.push_str(&base);
        if let Some(icon) = self.callout_fold_icon(fold) {
            text.push(' ');
            text.push_str(icon);
        }
        text
    }

    fn build_callout_label_text(
        &self,
        label: &str,
        label_override: Option<&str>,
        fold: Option<CalloutFold>,
        icon_spacing: usize,
    ) -> String {
        self.callout_label_text(label, label_override, fold, icon_spacing)
    }

    fn callout_display_label(&self, label: &str, label_override: Option<&str>) -> String {
        if let Some(label_override) = label_override {
            let trimmed = label_override.trim();
            if !trimmed.is_empty() {
                if self.config.callout_style.uppercase {
                    return trimmed.to_ascii_uppercase();
                }
                return trimmed.to_string();
            }
        }

        self.format_callout_label_case(label)
    }

    fn callout_fold_icon(&self, fold: Option<CalloutFold>) -> Option<&'static str> {
        if !self.config.callout_style.show_icons || !self.config.callout_style.show_fold_icons {
            return None;
        }

        match fold {
            Some(CalloutFold::Expanded) => Some(""),
            Some(CalloutFold::Collapsed) => Some(""),
            None => None,
        }
    }

    fn format_callout_label_case(&self, label: &str) -> String {
        if self.config.callout_style.uppercase {
            return label.to_ascii_uppercase();
        }

        let lower = label.to_ascii_lowercase();
        if lower == "faq" {
            return "FAQ".to_string();
        }

        let mut chars = lower.chars();
        match chars.next() {
            Some(first) => {
                let mut result = String::new();
                result.push(first.to_ascii_uppercase());
                result.push_str(chars.as_str());
                result
            }
            None => String::new(),
        }
    }

    fn callout_icon_spacing(&self, label_inside: bool) -> usize {
        if !self.config.callout_style.show_icons {
            return 0;
        }

        if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) {
            return 2;
        }

        if label_inside { 2 } else { 1 }
    }

    fn callout_icon_for_label(&self, label: &str) -> Option<&str> {
        if let Some(custom) = self.config.custom_callouts.get(label) {
            if let Some(icon) = custom.icon.as_deref() {
                return Some(icon);
            }
            if let Some(default_icon) = Self::default_callout_icon_for_label(label) {
                return Some(default_icon);
            }
            return Some(DEFAULT_UNKNOWN_CALLOUT_ICON);
        }

        Self::default_callout_icon_for_label(label).or(Some(DEFAULT_UNKNOWN_CALLOUT_ICON))
    }

    fn default_callout_icon_for_label(label: &str) -> Option<&'static str> {
        match label {
            "note" | "seealso" => Some(""),
            "info" => Some(""),
            "abstract" => Some("󰈚"),
            "summary" | "tldr" => Some(""),
            "example" => Some("󰅍"),
            "todo" => Some("󰅎"),
            "tip" => Some(""),
            "hint" => Some("󰌵"),
            "important" => Some("󰅽"),
            "success" | "check" | "done" => Some(""),
            "question" | "help" => Some(""),
            "faq" => Some("󰠗"),
            "warning" | "caution" | "attention" => Some(""),
            "failure" | "fail" | "missing" | "error" => Some(""),
            "danger" => Some(""),
            "bug" => Some(""),
            "quote" | "cite" => Some(""),
            _ => None,
        }
    }

    pub(super) fn render_callout_header(
        &mut self,
        kind: CalloutKind,
        label: &str,
        label_override: Option<&str>,
        fold: Option<CalloutFold>,
    ) {
        let outer_level = self.blockquote_level.saturating_sub(1);
        self.ensure_contextual_blank_line_for_blockquote_level(outer_level);

        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.push_indent_for_line_start();

        let icon_spacing = self.callout_icon_spacing(false);
        let label_text = self.callout_label_text(label, label_override, fold, icon_spacing);
        let display_label = if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) && self.config.callout_style.show_icons
        {
            label_text
        } else {
            format!("[{}]", label_text)
        };
        let label_style = self.callout_label_style(kind, label);
        let styled_label = label_style.apply(&display_label, self.config.no_colors);
        self.output.push_str(&styled_label);

        self.output.push('\n');
        self.push_indent_for_line_start();
        self.output.push('\n');

        if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) {
            self.suppress_next_paragraph_break = true;
        }
    }

    pub(super) fn maybe_render_callout_header(&mut self) {
        if self.pending_callout_label_override {
            return;
        }
        let mut header = None;
        if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut()
            && !info.header_rendered
        {
            info.header_rendered = true;
            header = Some((
                info.kind,
                info.label.clone(),
                info.label_override.clone(),
                info.fold,
            ));
        }

        if let Some((kind, label, label_override, fold)) = header {
            self.render_callout_header(kind, &label, label_override.as_deref(), fold);
        }
    }

    pub(super) fn finalize_pending_callout_label_override(&mut self) -> bool {
        if !self.pending_callout_label_override {
            return false;
        }

        let label_override = self.pending_callout_label_buffer.trim();
        let mut header = None;

        if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut() {
            if !label_override.is_empty() {
                info.label_override = Some(label_override.to_string());
            }

            if !info.header_rendered {
                info.header_rendered = true;
                header = Some((
                    info.kind,
                    info.label.clone(),
                    info.label_override.clone(),
                    info.fold,
                ));
            }
        }

        self.pending_callout_label_override = false;
        self.pending_callout_label_buffer.clear();

        if let Some((kind, label, label_override, fold)) = header {
            self.render_callout_header(kind, &label, label_override.as_deref(), fold);
            return true;
        }

        false
    }

    pub(super) fn render_callout_pretty_block(
        &mut self,
        callout_block: &str,
        callout_level: usize,
        kind: CalloutKind,
        label: &str,
        label_override: Option<&str>,
        fold: Option<CalloutFold>,
    ) -> bool {
        let mut lines: Vec<&str> = callout_block.split('\n').collect();
        if lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }

        let list_indent = if self.list_stack.is_empty() {
            0
        } else {
            self.calculate_list_content_indent()
                .saturating_sub(self.content_indent)
        };

        let mut leading_blank_line: Option<String> = None;
        let mut start_idx = 0usize;
        while start_idx < lines.len() {
            let stripped =
                self.strip_callout_prefix_from_line(lines[start_idx], callout_level, list_indent);
            if strip_ansi(&stripped).trim().is_empty() {
                leading_blank_line = Some(stripped);
                start_idx += 1;
            } else {
                break;
            }
        }

        if start_idx < lines.len() {
            start_idx = start_idx.saturating_add(1);
        }

        if start_idx < lines.len() {
            let stripped =
                self.strip_callout_prefix_from_line(lines[start_idx], callout_level, list_indent);
            if strip_ansi(&stripped).trim().is_empty() {
                start_idx += 1;
            }
        }

        let mut content_lines: Vec<String> = lines[start_idx..]
            .iter()
            .map(|line| self.strip_callout_prefix_from_line(line, callout_level, list_indent))
            .collect();

        while matches!(content_lines.first(), Some(line) if strip_ansi(line).trim().is_empty()) {
            content_lines.remove(0);
        }
        while matches!(content_lines.last(), Some(line) if strip_ansi(line).trim().is_empty()) {
            content_lines.pop();
        }

        let label_inside = self.config.callout_style.label_inside;
        if label_inside {
            while matches!(content_lines.first(), Some(line) if strip_ansi(line).trim().is_empty())
            {
                content_lines.remove(0);
            }
        } else if content_lines.is_empty() {
            content_lines.push(String::new());
        }

        if label_inside {
            let icon_spacing = self.callout_icon_spacing(true);
            let label_text =
                self.build_callout_label_text(label, label_override, fold, icon_spacing);
            let styled_label = if label_text.is_empty() {
                String::new()
            } else {
                self.callout_label_style(kind, label)
                    .apply(&label_text, self.config.no_colors)
            };

            let mut lines_with_label = Vec::with_capacity(content_lines.len() + 2);
            if !label_text.is_empty() {
                lines_with_label.push(styled_label);
            }
            lines_with_label.push(String::new());
            lines_with_label.extend(content_lines);
            content_lines = lines_with_label;
        } else if content_lines.is_empty() {
            content_lines.push(String::new());
        }

        let terminal_width = self.config.get_terminal_width();
        let context_width = self.compute_line_start_context_width();
        let available_frame_width = terminal_width.saturating_sub(context_width);
        if available_frame_width <= 4 {
            return false;
        }

        let left_padding = 1usize;
        let right_padding = 1usize;
        let available_content_width = available_frame_width
            .saturating_sub(2 + left_padding + right_padding)
            .max(1);
        if available_content_width == 0 {
            return false;
        }

        let mut max_content_width = 0usize;
        for line in &content_lines {
            max_content_width = max_content_width.max(display_width(&strip_ansi(line)));
        }

        let wrap_mode = match self.config.text_wrap_mode() {
            WrapMode::None => WrapMode::Character,
            other => other,
        };

        if max_content_width > available_content_width {
            let mut wrapped_lines = Vec::new();
            for line in content_lines {
                let line_width = display_width(&strip_ansi(&line));
                if line_width <= available_content_width {
                    wrapped_lines.push(line);
                    continue;
                }
                let wrapped =
                    self.wrap_callout_line_for_frame(&line, available_content_width, wrap_mode);
                wrapped_lines.extend(wrapped);
            }
            content_lines = wrapped_lines;
        }

        self.normalize_callout_single_char_tail_lines(
            &mut content_lines,
            available_content_width,
            wrap_mode,
        );

        max_content_width = 0usize;
        for line in &content_lines {
            max_content_width = max_content_width.max(display_width(&strip_ansi(line)));
        }

        let label_text = if label_inside {
            String::new()
        } else {
            let icon_spacing = self.callout_icon_spacing(false);
            self.build_callout_label_text(label, label_override, fold, icon_spacing)
        };
        let label_width = display_width(label_text.trim());

        let mut text_width = left_padding + max_content_width + right_padding;
        if text_width == 0 {
            text_width = 1;
        }
        let mut inner_box_width = text_width + 2;

        if label_width > 0 {
            let required_inner_width = label_width.saturating_add(6);
            if inner_box_width < required_inner_width {
                if required_inner_width <= available_frame_width {
                    inner_box_width = required_inner_width;
                    text_width = inner_box_width.saturating_sub(2).max(1);
                } else {
                    return false;
                }
            }
        }

        if inner_box_width > available_frame_width {
            return false;
        }

        if let Some(blank_line) = leading_blank_line {
            if !self.output.is_empty() && !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            if !blank_line.is_empty() {
                self.output.push_str(&blank_line);
            }
            self.output.push('\n');
        } else {
            self.ensure_contextual_blank_line();
        }

        self.push_indent_for_line_start();
        let top_line =
            self.render_callout_pretty_top_border(inner_box_width, kind, &label_text, label);
        self.output.push_str(&top_line);
        self.output.push('\n');

        for line in content_lines {
            self.push_indent_for_line_start();
            let content_line = self.render_callout_pretty_content_line(
                text_width,
                &line,
                left_padding,
                right_padding,
            );
            self.output.push_str(&content_line);
            self.output.push('\n');
        }

        self.push_indent_for_line_start();
        let bottom_line = self.render_callout_pretty_bottom_border(inner_box_width);
        self.output.push_str(&bottom_line);
        self.output.push('\n');

        self.ensure_contextual_blank_line();

        true
    }

    fn strip_callout_prefix_from_line(
        &self,
        line: &str,
        callout_level: usize,
        list_indent: usize,
    ) -> String {
        if callout_level == 0 {
            return line.to_string();
        }

        let clean = strip_ansi(line);
        let mut clean_chars = clean.chars().peekable();
        let mut leading_indent = 0usize;
        while matches!(clean_chars.peek(), Some(ch) if ch.is_whitespace()) {
            leading_indent += 1;
            clean_chars.next();
        }

        let mut pipe_count = 0usize;
        while matches!(clean_chars.peek(), Some(ch) if *ch == '│' || *ch == '┃') {
            pipe_count += 1;
            clean_chars.next();
        }

        if pipe_count == 0 || !matches!(clean_chars.peek(), Some(ch) if *ch == ' ') {
            return line.to_string();
        }

        let remove_pipes = callout_level.min(pipe_count);
        if remove_pipes == 0 {
            return line.to_string();
        }
        let remaining_pipes = pipe_count.saturating_sub(remove_pipes);

        let mut result = String::with_capacity(line.len());
        let mut in_escape = false;
        let mut seen_pipes = 0usize;
        let mut removed_pipes = 0usize;
        let mut prefix_started = false;
        let mut prefix_done = false;
        let mut remaining_list_indent = list_indent;
        let mut leading_indent_remaining = leading_indent;

        for ch in line.chars() {
            if in_escape {
                result.push(ch);
                if ch == 'm' {
                    in_escape = false;
                }
                continue;
            }

            if ch == '\x1b' {
                in_escape = true;
                result.push(ch);
                continue;
            }

            if !prefix_started {
                if ch.is_whitespace() {
                    if leading_indent_remaining > 0 {
                        leading_indent_remaining = leading_indent_remaining.saturating_sub(1);
                        result.push(ch);
                    }
                    continue;
                }
                prefix_started = true;
            }

            if !prefix_done {
                if ch == '│' || ch == '┃' {
                    seen_pipes += 1;
                    if removed_pipes < remove_pipes {
                        removed_pipes += 1;
                        continue;
                    }
                    result.push(ch);
                    continue;
                }

                if seen_pipes > 0 && ch == ' ' {
                    if remaining_pipes > 0 {
                        result.push(ch);
                    }
                    prefix_done = true;
                    continue;
                }

                prefix_done = true;
            }

            if prefix_done && remaining_list_indent > 0 && ch == ' ' {
                remaining_list_indent = remaining_list_indent.saturating_sub(1);
                continue;
            }

            result.push(ch);
        }

        result
    }

    fn render_callout_pretty_top_border(
        &self,
        inner_box_width: usize,
        kind: CalloutKind,
        label: &str,
        label_key: &str,
    ) -> String {
        let mut line = String::new();
        if inner_box_width == 0 {
            return line;
        }

        line.push_str(&self.callout_pretty_accent("╭"));

        if inner_box_width == 1 {
            line.push_str(&self.callout_pretty_accent("╮"));
            return line;
        }

        let mut middle_width = inner_box_width.saturating_sub(2);
        if middle_width > 0 {
            line.push_str(&self.callout_pretty_accent("─"));
            middle_width = middle_width.saturating_sub(1);
        }

        let trimmed = label.trim();
        if !trimmed.is_empty() && middle_width >= 2 {
            let max_label_width = middle_width.saturating_sub(2);
            if max_label_width > 0 {
                let mut label_text = trimmed.to_string();
                if display_width(&label_text) > max_label_width {
                    label_text = self.take_prefix_by_width(&label_text, max_label_width).0;
                }

                let label_width = display_width(&label_text);
                if label_width > 0 {
                    line.push_str(&self.callout_pretty_accent(" "));
                    let styled_label = self
                        .callout_label_style(kind, label_key)
                        .apply(&label_text, self.config.no_colors);
                    line.push_str(&styled_label);
                    line.push_str(&self.callout_pretty_accent(" "));
                    middle_width = middle_width.saturating_sub(label_width + 2);
                }
            }
        }

        while middle_width > 0 {
            line.push_str(&self.callout_pretty_accent("─"));
            middle_width = middle_width.saturating_sub(1);
        }

        line.push_str(&self.callout_pretty_accent("╮"));
        line
    }

    fn render_callout_pretty_bottom_border(&self, inner_box_width: usize) -> String {
        let mut line = String::new();
        if inner_box_width == 0 {
            return line;
        }

        line.push_str(&self.callout_pretty_accent("╰"));
        if inner_box_width > 1 {
            let repeat = inner_box_width.saturating_sub(2);
            if repeat > 0 {
                line.push_str(&self.callout_pretty_accent(&"─".repeat(repeat)));
            }
            line.push_str(&self.callout_pretty_accent("╯"));
        } else {
            line.push_str(&self.callout_pretty_accent("╯"));
        }
        line
    }

    fn render_callout_pretty_content_line(
        &self,
        text_width: usize,
        part: &str,
        left_padding: usize,
        right_padding: usize,
    ) -> String {
        let content_width = display_width(&strip_ansi(part));
        let base_width = left_padding + content_width + right_padding;
        let line_width = text_width.max(1);
        let trailing_pad = line_width.saturating_sub(base_width);

        let mut line = String::new();
        line.push_str(&self.callout_pretty_accent("│"));
        if left_padding > 0 {
            line.push_str(&" ".repeat(left_padding));
        }
        line.push_str(part);
        if right_padding > 0 {
            line.push_str(&" ".repeat(right_padding));
        }
        if trailing_pad > 0 {
            line.push_str(&" ".repeat(trailing_pad));
        }
        line.push_str(&self.callout_pretty_accent("│"));
        line
    }

    fn callout_pretty_accent(&self, text: &str) -> String {
        if self.config.no_colors {
            text.to_string()
        } else {
            AnsiStyle::new()
                .fg(PRETTY_ACCENT_COLOR)
                .apply(text, self.config.no_colors)
        }
    }

    fn wrap_callout_line_for_frame(
        &self,
        line: &str,
        width: usize,
        wrap_mode: WrapMode,
    ) -> Vec<String> {
        let wrapped = wrap_text_with_mode(line, width, wrap_mode);
        let mut lines: Vec<String> = wrapped.split('\n').map(|part| part.to_string()).collect();

        if matches!(wrap_mode, WrapMode::Word) {
            let mut normalized = Vec::with_capacity(lines.len());
            for part in lines {
                if display_width(&strip_ansi(&part)) <= width {
                    normalized.push(part);
                    continue;
                }

                // Keep pretty callouts renderable when word wrap meets a single
                // token that is wider than the available frame width.
                let char_wrapped = wrap_text_with_mode(&part, width, WrapMode::Character);
                normalized.extend(char_wrapped.split('\n').map(|segment| segment.to_string()));
            }
            lines = normalized;
        }

        if !matches!(wrap_mode, WrapMode::Character) || !Self::has_single_visible_char_tail(&lines)
        {
            return lines;
        }

        // In char mode, prefer word fallback when char wrapping leaves a 1-char orphan tail.
        let word_wrapped = wrap_text_with_mode(line, width, WrapMode::Word);
        let word_lines: Vec<String> = word_wrapped
            .split('\n')
            .map(|part| part.to_string())
            .collect();
        let word_fits = word_lines
            .iter()
            .all(|part| display_width(&strip_ansi(part)) <= width);

        if word_fits && !Self::has_single_visible_char_tail(&word_lines) {
            lines = word_lines;
        }

        lines
    }

    fn normalize_callout_single_char_tail_lines(
        &self,
        lines: &mut Vec<String>,
        width: usize,
        wrap_mode: WrapMode,
    ) {
        if !matches!(wrap_mode, WrapMode::Character) || width == 0 || lines.len() < 2 {
            return;
        }

        let mut idx = 1usize;
        while idx < lines.len() {
            let has_single_char_tail = Self::is_single_visible_char_line(&lines[idx]);
            let previous_visible = strip_ansi(&lines[idx - 1]);
            let current_visible = strip_ansi(&lines[idx]);
            let previous_tail = previous_visible.trim_end().chars().next_back();
            let current_head = current_visible.trim_start().chars().next();
            let is_word_boundary_split = previous_tail.is_some_and(|ch| ch.is_alphanumeric())
                && current_head.is_some_and(|ch| ch.is_alphanumeric());

            if !has_single_char_tail || !is_word_boundary_split {
                idx += 1;
                continue;
            }

            let merged = format!("{}{}", lines[idx - 1].trim_end(), lines[idx].trim_start());
            let replacement = self.wrap_callout_line_for_frame(&merged, width, wrap_mode);
            let replacement_valid = !replacement.is_empty()
                && replacement
                    .iter()
                    .all(|part| display_width(&strip_ansi(part)) <= width)
                && replacement
                    .iter()
                    .all(|part| !Self::is_single_visible_char_line(part));

            if replacement_valid {
                lines.splice(idx - 1..=idx, replacement);
                idx = idx.saturating_sub(1).max(1);
            } else {
                idx += 1;
            }
        }
    }

    fn has_single_visible_char_tail(lines: &[String]) -> bool {
        lines
            .last()
            .is_some_and(|line| Self::is_single_visible_char_line(line))
    }

    fn is_single_visible_char_line(line: &str) -> bool {
        let visible = strip_ansi(line);
        let trimmed = visible.trim();
        !trimmed.is_empty() && display_width(trimmed) == 1
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
