use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn process_event(&mut self, event: Event) -> Result<()> {
        if let Some(marker) = crate::markdown::source_line_from_event(&event) {
            match marker {
                crate::markdown::SourceLineMarker::Content(source_line) => {
                    self.push_source_line_marker(source_line);
                }
                crate::markdown::SourceLineMarker::Blank(source_line) => {
                    self.flush_pending_html_block_buffer()?;
                    self.push_source_blank_line_marker(source_line);
                }
            }
            return Ok(());
        }

        if !matches!(event, Event::Text(_)) {
            self.reset_footnote_text_scan();
        }
        if self.config.render_html
            && self.pending_html_block_buffer.is_some()
            && !matches!(event, Event::Html(_) | Event::InlineHtml(_))
            && !(self.pending_html_buffer_captures_markdown_events()
                && matches!(
                    event,
                    Event::Text(_) | Event::Code(_) | Event::SoftBreak | Event::HardBreak
                ))
        {
            self.flush_pending_html_block_buffer()?;
        }
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag_end) => self.handle_end_tag(tag_end)?,
            Event::Text(text) => {
                if self.append_pending_html_buffer_text(text.as_ref()) {
                    return Ok(());
                }
                self.handle_text(text)?;
            }
            Event::Code(code) => {
                if self.append_pending_html_buffer_text(code.as_ref()) {
                    return Ok(());
                }
                self.handle_inline_code(code)?;
            }
            Event::Html(html) => self.handle_html(html)?,
            Event::InlineHtml(html) => self.handle_inline_html(html)?,
            Event::SoftBreak => {
                if self.append_pending_html_buffer_soft_break() {
                    return Ok(());
                }
                self.handle_soft_break()?;
            }
            Event::HardBreak => {
                if self.append_pending_html_buffer_hard_break() {
                    return Ok(());
                }
                if self.finalize_pending_callout_label_override() {
                    return Ok(());
                }
                if self.current_paragraph_start.is_some() && !self.current_paragraph_has_content {
                    self.current_paragraph_has_leading_break = true;
                    if let Some(start) = self.current_paragraph_start
                        && start <= self.output.len()
                    {
                        self.output.truncate(start);
                    }
                } else {
                    self.handle_hard_break();
                }
            }
            Event::Rule => self.handle_horizontal_rule()?,
            Event::FootnoteReference(name) => self.handle_footnote_reference(name)?,
            Event::TaskListMarker(checked) => self.handle_task_list_marker(checked)?,
            Event::InlineMath(math) => self.handle_inline_math(math)?,
            Event::DisplayMath(math) => self.handle_display_math(math)?,
        }
        Ok(())
    }

    pub(super) fn push_source_line_marker(&mut self, source_line: usize) {
        let marker = crate::renderer::line_numbers::encode_internal_marker(source_line);
        if self.in_code_block {
            self.code_block_content.push_str(&marker);
        } else if self.in_link {
            self.current_link_text.push_str(&marker);
        } else if let Some(table) = self.table_state.as_mut() {
            table.current_cell.push_str(&marker);
        } else if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str(&marker);
        } else {
            self.output.push_str(&marker);
        }
    }

    pub(super) fn push_source_blank_line_marker(&mut self, source_line: usize) {
        if !self.output.ends_with('\n') {
            return;
        }

        let line_end = self.output.len().saturating_sub(1);
        let line_start = self.output[..line_end]
            .rfind('\n')
            .map_or(0, |idx| idx.saturating_add(1));
        let visible_line = strip_ansi(&self.output[line_start..line_end]);
        let has_content = visible_line
            .chars()
            .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃');
        if has_content {
            return;
        }

        let marker = crate::renderer::line_numbers::encode_internal_marker(source_line);
        self.output.insert_str(line_start, &marker);
    }
}
