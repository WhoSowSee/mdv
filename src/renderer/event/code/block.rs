use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_code_block_end(&mut self) -> Result<()> {
        self.in_code_block = false;

        self.reset_explicit_blank_line_streak();

        let mut raw_code = std::mem::take(&mut self.code_block_content);
        let language_hint = self.code_block_language.clone();
        if Self::is_markdown_language_hint(language_hint.as_deref()) {
            let (cleaned, definitions) = self.extract_markdown_code_footnote_definitions(&raw_code);
            if !definitions.is_empty() {
                self.footnote_definitions.extend(definitions);
            }
            raw_code = cleaned;
        }
        self.register_footnotes_in_text(&raw_code);

        let is_empty = raw_code.trim().is_empty();
        if is_empty && !self.config.show_empty_elements {
            self.code_block_language = None;
            return Ok(());
        }

        if let Some(hint) = language_hint.as_deref()
            && is_math_language_hint(hint)
        {
            self.code_block_language = None;
            return self.handle_math_code_block(&raw_code, language_hint.as_deref());
        }
        let treat_as_plaintext =
            self.should_render_code_block_as_plaintext(language_hint.as_deref());
        let (
            mut highlighted,
            captured_reference_blocks,
            deferred_reference_blocks,
            collected_document_links,
            reference_counter,
        ) = if treat_as_plaintext {
            let PlaintextRenderResult {
                body,
                references,
                deferred_references,
                document_links,
                reference_counter,
            } = self.render_plaintext_code_block(&raw_code)?;
            (
                body,
                references,
                deferred_references,
                document_links,
                reference_counter,
            )
        } else {
            (
                self.highlight_code(&raw_code, language_hint.as_deref())?,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                self.paragraph_link_counter,
            )
        };

        if matches!(self.config.link_style, LinkStyle::EndTable) {
            if !collected_document_links.is_empty() {
                self.document_links.extend(collected_document_links);
            }
            self.paragraph_link_counter = reference_counter;
        }

        if !captured_reference_blocks.is_empty() {
            highlighted = Self::embed_captured_reference_blocks_in_code_body(
                highlighted,
                captured_reference_blocks,
            );
        }

        let highlighted_is_empty = strip_ansi(&highlighted).trim().is_empty();
        if highlighted_is_empty {
            if !self.config.show_empty_elements {
                self.code_block_language = None;
                return Ok(());
            }

            if highlighted.is_empty() {
                highlighted.push('\n');
            }
        }

        let code_starts_with_blank = raw_code.starts_with('\n');

        let language_label =
            if self.config.code_block_style.show_name || self.config.code_block_style.show_icon {
                let (base_label, hint_key) = match language_hint.as_deref() {
                    Some(raw) => {
                        let syntax = self.resolve_syntax(Some(raw), &raw_code);
                        let resolved = Self::resolve_language_label(raw, syntax);
                        let custom_label = self
                            .find_custom_code_block(raw)
                            .and_then(|b| b.label.clone());
                        (custom_label.unwrap_or(resolved), raw)
                    }
                    None => {
                        let custom_label = self
                            .find_custom_code_block("text")
                            .and_then(|b| b.label.clone());
                        (custom_label.unwrap_or_else(|| "Text".to_string()), "text")
                    }
                };
                self.format_code_block_label(hint_key, &base_label)
            } else {
                None
            };

        self.code_block_language = None;

        let should_wrap = self.config.is_text_wrapping_enabled();
        let wrap_mode = self.config.text_wrap_mode();

        // Ensure exactly one contextual blank line before the block.
        let code_block_prefix = self.current_code_block_prefix();
        let spacing = self.config.block_spacing.spacing(BlockElement::CodeBlock);
        self.ensure_contextual_blank_lines_with_prefix(spacing.top, &code_block_prefix);

        let render_input = CodeBlockRenderInput::new(
            &highlighted,
            language_label.as_deref(),
            code_starts_with_blank,
            should_wrap,
            wrap_mode,
            self.config.get_content_width(),
            &raw_code,
        );

        match self.config.code_block_style.style {
            CodeBlockStyle::Basic => {
                self.render_code_block_basic(render_input)?;
            }
            CodeBlockStyle::Simple => {
                self.render_code_block_simple(render_input)?;
            }
            CodeBlockStyle::Pretty => {
                self.render_code_block_pretty(render_input)?;
            }
        }

        self.ensure_contextual_blank_lines_with_prefix(spacing.bottom, &code_block_prefix);

        if !deferred_reference_blocks.is_empty() {
            for block in deferred_reference_blocks {
                self.trim_trailing_blank_lines();
                self.render_link_reference_blocks(
                    &block.links,
                    block.add_trailing_newline,
                    false,
                    0,
                );
            }
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
