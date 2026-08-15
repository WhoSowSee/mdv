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

        if let Some(tag) = buffering_html_container_tag(html)
            && !contains_html_tag(html, tag, true)
        {
            self.pending_html_block_buffer = Some(HtmlBlockBuffer {
                tag,
                content: html.to_string(),
                captures_markdown_events: self.table_state.is_some(),
            });
            return Ok(());
        }

        if let Some(tag) = buffering_inline_html_container_tag(html)
            && !contains_html_tag(html, tag, true)
        {
            self.pending_html_block_buffer = Some(HtmlBlockBuffer {
                tag,
                content: html.to_string(),
                captures_markdown_events: true,
            });
            return Ok(());
        }

        self.render_html_fragment_as_terminal(html)
    }

    pub(in crate::renderer::event) fn flush_pending_html_block_buffer(&mut self) -> Result<()> {
        let Some(buffer) = self.pending_html_block_buffer.take() else {
            return Ok(());
        };

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
        self.commit_pending_heading_placeholder_if_content();
        self.flush_html_inline_table_references();
        Ok(())
    }
}
