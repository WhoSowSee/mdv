use super::super::headings::{heading_marker_count, heading_theme_element};
use super::super::tables::HTML_TABLE_HORIZONTAL_RULE;
use super::{ElementRef, EventRenderer, HeadingLevel, HtmlContext, Result};

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_table_horizontal_rule(&mut self, context: HtmlContext) {
        self.begin_html_table_cell_line(context.list_depth * 2);
        if let Some(ref mut table) = self.table_state {
            table.current_cell.push_str(HTML_TABLE_HORIZONTAL_RULE);
        }
        self.end_html_table_cell_line();
    }

    pub(super) fn render_html_table_block(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.begin_html_table_cell_line(context.list_depth * 2);
        let result = self.render_html_children(element, context);
        if result.is_ok() {
            self.end_html_table_cell_line();
        }
        result
    }

    pub(super) fn render_html_table_heading(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        level: HeadingLevel,
    ) -> Result<()> {
        self.begin_html_table_cell_line(context.list_depth * 2);
        let stack_len = self.formatting_stack.len();
        self.formatting_stack.push(heading_theme_element(level));
        if self.config.show_heading_markers {
            self.render_html_inline_literal(&format!(
                "{} ",
                "#".repeat(heading_marker_count(level))
            ))?;
        }
        let result = self.render_html_children(element, context);
        self.formatting_stack.truncate(stack_len);
        if result.is_ok() {
            self.end_html_table_cell_line();
        }
        result
    }

    pub(super) fn render_html_table_blockquote(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        let content = self.capture_html_table_children(element, context.without_list_depth())?;
        let content = content.trim_end_matches('\n');
        if content.is_empty() {
            return Ok(());
        }

        let prefix = self.render_blockquote_prefix_for_level(1);
        for line in content.lines() {
            self.begin_html_table_cell_line(context.list_depth * 2);
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(&prefix);
                table.current_cell.push_str(line);
            }
            self.end_html_table_cell_line();
        }
        Ok(())
    }

    pub(super) fn render_html_table_figure(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.begin_html_table_cell_line(context.list_depth * 2);
        self.render_html_figure_children(element, context)?;
        self.end_html_table_cell_line();
        Ok(())
    }

    pub(super) fn render_html_table_preformatted_block(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        text: String,
    ) -> Result<()> {
        let text = text.replace('\t', &" ".repeat(self.config.tab_length));
        let language = html_preformatted_language(element);
        let mut renderer =
            EventRenderer::new(self.config, self.theme, self.syntax_set, self.code_theme);
        renderer.code_block_content = text;
        renderer.code_block_language = language;
        renderer.handle_code_block_end()?;

        let rendered = renderer.output.trim_matches('\n');
        if rendered.is_empty() {
            return Ok(());
        }

        self.begin_html_table_cell_line(context.list_depth * 2);
        if let Some(ref mut table) = self.table_state {
            table.current_cell.push_str(rendered);
        }
        self.end_html_table_cell_line();
        Ok(())
    }

    fn capture_html_table_children(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<String> {
        let previous = self
            .table_state
            .as_mut()
            .map(|table| std::mem::take(&mut table.current_cell))
            .unwrap_or_default();
        let result = self.render_html_children(element, context);
        let captured = self
            .table_state
            .as_mut()
            .map(|table| std::mem::take(&mut table.current_cell))
            .unwrap_or_default();
        if let Some(ref mut table) = self.table_state {
            table.current_cell = previous;
        }
        result?;
        Ok(captured)
    }
}

fn html_preformatted_language(element: ElementRef<'_>) -> Option<String> {
    std::iter::once(element)
        .chain(
            element
                .child_elements()
                .filter(|child| child.value().name().eq_ignore_ascii_case("code")),
        )
        .find_map(html_language_hint)
}

fn html_language_hint(element: ElementRef<'_>) -> Option<String> {
    for attribute in ["data-language", "data-lang", "lang"] {
        if let Some(language) = element.attr(attribute).map(str::trim)
            && !language.is_empty()
        {
            return Some(language.to_string());
        }
    }

    element.attr("class").and_then(|classes| {
        classes.split_ascii_whitespace().find_map(|class| {
            ["language-", "lang-"]
                .iter()
                .find_map(|prefix| class.strip_prefix(prefix))
                .filter(|language| !language.is_empty())
                .map(str::to_string)
        })
    })
}
