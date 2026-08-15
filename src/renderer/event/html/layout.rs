use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn begin_html_block(&mut self) {
        if self.output.is_empty() {
            return;
        }
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    pub(super) fn end_html_block(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    pub(super) fn begin_html_table_cell_line(&mut self, indent: usize) {
        self.end_html_table_cell_line();
        if let Some(ref mut table) = self.table_state {
            table.current_cell.extend(std::iter::repeat_n(' ', indent));
        }
    }

    pub(super) fn end_html_table_cell_line(&mut self) {
        if let Some(ref mut table) = self.table_state {
            while matches!(table.current_cell.chars().next_back(), Some(' ' | '\t')) {
                table.current_cell.pop();
            }
            if !table.current_cell.is_empty() && !table.current_cell.ends_with('\n') {
                table.current_cell.push('\n');
            }
        }
    }

    pub(super) fn push_html_inline_style_elements(&mut self, element: &ElementRef<'_>) {
        for theme_element in html_inline_style_elements(element) {
            if !self.formatting_stack.contains(&theme_element) {
                self.formatting_stack.push(theme_element);
            }
        }
    }

    pub(super) fn flush_html_inline_table_references(&mut self) {
        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && self.table_state.is_none()
            && self.current_paragraph_start.is_none()
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references();
            self.ensure_contextual_blank_line();
        }
    }

    pub(super) fn align_rendered_html_span(&mut self, start: usize, alignment: HtmlAlignment) {
        if matches!(alignment, HtmlAlignment::Left) {
            return;
        }
        if start >= self.output.len() {
            return;
        }

        let prefix = self.current_line_prefix();
        let span = self.output[start..].to_string();
        let mut aligned = String::new();
        let mut lines = span.split('\n').peekable();

        while let Some(line) = lines.next() {
            let has_more = lines.peek().is_some();
            aligned.push_str(&self.align_html_line(line, &prefix, alignment));
            if has_more {
                aligned.push('\n');
            }
        }

        self.output.replace_range(start.., &aligned);
    }

    pub(super) fn indent_rendered_html_span(&mut self, start: usize, indent: usize) {
        if start >= self.output.len() || indent == 0 {
            return;
        }

        let prefix = self.current_line_prefix();
        let span = self.output[start..].to_string();
        let mut indented = String::new();
        let mut lines = span.split('\n').peekable();

        while let Some(line) = lines.next() {
            let has_more = lines.peek().is_some();
            if strip_ansi(line).trim().is_empty() {
                indented.push_str(line);
            } else if !prefix.is_empty() && line.starts_with(&prefix) {
                indented.push_str(&prefix);
                indented.push_str(&" ".repeat(indent));
                indented.push_str(&line[prefix.len()..]);
            } else {
                indented.push_str(&" ".repeat(indent));
                indented.push_str(line);
            }

            if has_more {
                indented.push('\n');
            }
        }

        self.output.replace_range(start.., &indented);
    }

    pub(super) fn align_html_line(
        &self,
        line: &str,
        prefix: &str,
        alignment: HtmlAlignment,
    ) -> String {
        if strip_ansi(line).trim().is_empty() {
            return line.to_string();
        }

        let (line_prefix, content) = if !prefix.is_empty() && line.starts_with(prefix) {
            (prefix, &line[prefix.len()..])
        } else {
            ("", line)
        };
        let content = content.trim();
        let content_width = display_width(&strip_ansi(content));
        let prefix_width = display_width(&strip_ansi(line_prefix));
        let available_width = self.effective_text_width().saturating_sub(prefix_width);
        let padding = match alignment {
            HtmlAlignment::Left => 0,
            HtmlAlignment::Center => available_width.saturating_sub(content_width) / 2,
            HtmlAlignment::Right => available_width.saturating_sub(content_width),
        };

        format!("{line_prefix}{}{content}", " ".repeat(padding))
    }
}
