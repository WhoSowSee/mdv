use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn render_html_fragment_buffering_blocks(
        &mut self,
        html: &str,
    ) -> Result<()> {
        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str(html);
            if contains_html_tag(html, buffer.tag, true) {
                self.flush_pending_html_block_buffer()?;
            }
            return Ok(());
        }

        let details_depth = html_details_balance(html, 0);
        let container = if details_depth > 0 {
            Some("details")
        } else {
            buffering_html_container_tag(html)
        };
        let pending = container
            .filter(|tag| details_depth > 0 || !contains_html_tag(html, tag, true))
            .map(|tag| (tag, self.table_state.is_some()))
            .or_else(|| {
                buffering_inline_html_container_tag(html)
                    .filter(|tag| !contains_html_tag(html, tag, true))
                    .map(|tag| (tag, true))
            });
        if let Some((tag, captures_markdown_events)) = pending {
            self.pending_html_block_buffer = Some(HtmlBlockBuffer {
                tag,
                content: html.to_string(),
                captures_markdown_events,
                details_events: Vec::new(),
                details_depth,
            });
            return Ok(());
        }

        self.render_html_fragment_as_terminal(html)
    }

    pub(in crate::renderer::event) fn flush_pending_html_block_buffer(&mut self) -> Result<()> {
        let Some(mut buffer) = self.pending_html_block_buffer.take() else {
            return Ok(());
        };

        pulldown_cmark::html::push_html(&mut buffer.content, buffer.details_events.into_iter());
        if buffer.content.trim().is_empty() {
            return Ok(());
        }

        self.render_html_fragment_as_terminal(&buffer.content)
    }

    pub(in crate::renderer::event) fn pending_html_buffer_captures_markdown_events(&self) -> bool {
        self.pending_html_block_buffer
            .as_ref()
            .map(|buffer| buffer.captures_markdown_events)
            .unwrap_or(false)
    }

    pub(in crate::renderer::event) fn append_pending_html_buffer_text(
        &mut self,
        text: &str,
    ) -> bool {
        if !self.pending_html_buffer_captures_markdown_events() {
            return false;
        }

        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str(&escape_html_text(text));
        }
        true
    }

    pub(in crate::renderer::event) fn append_pending_html_buffer_soft_break(&mut self) -> bool {
        if !self.pending_html_buffer_captures_markdown_events() {
            return false;
        }

        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push('\n');
        }
        true
    }

    pub(in crate::renderer::event) fn append_pending_html_buffer_hard_break(&mut self) -> bool {
        if !self.pending_html_buffer_captures_markdown_events() {
            return false;
        }

        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str("<br>");
        }
        true
    }

    pub(in crate::renderer::event) fn render_html_fragment_as_terminal(
        &mut self,
        html: &str,
    ) -> Result<()> {
        let fragment = Html::parse_fragment(html);
        for node in fragment.tree.root().children() {
            self.render_html_node(node, HtmlContext::default())?;
        }
        self.pending_html_source_line = None;
        self.commit_pending_heading_placeholder_if_content();
        self.flush_html_inline_table_references();
        Ok(())
    }
}
