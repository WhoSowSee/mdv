use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn is_markdown_language_hint(hint: Option<&str>) -> bool {
        let Some(raw) = hint else {
            return false;
        };
        let normalized = raw.trim().to_ascii_lowercase();
        matches!(normalized.as_str(), "md" | "markdown")
    }

    pub(super) fn should_render_code_block_as_plaintext(
        &self,
        language_hint: Option<&str>,
    ) -> bool {
        if self.plaintext_code_block_depth > 0 {
            return false;
        }

        let hint = match language_hint {
            Some(raw) => raw.trim(),
            None => return false,
        };

        if hint.is_empty() {
            return false;
        }

        let normalized = hint.to_ascii_lowercase();
        matches!(
            normalized.as_str(),
            "text" | "plain" | "plaintext" | "txt" | "markdown" | "md"
        )
    }

    pub(super) fn render_plaintext_code_block(&self, code: &str) -> Result<PlaintextRenderResult> {
        let mut nested_config = self.config.clone();
        nested_config.from_text = None;
        nested_config.margin = crate::cli::HorizontalMargins::default();
        nested_config.line_numbers = None;
        nested_config.code_line_numbers = None;
        nested_config.code_line_number_width = 0;
        nested_config.line_number_gutter_width = 0;

        if let Some(width) = self.estimate_plaintext_block_width() {
            nested_config.cols = Some(width);
            nested_config.cols_from_cli = true;
        }

        let processor = MarkdownProcessor::new(&nested_config);
        let events = processor.parse(code)?;

        let mut nested_renderer =
            EventRenderer::new(&nested_config, self.theme, self.syntax_set, self.code_theme);
        nested_renderer.plaintext_code_block_depth = self.plaintext_code_block_depth + 1;
        nested_renderer.suppress_footnote_output = true;
        if matches!(self.config.link_style, LinkStyle::EndTable) {
            nested_renderer.paragraph_link_counter = self.paragraph_link_counter;
        }

        let mut rendered = nested_renderer.render_events(events)?;
        rendered = rendered
            .trim_end_matches('\n')
            .trim_start_matches('\n')
            .to_string();

        let references = std::mem::take(&mut nested_renderer.captured_reference_blocks);
        let deferred_references = std::mem::take(&mut nested_renderer.deferred_reference_blocks);
        let document_links = std::mem::take(&mut nested_renderer.document_links);
        let reference_counter = nested_renderer.paragraph_link_counter;

        Ok(PlaintextRenderResult {
            body: rendered,
            references,
            deferred_references,
            document_links,
            reference_counter,
        })
    }

    pub(super) fn estimate_plaintext_block_width(&self) -> Option<usize> {
        let terminal_width = self.config.get_content_width();
        if terminal_width == 0 {
            return None;
        }

        let context_width = self.compute_code_block_context_width();
        let available = terminal_width.saturating_sub(context_width);
        if available == 0 {
            return None;
        }

        let width = match self.config.code_block_style.style {
            CodeBlockStyle::Basic => available.saturating_sub(BASIC_CODE_BLOCK_INDENT),
            CodeBlockStyle::Simple => available.saturating_sub(2),
            CodeBlockStyle::Pretty => {
                let left_padding = 1usize;
                let right_padding = 1usize;

                if available <= 4 {
                    // Frame too tight, pretty style will fall back to simple.
                    available.saturating_sub(2)
                } else {
                    let max_inner_box_width = available;
                    let max_text_width_allowed = max_inner_box_width.saturating_sub(2);
                    if max_text_width_allowed < left_padding + right_padding + 1 {
                        // Not enough room for pretty box content, fall back to simple width.
                        available.saturating_sub(2)
                    } else {
                        let wrap_width_allowed =
                            max_text_width_allowed.saturating_sub(left_padding + right_padding);
                        if wrap_width_allowed == 0 {
                            available.saturating_sub(2)
                        } else {
                            wrap_width_allowed
                        }
                    }
                }
            }
        };

        let sanitized = width.max(1);
        Some(sanitized)
    }

    pub(super) fn embed_captured_reference_blocks_in_code_body(
        mut body: String,
        blocks: Vec<CapturedReferenceBlock>,
    ) -> String {
        for block in blocks {
            if !body.is_empty() && !body.ends_with('\n') {
                body.push('\n');
            }
            body.push('\n');

            for (idx, line) in block.lines.into_iter().enumerate() {
                if idx > 0 {
                    body.push('\n');
                }
                body.push_str(&line);
            }

            if block.add_trailing_newline {
                body.push('\n');
            }
        }

        body.trim_end_matches('\n').to_string()
    }
}
