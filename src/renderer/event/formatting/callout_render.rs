use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn render_callout_pretty_block(
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

        let mut leading_blank_lines = Vec::new();
        let mut start_idx = 0usize;
        while start_idx < lines.len() {
            let stripped =
                self.strip_callout_prefix_from_line(lines[start_idx], callout_level, list_indent);
            if strip_ansi(&stripped).trim().is_empty() {
                leading_blank_lines.push(stripped);
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
            let label_text = self.callout_label_text(label, label_override, fold, icon_spacing);
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

        let terminal_width = self.config.get_content_width();
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

        let wrap_mode = match (kind, self.config.text_wrap_mode()) {
            (CalloutKind::Properties, _) | (_, WrapMode::None) => WrapMode::Character,
            (_, other) => other,
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
            self.callout_label_text(label, label_override, fold, icon_spacing)
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

        let spacing = self.config.block_spacing.spacing(BlockElement::Callout);
        if leading_blank_lines.is_empty() {
            self.ensure_contextual_blank_lines(spacing.top);
        } else {
            for blank_line in leading_blank_lines {
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
                if !blank_line.is_empty() {
                    self.output.push_str(&blank_line);
                }
                self.output.push('\n');
            }
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
                kind,
            );
            self.output.push_str(&content_line);
            self.output.push('\n');
        }

        self.push_indent_for_line_start();
        let bottom_line = self.render_callout_pretty_bottom_border(inner_box_width, kind);
        self.output.push_str(&bottom_line);
        self.output.push('\n');

        true
    }
}
