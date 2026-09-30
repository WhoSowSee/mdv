use super::*;
use pulldown_cmark::Event;

pub(super) fn html_details_balance(html: &str, mut depth: usize) -> usize {
    static TAGS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let tags = TAGS.get_or_init(|| {
        regex::Regex::new(r#"(?is)<!--.*?-->|</?details(?:\s+(?:[^>\"']|\"[^\"]*\"|'[^']*')*)?/?>"#)
            .expect("valid details tag pattern")
    });
    for tag in tags.find_iter(html).map(|tag| tag.as_str()) {
        if tag.starts_with("<!--") {
            continue;
        }
        if tag.starts_with("</") {
            depth = depth.saturating_sub(1);
        } else {
            depth += 1;
        }
    }
    depth
}

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn buffer_html_details_event<'e>(
        &mut self,
        event: Event<'e>,
    ) -> Result<Option<Event<'e>>> {
        let Some(buffer) = self
            .pending_html_block_buffer
            .as_mut()
            .filter(|buffer| buffer.tag == "details")
        else {
            return Ok(Some(event));
        };
        if let Some(marker) = crate::markdown::source_line_from_event(&event) {
            if let crate::markdown::SourceLineMarker::Content(line) = marker {
                buffer.details_events.push(Event::Html(
                    crate::renderer::line_numbers::encode_internal_marker(line).into(),
                ));
            }
            return Ok(None);
        }
        if let Event::Html(html) | Event::InlineHtml(html) = &event {
            if html.trim() == crate::markdown::BLANK_LINE_MARKER {
                return Ok(None);
            }
            buffer.details_depth = html_details_balance(html, buffer.details_depth);
        }
        buffer.details_events.push(event.into_static());
        if buffer.details_depth == 0 {
            self.flush_pending_html_block_buffer()?;
        }
        Ok(None)
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
        let summary = element
            .child_elements()
            .find(|child| child.value().name().eq_ignore_ascii_case("summary"));
        if let Some(summary) = summary {
            self.push_indent_for_line_start();
            let marker =
                create_style(self.theme, ThemeElement::Code).apply(" ", self.output_style);
            self.output.push_str(&marker);
            let stack_len = self.formatting_stack.len();
            self.formatting_stack
                .extend([ThemeElement::Strong, ThemeElement::Code]);
            let result = self.render_html_children(summary, context);
            self.close_inline_backticks();
            self.formatting_stack.truncate(stack_len);
            result?;
            self.end_html_block();
        }

        self.html_details_depth += 1;
        let result = self.render_html_details_body(element, summary, context);
        self.html_details_depth -= 1;
        result?;
        self.rebase_html_trailing_spacing();
        self.end_html_block();
        Ok(())
    }

    fn render_html_details_body(
        &mut self,
        element: ElementRef<'_>,
        summary: Option<ElementRef<'_>>,
        context: HtmlContext,
    ) -> Result<()> {
        for child in element.children() {
            if summary.is_some_and(|summary| summary.id() == child.id()) {
                continue;
            }
            self.render_html_node(child, context)?;
        }
        self.flush_html_inline_table_references();
        Ok(())
    }
}
