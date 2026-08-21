use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn render_code_block_basic(
        &mut self,
        input: CodeBlockRenderInput<'_>,
    ) -> Result<()> {
        let indent = " ".repeat(BASIC_CODE_BLOCK_INDENT);
        let context_width = self.compute_code_block_context_width();
        let available = input
            .terminal_width
            .saturating_sub(context_width + BASIC_CODE_BLOCK_INDENT);

        if let Some(label) = input.language_label {
            let base_label = if label.trim().is_empty() {
                "Text"
            } else {
                label
            };
            let wrapped_label = if input.should_wrap && available > 0 {
                crate::utils::wrap_text_with_mode(base_label, available, input.wrap_mode)
            } else {
                base_label.to_string()
            };

            for part in wrapped_label.split('\n') {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&indent);
                self.output.push_str(&self.style_pretty_accent(part));
                self.output.push('\n');
            }

            if !input.code_starts_with_blank {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&indent);
                self.output.push('\n');
            }
        }

        let layout = self.layout_code_lines(input, available, false);
        self.record_code_line_number_width(&layout);
        for line in &layout.lines {
            self.push_code_block_indent_for_line_start();
            self.output.push_str(&indent);
            let decorated = self.highlight_footnote_markers_in_ansi(&line.text);
            let rendered = self.format_code_line(&layout, line, &decorated);
            self.output.push_str(&rendered);
            self.output.push('\n');
        }

        Ok(())
    }

    pub(in crate::renderer::event) fn render_code_block_simple(
        &mut self,
        input: CodeBlockRenderInput<'_>,
    ) -> Result<()> {
        let prefix = self.render_code_block_border();
        if let Some(label) = input.language_label {
            let base_label = if label.trim().is_empty() {
                "Text"
            } else {
                label
            };

            let context_width = self.compute_code_block_context_width();
            let border_visible_width = display_width(&strip_ansi(&prefix));
            let available_width = input
                .terminal_width
                .saturating_sub(context_width + border_visible_width);

            let wrapped_label = if input.should_wrap && available_width > 0 {
                crate::utils::wrap_text_with_mode(base_label, available_width, input.wrap_mode)
            } else {
                base_label.to_string()
            };

            for part in wrapped_label.split('\n') {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&prefix);
                self.output.push_str(&self.style_pretty_accent(part));
                self.output.push('\n');
            }

            if !input.code_starts_with_blank {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&prefix);
                self.output.push('\n');
            }
        }

        let context_width = self.compute_code_block_context_width();
        let border_visible_width = 2usize;
        let available = input
            .terminal_width
            .saturating_sub(context_width + border_visible_width);
        let layout = self.layout_code_lines(input, available, false);
        self.record_code_line_number_width(&layout);

        for line in &layout.lines {
            self.push_code_block_indent_for_line_start();
            self.output.push_str(&prefix);
            let decorated = self.highlight_footnote_markers_in_ansi(&line.text);
            let rendered = self.format_code_line(&layout, line, &decorated);
            self.output.push_str(&rendered);
            self.output.push('\n');
        }

        Ok(())
    }
}
