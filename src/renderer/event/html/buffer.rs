use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn render_html_fragment_buffering_blocks(
        &mut self,
        html: &str,
    ) -> Result<()> {
        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str(html);
            let complete = match buffer.kind {
                HtmlBlockKind::Element(tag) => {
                    contains_html_tag_outside_comments(html, tag, true, &mut buffer.comment_open)
                }
                HtmlBlockKind::Comment => html.contains("-->"),
            };
            if complete {
                self.flush_pending_html_block_buffer()?;
            }
            return Ok(());
        }

        let trimmed = html.trim_start();
        if trimmed.starts_with("<!--") {
            if let Some(end) = trimmed.find("-->") {
                if trimmed[end + 3..].trim().is_empty() {
                    return self.render_html_comment(html);
                }
            } else {
                self.pending_html_block_buffer = Some(HtmlBlockBuffer {
                    kind: HtmlBlockKind::Comment,
                    content: html.to_string(),
                    captures_markdown_events: false,
                    details_events: Vec::new(),
                    details_depth: 0,
                    comment_open: true,
                });
                return Ok(());
            }
        }

        let mut comment_open = false;
        let details_depth = html_details_balance(html, 0, &mut comment_open);
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
                kind: HtmlBlockKind::Element(tag),
                content: html.to_string(),
                captures_markdown_events,
                details_events: Vec::new(),
                details_depth,
                comment_open,
            });
            return Ok(());
        }

        self.render_html_fragment_as_terminal(html)
    }

    pub(in crate::renderer::event) fn flush_pending_html_block_buffer(&mut self) -> Result<()> {
        let Some(mut buffer) = self.pending_html_block_buffer.take() else {
            return Ok(());
        };

        if buffer.kind == HtmlBlockKind::Comment {
            if let Some(end) = buffer.content.find("-->") {
                let (comment, remainder) = buffer.content.split_at(end + 3);
                if !remainder.trim().is_empty() {
                    let last_line = comment
                        .rsplit('\n')
                        .next()
                        .expect("comment has a closing delimiter");
                    let (_, source_line) =
                        crate::renderer::line_numbers::strip_internal_markers(last_line);
                    self.render_html_comment(comment)?;
                    self.pending_html_source_line = source_line;
                    return self.render_html_fragment_buffering_blocks(remainder);
                }
            }
            return self.render_html_comment(&buffer.content);
        }

        pulldown_cmark::html::push_html(&mut buffer.content, buffer.details_events.into_iter());
        if buffer.content.trim().is_empty() {
            return Ok(());
        }

        self.render_html_fragment_as_terminal(&buffer.content)
    }

    pub(super) fn render_html_comment(&mut self, html: &str) -> Result<()> {
        let source_line = self.pending_html_source_line.take();
        if self.config.hide_comments {
            return Ok(());
        }
        if let Some(source_line) = source_line {
            let marker = crate::renderer::line_numbers::encode_internal_marker(source_line);
            return self.render_literal_html(&format!("{marker}{html}"));
        }
        self.render_literal_html(html)
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
