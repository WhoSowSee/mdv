use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_block(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_table_block(element, context);
        }

        self.begin_html_block();
        let content_start = self.output.len();
        self.render_html_children(element, context)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    pub(super) fn render_html_heading(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        level: HeadingLevel,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_table_heading(element, context, level);
        }

        self.handle_header_start(level)?;
        let content_start = self.output.len();
        self.render_html_children(element, context)?;
        self.handle_header_end(level)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.flush_html_inline_table_references();
        Ok(())
    }

    pub(super) fn render_html_link(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        let Some(href) = element.attr("href").filter(|href| !href.trim().is_empty()) else {
            return self.render_html_children(element, context);
        };

        self.handle_link_start(CowStr::from(href.to_string()))?;
        self.render_html_children(element, context)?;
        self.handle_link_end()
    }

    pub(super) fn render_html_with_formatting(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        theme_element: ThemeElement,
    ) -> Result<()> {
        self.close_inline_backticks();
        self.formatting_stack.push(theme_element);
        let result = self.render_html_children(element, context);
        self.close_inline_backticks();
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == theme_element)
        {
            self.formatting_stack.remove(index);
        }
        result
    }

    pub(super) fn render_html_code_like(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        prefix: &str,
        suffix: &str,
    ) -> Result<()> {
        self.close_inline_backticks();
        self.formatting_stack.push(ThemeElement::Code);
        let code_context = context.with_preserve_whitespace();
        let result = self
            .render_html_inline_literal(prefix)
            .and_then(|()| self.render_html_children(element, code_context))
            .and_then(|()| self.render_html_inline_literal(suffix));
        self.close_inline_backticks();
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == ThemeElement::Code)
        {
            self.formatting_stack.remove(index);
        }
        result
    }

    pub(super) fn render_html_abbr(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.render_html_with_formatting(element, context, ThemeElement::TextLight)?;
        if let Some(title) = element
            .attr("title")
            .map(str::trim)
            .filter(|title| !title.is_empty())
        {
            self.render_html_text(&format!(" ({title})"), context)?;
        }
        Ok(())
    }

    pub(super) fn render_html_inline_literal(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        self.note_paragraph_content();
        self.process_segment_with_wrapping_and_formatting(text, false, self.table_state.is_some())
    }

    pub(super) fn render_html_preformatted_block(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        let text = normalize_preformatted_html_text(&element.text().collect::<String>());
        if self.table_state.is_some() {
            return self.render_html_table_preformatted_block(element, context, text);
        }

        self.begin_html_block();
        let content_start = self.output.len();
        self.render_html_preformatted_text(&text, context)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        Ok(())
    }

    pub(super) fn render_html_preformatted_text(
        &mut self,
        text: &str,
        context: HtmlContext,
    ) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        self.note_paragraph_content();
        for (line_index, line) in text.split('\n').enumerate() {
            if line_index > 0 {
                self.output.push('\n');
            }
            if self.output.is_empty() || self.output.ends_with('\n') {
                self.push_indent_for_line_start();
            }
            let formatted = self.apply_formatting_with_highlight(line, context.highlighted);
            self.output.push_str(&formatted);
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
