use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn strip_callout_prefix_from_line(
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

    pub(in crate::renderer::event) fn render_callout_pretty_top_border(
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

    pub(in crate::renderer::event) fn render_callout_pretty_bottom_border(
        &self,
        inner_box_width: usize,
    ) -> String {
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

    pub(in crate::renderer::event) fn render_callout_pretty_content_line(
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

    pub(in crate::renderer::event) fn callout_pretty_accent(&self, text: &str) -> String {
        if self.config.no_colors {
            text.to_string()
        } else {
            AnsiStyle::new()
                .fg(PRETTY_ACCENT_COLOR)
                .apply(text, self.config.no_colors)
        }
    }

    pub(in crate::renderer::event) fn wrap_callout_line_for_frame(
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

    pub(in crate::renderer::event) fn normalize_callout_single_char_tail_lines(
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

    pub(in crate::renderer::event) fn has_single_visible_char_tail(lines: &[String]) -> bool {
        lines
            .last()
            .is_some_and(|line| Self::is_single_visible_char_line(line))
    }

    pub(in crate::renderer::event) fn is_single_visible_char_line(line: &str) -> bool {
        let visible = strip_ansi(line);
        let trimmed = visible.trim();
        !trimmed.is_empty() && display_width(trimmed) == 1
    }
}
