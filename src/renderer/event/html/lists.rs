use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_list(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        ordered: bool,
    ) -> Result<()> {
        let mut marker_state = html_list_marker_state(&element, ordered);
        let in_table = self.table_state.is_some();
        if !in_table {
            self.begin_html_block();
        }
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && child_element.value().name().eq_ignore_ascii_case("li")
            {
                let marker = marker_state.next_marker(child_element);
                let pretty_level = (!ordered).then_some(context.list_depth.saturating_add(1));
                self.render_html_list_item(child_element, context, &marker, pretty_level)?;
                continue;
            }
            self.render_html_node(child, context)?;
        }
        if !in_table {
            self.end_html_block();
            self.flush_html_inline_table_references();
        }
        Ok(())
    }

    pub(super) fn render_html_list_item(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        marker: &str,
        pretty_level: Option<usize>,
    ) -> Result<()> {
        let marker = if html_list_item_starts_with_checkbox(&element) {
            String::new()
        } else {
            self.styled_list_marker(marker, pretty_level)
        };
        let child_context = context.in_nested_list();

        if self.table_state.is_some() {
            self.begin_html_table_cell_line(context.list_depth * 2);
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(&marker);
            }
            self.render_html_children(element, child_context)?;
            return Ok(());
        }

        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.push_indent_for_line_start();
        self.output.push_str(&marker);
        self.note_paragraph_content();
        self.render_html_children(element, child_context)?;
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        Ok(())
    }

    pub(super) fn render_html_details(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_children(element, context);
        }

        self.begin_html_block();
        let content_start = self.output.len();
        let mut rendered_summary = false;
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && child_element.value().name().eq_ignore_ascii_case("summary")
                && !rendered_summary
            {
                self.render_html_summary_label(child_element, context)?;
                rendered_summary = true;
                continue;
            }
            self.render_html_node(child, context)?;
        }
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    pub(super) fn render_html_summary_label(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.render_html_styled_block_line(element, context, &[ThemeElement::Strong])
    }

    pub(super) fn render_html_styled_block_line(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        styles: &[ThemeElement],
    ) -> Result<()> {
        let in_table = self.table_state.is_some();
        if in_table {
            self.begin_html_table_cell_line(context.list_depth * 2);
        } else {
            if !self.output.is_empty() && !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            self.push_indent_for_line_start();
        }

        let stack_len = self.formatting_stack.len();
        self.formatting_stack.extend_from_slice(styles);
        let result = self.render_html_children(element, context);
        self.formatting_stack.truncate(stack_len);
        if result.is_ok() {
            if in_table {
                self.end_html_table_cell_line();
            } else if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
        }
        result
    }
}
