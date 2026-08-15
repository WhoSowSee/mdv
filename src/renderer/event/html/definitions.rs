use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_definition_list(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_children(element, context);
        }

        self.begin_html_block();
        for child in element.children() {
            self.render_html_node(child, context)?;
        }
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    pub(super) fn render_html_definition_term(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.render_html_styled_block_line(element, context, &[ThemeElement::Strong])
    }

    pub(super) fn render_html_definition_description(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            self.begin_html_table_cell_line(context.list_depth * 2 + 2);
            return self.render_html_definition_description_children(element, context);
        }

        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.push_indent_for_line_start();
        let content_start = self.output.len();
        self.note_paragraph_content();
        self.render_html_definition_description_children(element, context)?;
        self.indent_rendered_html_span(content_start, 2);
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        Ok(())
    }

    pub(super) fn render_html_definition_description_children(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && is_definition_description_inline_block(child_element.value().name())
            {
                self.render_html_children(child_element, context)?;
                continue;
            }
            self.render_html_node(child, context)?;
        }
        Ok(())
    }

    pub(super) fn render_html_figure(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_table_figure(element, context);
        }

        self.begin_html_block();
        let content_start = self.output.len();
        self.render_html_figure_children(element, context)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    pub(super) fn render_html_figure_children(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && child_element
                    .value()
                    .name()
                    .eq_ignore_ascii_case("figcaption")
            {
                continue;
            }
            self.render_html_node(child, context)?;
        }
        for caption in element
            .child_elements()
            .filter(|child| child.value().name().eq_ignore_ascii_case("figcaption"))
        {
            self.render_html_figcaption(caption, context)?;
        }
        Ok(())
    }

    pub(super) fn render_html_figcaption(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.render_html_styled_block_line(
            element,
            context,
            &[ThemeElement::TextLight, ThemeElement::Emphasis],
        )
    }
}
