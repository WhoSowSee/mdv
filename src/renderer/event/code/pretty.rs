use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn render_code_block_pretty(
        &mut self,
        input: CodeBlockRenderInput<'_>,
    ) -> Result<()> {
        let left_padding = 1usize;
        let right_padding = 1usize;

        let context_width = self.compute_code_block_context_width();
        let available_frame_width = input.terminal_width.saturating_sub(context_width);
        if available_frame_width <= 4 {
            return self.render_code_block_simple(input);
        }

        let max_inner_box_width = available_frame_width;
        let max_text_width_allowed = max_inner_box_width.saturating_sub(2);
        if max_text_width_allowed < left_padding + right_padding + 1 {
            return self.render_code_block_simple(input);
        }

        let highlight_lines: Vec<&str> = input.highlighted.lines().collect();
        let raw_code_lines: Vec<&str> = input.raw_code.lines().collect();
        let mut max_line_width = 0usize;
        for line in &highlight_lines {
            max_line_width = max_line_width.max(display_width(&strip_ansi(line)));
        }

        let wrap_width_allowed =
            max_text_width_allowed.saturating_sub(left_padding + right_padding);
        let needs_wrap = input.should_wrap
            && max_line_width + left_padding + right_padding > max_text_width_allowed;

        let mut rendered_lines: Vec<String> = Vec::new();
        let mut max_part_width = 0usize;

        if needs_wrap {
            if wrap_width_allowed == 0 {
                return self.render_code_block_simple(input);
            }

            for (idx, line) in highlight_lines.iter().enumerate() {
                let raw_line = raw_code_lines.get(idx).copied();
                let segments = self.wrap_code_line_segments_pretty(
                    line,
                    raw_line,
                    wrap_width_allowed,
                    input.should_wrap,
                    input.wrap_mode,
                );

                for segment in segments {
                    max_part_width = max_part_width.max(segment.visible_width);
                    rendered_lines.push(segment.text);
                }
            }

            if max_part_width > wrap_width_allowed {
                return self.render_code_block_simple(input);
            }
        } else {
            if highlight_lines.is_empty() {
                rendered_lines.push(String::new());
            } else {
                for (idx, line) in highlight_lines.iter().enumerate() {
                    let raw_line = raw_code_lines.get(idx).copied();
                    let segments = self.wrap_code_line_segments_pretty(
                        line,
                        raw_line,
                        wrap_width_allowed,
                        false,
                        input.wrap_mode,
                    );

                    for segment in segments {
                        max_part_width = max_part_width.max(segment.visible_width);
                        rendered_lines.push(segment.text);
                    }
                }
            }

            if max_part_width + left_padding + right_padding > max_text_width_allowed {
                return self.render_code_block_simple(input);
            }
        }

        if rendered_lines.is_empty() {
            rendered_lines.push(String::new());
        }

        let block_is_empty = rendered_lines
            .iter()
            .all(|line| strip_ansi(line).trim().is_empty());

        let mut text_width = left_padding + max_part_width + right_padding;
        let mut inner_box_width = text_width + 2;

        if let Some(label) = input.language_label
            && !label.trim().is_empty()
        {
            if block_is_empty {
                let label_width = display_width(label);
                let required_inner_width = label_width + 6;
                if required_inner_width > max_inner_box_width {
                    return self.render_code_block_simple(input);
                }
            }

            let label_width = display_width(label);
            // Ensure at least one trailing dash after the label on the top border
            // so frames like an empty "Text" block appear balanced: "╭─ Text ─╮".
            let required_inner_width = (label_width + 6).min(max_inner_box_width);
            if inner_box_width < required_inner_width {
                inner_box_width = required_inner_width;
                text_width = inner_box_width.saturating_sub(2);
            }
        }

        self.push_code_block_indent_for_line_start();
        let top_line = self.render_pretty_top_border(inner_box_width, input.language_label);
        self.output.push_str(&top_line);
        self.output.push('\n');

        for part in rendered_lines {
            self.push_code_block_indent_for_line_start();
            let decorated = self.highlight_footnote_markers_in_ansi(&part);
            let content_line = self.render_pretty_content_line(text_width, &decorated);
            self.output.push_str(&content_line);
            self.output.push('\n');
        }

        self.push_code_block_indent_for_line_start();
        let bottom_line = self.render_pretty_bottom_border(inner_box_width);
        self.output.push_str(&bottom_line);
        self.output.push('\n');

        Ok(())
    }

    pub(super) fn wrap_code_line_segments_pretty(
        &self,
        highlighted_line: &str,
        raw_line: Option<&str>,
        width: usize,
        should_wrap: bool,
        wrap_mode: WrapMode,
    ) -> Vec<WrappedCodeSegment> {
        let mut segments =
            self.wrap_code_line_segments(highlighted_line, raw_line, width, should_wrap, wrap_mode);

        if should_wrap && width > 0 && matches!(wrap_mode, WrapMode::Word) {
            let has_overflow = segments.iter().any(|segment| segment.visible_width > width);
            if has_overflow {
                // Fall back to character wrapping to keep the pretty frame consistent.
                segments = self.wrap_code_line_segments(
                    highlighted_line,
                    raw_line,
                    width,
                    should_wrap,
                    WrapMode::Character,
                );
            }
        }

        segments
    }

    pub(super) fn wrap_code_line_segments(
        &self,
        highlighted_line: &str,
        raw_line: Option<&str>,
        width: usize,
        should_wrap: bool,
        wrap_mode: WrapMode,
    ) -> Vec<WrappedCodeSegment> {
        let base_indent = if let Some(line) = raw_line {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        } else {
            let stripped = strip_ansi(highlighted_line);
            stripped
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        };

        let continuation_indent = match self.config.code_wrap_indent {
            CodeWrapIndent::None => String::new(),
            CodeWrapIndent::Base => base_indent.clone(),
            CodeWrapIndent::Double => {
                let mut indent = base_indent.clone();
                indent.push_str("  ");
                indent
            }
        };

        let raw_wrapped = if should_wrap && width > 0 {
            crate::utils::wrap_text_with_mode(highlighted_line, width, wrap_mode)
        } else {
            highlighted_line.to_string()
        };

        let mut segments_raw: Vec<String> = raw_wrapped
            .split('\n')
            .map(|part| part.to_string())
            .collect();
        if segments_raw.is_empty() {
            segments_raw.push(String::new());
        }

        let mut segments = Vec::with_capacity(segments_raw.len());
        for (idx, mut segment) in segments_raw.into_iter().enumerate() {
            let mut visible_width = display_width(&strip_ansi(&segment));

            if idx > 0 && !continuation_indent.is_empty() {
                let candidate = format!("{}{}", continuation_indent, segment);
                let candidate_width = display_width(&strip_ansi(&candidate));
                if should_wrap && width > 0 && candidate_width > width {
                    // Not enough room to apply hanging indent - retain original segment.
                    visible_width = display_width(&strip_ansi(&segment));
                } else {
                    segment = candidate;
                    visible_width = candidate_width;
                }
            }

            segments.push(WrappedCodeSegment {
                text: segment,
                visible_width,
            });
        }

        segments
    }

    pub(super) fn render_pretty_top_border(
        &self,
        inner_box_width: usize,
        label: Option<&str>,
    ) -> String {
        let mut line = String::from("╭");
        if inner_box_width <= 1 {
            return self.style_pretty_accent(&line);
        }

        let mut middle_width = inner_box_width.saturating_sub(2);

        if middle_width > 0 {
            line.push('─');
            middle_width = middle_width.saturating_sub(1);
        }

        if let Some(raw_label) = label
            && !raw_label.trim().is_empty()
            && middle_width > 0
        {
            line.push(' ');
            middle_width = middle_width.saturating_sub(1);

            if middle_width > 0 {
                let mut label_text = raw_label.to_string();
                if display_width(&label_text) > middle_width {
                    label_text = self.take_prefix_by_width(&label_text, middle_width).0;
                }

                let label_width = display_width(&label_text);
                if label_width > 0 && label_width <= middle_width {
                    line.push_str(&label_text);
                    middle_width = middle_width.saturating_sub(label_width);
                    if middle_width > 0 {
                        line.push(' ');
                        middle_width = middle_width.saturating_sub(1);
                    }
                } else {
                    // Not enough room for the label – remove the preceding space
                    if line.ends_with(' ') {
                        line.pop();
                        middle_width = middle_width.saturating_add(1);
                    }
                }
            }
        }

        while middle_width > 0 {
            line.push('─');
            middle_width = middle_width.saturating_sub(1);
        }

        line.push('╮');

        self.style_pretty_accent(&line)
    }

    pub(super) fn render_pretty_bottom_border(&self, inner_box_width: usize) -> String {
        let mut line = String::from("╰");
        if inner_box_width > 1 {
            let repeat = inner_box_width.saturating_sub(2);
            if repeat > 0 {
                line.push_str(&"─".repeat(repeat));
            }
            line.push('╯');
        } else {
            line.push('╯');
        }

        self.style_pretty_accent(&line)
    }

    pub(super) fn render_pretty_content_line(&self, text_width: usize, part: &str) -> String {
        let content_width = display_width(&strip_ansi(part));
        let inner_width = (1 + content_width).max(2);
        let mandatory_right_pad = inner_width - (1 + content_width);
        let trailing_pad = text_width.saturating_sub(inner_width);

        let mut line = String::new();
        line.push_str(&self.style_pretty_accent("│"));
        line.push(' ');
        line.push_str(part);
        if mandatory_right_pad > 0 {
            line.push_str(&" ".repeat(mandatory_right_pad));
        }
        if trailing_pad > 0 {
            line.push_str(&" ".repeat(trailing_pad));
        }
        line.push_str(&self.style_pretty_accent("│"));
        line
    }

    pub(super) fn style_pretty_accent(&self, text: &str) -> String {
        if self.config.no_colors {
            text.to_string()
        } else {
            AnsiStyle::new()
                .fg(PRETTY_ACCENT_COLOR)
                .apply(text, self.config.no_colors)
        }
    }
}
