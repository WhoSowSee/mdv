use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn finalize_inline_footnotes(
        &mut self,
        add_trailing_newline: bool,
        in_list: bool,
    ) -> Result<()> {
        if self.suppress_footnote_output {
            self.current_inline_footnotes.clear();
            return Ok(());
        }

        if !matches!(self.config.footnote_style, FootnoteStyle::Attached) {
            self.current_inline_footnotes.clear();
            return Ok(());
        }

        if self.current_inline_footnotes.is_empty() {
            return Ok(());
        }

        let inline_notes = self.current_inline_footnotes.clone();
        self.render_footnote_block(&inline_notes, add_trailing_newline, in_list)?;
        self.current_inline_footnotes.clear();
        Ok(())
    }

    pub(in crate::renderer::event) fn finalize_document_footnotes(&mut self) -> Result<()> {
        if self.suppress_footnote_output {
            return Ok(());
        }

        if !matches!(self.config.footnote_style, FootnoteStyle::Endnotes) {
            return Ok(());
        }

        self.ensure_placeholder_footnotes_in_order();

        if self.footnote_order.is_empty() {
            return Ok(());
        }

        self.render_footnote_block(&self.footnote_order.clone(), true, false)?;
        self.footnote_order.clear();
        Ok(())
    }

    pub(super) fn render_footnote_block(
        &mut self,
        names: &[String],
        add_trailing_newline: bool,
        _in_list: bool,
    ) -> Result<()> {
        let entries = self.build_footnote_blocks(names)?;
        if entries.is_empty() {
            return Ok(());
        }

        let separator = self.footnote_separator_line();
        let mut block_lines = Vec::new();
        block_lines.push(separator.clone());
        for lines in entries.iter() {
            block_lines.extend(lines.clone());
        }
        if matches!(self.config.footnote_style, FootnoteStyle::Attached) {
            block_lines.push(separator);
        }

        if self.plaintext_code_block_depth > 0 {
            self.captured_reference_blocks.push(CapturedReferenceBlock {
                lines: block_lines,
                add_trailing_newline,
            });
            return Ok(());
        }

        let spacing_element = match self.config.footnote_style {
            FootnoteStyle::Attached => BlockElement::AttachedFootnotes,
            FootnoteStyle::Endnotes => BlockElement::Endnotes,
        };
        let spacing = self.config.block_spacing.spacing(spacing_element);
        self.ensure_contextual_blank_lines(spacing.top);

        for (line_idx, line) in block_lines.iter().enumerate() {
            if line_idx > 0 {
                self.output.push('\n');
            }

            // Footnotes always start at column 0 to avoid inherited indents.
            self.output.push_str(line);
        }

        if add_trailing_newline {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            self.ensure_contextual_blank_lines(spacing.bottom);
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn build_footnote_blocks(&mut self, names: &[String]) -> Result<Vec<Vec<String>>> {
        let mut blocks = Vec::new();

        for name in names {
            let marker = format!("[^{}]", name);
            let occurrence = self.next_footnote_occurrence(name);
            let Some(body) = self.render_footnote_definition_at(name, occurrence)? else {
                continue;
            };
            let trimmed_body = body.trim_end_matches('\n');
            let lines = self.wrap_footnote_entry(&marker, trimmed_body);
            blocks.push(lines);
        }

        Ok(blocks)
    }

    pub(super) fn next_footnote_occurrence(&mut self, name: &str) -> usize {
        let entry = self.footnote_use_count.entry(name.to_string()).or_insert(0);
        let current = *entry;
        *entry += 1;
        current
    }

    pub(super) fn render_footnote_definition_at(
        &self,
        name: &str,
        occurrence: usize,
    ) -> Result<Option<String>> {
        let mut seen = 0usize;
        let mut fallback: Option<FootnoteDefinition> = None;
        let mut definition: Option<FootnoteDefinition> = None;

        for def in self.footnote_definitions.iter() {
            if def.name != name {
                continue;
            }
            fallback = Some(def.clone());
            if seen == occurrence {
                definition = Some(def.clone());
                break;
            }
            seen += 1;
        }

        let definition = definition.or(fallback);

        let Some(definition) = definition else {
            if self.should_render_missing_footnote() {
                return Ok(Some(MISSING_FOOTNOTE_PLACEHOLDER.to_string()));
            }
            return Ok(None);
        };

        match definition.kind {
            FootnoteDefinitionKind::InvalidSyntax => {
                if !self.should_render_missing_footnote() {
                    return Ok(None);
                }
                return Ok(Some(INVALID_FOOTNOTE_SYNTAX_MESSAGE.to_string()));
            }
            FootnoteDefinitionKind::EmptyBody => {
                if !self.should_render_missing_footnote() {
                    return Ok(None);
                }
                return Ok(Some(EMPTY_FOOTNOTE_CONTENT_MESSAGE.to_string()));
            }
            FootnoteDefinitionKind::Normal => {}
        }

        let mut nested_config = self.config.clone();
        nested_config.footnote_style = FootnoteStyle::Endnotes;

        let mut nested_renderer =
            EventRenderer::new(&nested_config, self.theme, self.syntax_set, self.code_theme);
        nested_renderer.suppress_footnote_output = true;
        nested_renderer.footnote_definitions = self.footnote_definitions.clone();

        let rendered = nested_renderer.render_events(definition.events)?;
        let trimmed = rendered.trim_end_matches('\n').to_string();
        if trimmed.is_empty() {
            if !self.should_render_missing_footnote() {
                return Ok(None);
            }
            return Ok(Some(EMPTY_FOOTNOTE_CONTENT_MESSAGE.to_string()));
        }
        Ok(Some(trimmed))
    }

    pub(super) fn has_footnote_definition(&self, name: &str) -> bool {
        self.footnote_definitions
            .iter()
            .any(|definition| definition.name == name)
    }

    pub(super) fn should_render_footnote_entry(&self, name: &str) -> bool {
        let definition = self
            .footnote_definitions
            .iter()
            .find(|definition| definition.name == name);

        match definition.map(|def| def.kind) {
            Some(FootnoteDefinitionKind::InvalidSyntax | FootnoteDefinitionKind::EmptyBody) => {
                self.should_render_missing_footnote()
            }
            Some(FootnoteDefinitionKind::Normal) => true,
            None => self.should_render_missing_footnote(),
        }
    }

    pub(super) fn should_render_missing_footnote(&self) -> bool {
        matches!(
            self.config.missing_footnote_style,
            MissingFootnoteStyle::Show
        )
    }

    pub(super) fn wrap_footnote_entry(&self, marker: &str, body: &str) -> Vec<String> {
        let marker_style = create_style(self.theme, ThemeElement::Link);
        let styled_marker = marker_style.apply(marker, self.config.no_colors);
        let marker_width = crate::utils::display_width(marker);
        let available_width = self.available_width_for_footnote(marker_width + 1);
        let wrap_mode = self.config.text_wrap_mode();

        let mut lines = Vec::new();

        if body.is_empty() {
            lines.push(styled_marker);
            return lines;
        }

        let mut is_first = true;
        for (line_idx, raw_line) in body.split('\n').enumerate() {
            // Preserve intentional blank lines inside the body
            if line_idx > 0 && raw_line.is_empty() {
                lines.push(String::new());
                continue;
            }

            let wrapped = if self.config.is_text_wrapping_enabled() && available_width > 0 {
                wrap_text_with_mode(raw_line, available_width, wrap_mode)
            } else {
                raw_line.to_string()
            };

            for segment in wrapped.split('\n') {
                if is_first {
                    if segment.is_empty() {
                        lines.push(styled_marker.clone());
                    } else {
                        lines.push(format!("{} {}", styled_marker, segment));
                    }
                    is_first = false;
                } else {
                    let spacer = " ".repeat(marker_width + 1);
                    lines.push(format!("{}{}", spacer, segment));
                }
            }
        }

        if lines.is_empty() {
            lines.push(styled_marker);
        }

        lines
    }

    pub(super) fn available_width_for_footnote(&self, prefix_width: usize) -> usize {
        let terminal_width = self.effective_text_width();
        let available = terminal_width.saturating_sub(prefix_width);
        available.max(1)
    }

    pub(super) fn footnote_separator_line(&self) -> String {
        let terminal_width = self.effective_text_width();
        let available = terminal_width;

        // Keep a visible separator even on very narrow widths.
        if available <= 4 {
            return "◇──◇".to_string();
        }

        let filler_width = available.saturating_sub(2).max(2);
        let line = format!("◇{}◇", "─".repeat(filler_width));
        let style = AnsiStyle::new().fg(PRETTY_ACCENT_COLOR);
        style.apply(&line, self.config.no_colors)
    }
}
