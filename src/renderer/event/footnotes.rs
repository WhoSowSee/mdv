use super::{
    CapturedReferenceBlock, Event, EventRenderer, FootnoteStyle, PRETTY_ACCENT_COLOR, Result, Tag,
    TagEnd, ThemeElement, create_style, wrap_text_with_mode,
};
use crate::terminal::AnsiStyle;
use regex::Regex;

#[derive(Debug, Clone)]
pub(crate) struct FootnoteDefinition {
    pub name: String,
    pub events: Vec<Event<'static>>,
}

impl<'a> EventRenderer<'a> {
    pub(super) fn extract_footnote_definitions(
        &self,
        events: Vec<Event<'static>>,
    ) -> (Vec<Event<'static>>, Vec<FootnoteDefinition>) {
        let mut cleaned = Vec::new();
        let mut definitions = Vec::new();
        let mut current: Option<FootnoteDefinition> = None;

        for event in events {
            match event {
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    if let Some(def) = current.take() {
                        definitions.push(def);
                    }
                    current = Some(FootnoteDefinition {
                        name: name.to_string(),
                        events: Vec::new(),
                    });
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    if let Some(def) = current.take() {
                        definitions.push(def);
                    }
                }
                other => {
                    if let Some(def) = current.as_mut() {
                        def.events.push(other);
                    } else {
                        cleaned.push(other);
                    }
                }
            }
        }

        if let Some(def) = current {
            definitions.push(def);
        }

        (cleaned, definitions)
    }

    pub(super) fn register_footnote_reference(&mut self, name: &str) {
        if self.suppress_footnote_output {
            return;
        }

        self.footnote_order.push(name.to_string());

        if matches!(self.config.footnote_style, FootnoteStyle::Attached) {
            self.current_inline_footnotes.push(name.to_string());
        }
    }

    pub(super) fn finalize_inline_footnotes(
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

    pub(super) fn finalize_document_footnotes(&mut self) -> Result<()> {
        if self.suppress_footnote_output {
            return Ok(());
        }

        if !matches!(self.config.footnote_style, FootnoteStyle::Endnotes) {
            return Ok(());
        }

        if self.footnote_order.is_empty() {
            return Ok(());
        }

        self.render_footnote_block(&self.footnote_order.clone(), true, false)?;
        self.footnote_order.clear();
        Ok(())
    }

    fn render_footnote_block(
        &mut self,
        names: &[String],
        add_trailing_newline: bool,
        _in_list: bool,
    ) -> Result<()> {
        let entries = self.build_footnote_blocks(names)?;

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

        // Ensure exactly one blank line before the footnote block, without piling up
        // extra spacing left by preceding elements.
        self.trim_trailing_blank_lines();
        if !self.output.is_empty() {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            self.output.push('\n');
        }

        for (line_idx, line) in block_lines.iter().enumerate() {
            if line_idx > 0 {
                self.output.push('\n');
            }

            self.output.push_str(line);
        }

        if add_trailing_newline {
            self.trim_trailing_blank_lines();
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            self.output.push('\n');
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    fn build_footnote_blocks(&mut self, names: &[String]) -> Result<Vec<Vec<String>>> {
        let mut blocks = Vec::new();

        for name in names {
            let marker = format!("[^{}]", name);
            let occurrence = self.next_footnote_occurrence(name);
            let body = self.render_footnote_definition_at(name, occurrence)?;
            let trimmed_body = body.trim_end_matches('\n');
            let lines = self.wrap_footnote_entry(&marker, trimmed_body);
            blocks.push(lines);
        }

        Ok(blocks)
    }

    fn next_footnote_occurrence(&mut self, name: &str) -> usize {
        let entry = self.footnote_use_count.entry(name.to_string()).or_insert(0);
        let current = *entry;
        *entry += 1;
        current
    }

    fn render_footnote_definition_at(&self, name: &str, occurrence: usize) -> Result<String> {
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
            return Ok(format!("Missing footnote definition: {}", name));
        };

        let mut nested_config = self.config.clone();
        nested_config.footnote_style = FootnoteStyle::Endnotes;

        let mut nested_renderer =
            EventRenderer::new(&nested_config, self.theme, self.syntax_set, self.code_theme);
        nested_renderer.suppress_footnote_output = true;
        nested_renderer.footnote_definitions = self.footnote_definitions.clone();

        let rendered = nested_renderer.render_events(definition.events)?;
        Ok(rendered.trim_end_matches('\n').to_string())
    }

    fn wrap_footnote_entry(&self, marker: &str, body: &str) -> Vec<String> {
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

    fn available_width_for_footnote(&self, prefix_width: usize) -> usize {
        let terminal_width = self.config.get_terminal_width();
        let available = terminal_width.saturating_sub(prefix_width);
        available.max(1)
    }

    fn footnote_separator_line(&self) -> String {
        let terminal_width = self.config.get_terminal_width();
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

    pub(super) fn register_footnotes_in_text(&mut self, text: &str) {
        static REGEX: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new(r"\[\^([^\]\s][^\]]*)\]").unwrap());

        for capture in REGEX.captures_iter(text) {
            if let Some(name) = capture.get(1) {
                self.register_footnote_reference(name.as_str());
            }
        }
    }
}
