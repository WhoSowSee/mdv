use super::core::FootnoteTextState;
use super::{
    CapturedReferenceBlock, Event, EventRenderer, FootnoteStyle, MissingFootnoteStyle,
    PRETTY_ACCENT_COLOR, Result, Tag, TagEnd, ThemeElement, create_style, wrap_text_with_mode,
};
use crate::terminal::AnsiStyle;
use regex::Regex;
use std::collections::HashSet;

const MISSING_FOOTNOTE_PLACEHOLDER: &str = "Missing footnote definition";
const INVALID_FOOTNOTE_SYNTAX_MESSAGE: &str = "Invalid footnote syntax";
const EMPTY_FOOTNOTE_CONTENT_MESSAGE: &str = "Empty footnote content";
const FOOTNOTE_NAME_MAX_LEN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FootnoteDefinitionKind {
    Normal,
    EmptyBody,
    InvalidSyntax,
}

#[derive(Debug, Clone)]
pub(crate) struct FootnoteDefinition {
    pub name: String,
    pub events: Vec<Event<'static>>,
    pub kind: FootnoteDefinitionKind,
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
                        kind: FootnoteDefinitionKind::Normal,
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

        let (cleaned, definitions) =
            self.extract_placeholder_footnote_definitions(cleaned, definitions);

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

    pub(super) fn should_highlight_footnote_reference(&self, name: &str) -> bool {
        if self.has_footnote_definition(name) {
            return true;
        }

        matches!(
            self.config.missing_footnote_style,
            MissingFootnoteStyle::Hide
        )
    }

    pub(super) fn has_renderable_footnotes(&self, names: &[String]) -> bool {
        names
            .iter()
            .any(|name| self.should_render_footnote_entry(name))
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

        self.ensure_placeholder_footnotes_in_order();

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

            // Footnotes always start at column 0 to avoid inherited indents.
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
            let Some(body) = self.render_footnote_definition_at(name, occurrence)? else {
                continue;
            };
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

    fn render_footnote_definition_at(
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

    fn has_footnote_definition(&self, name: &str) -> bool {
        self.footnote_definitions
            .iter()
            .any(|definition| definition.name == name)
    }

    fn should_render_footnote_entry(&self, name: &str) -> bool {
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

    fn should_render_missing_footnote(&self) -> bool {
        matches!(
            self.config.missing_footnote_style,
            MissingFootnoteStyle::Show
        )
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

    fn available_width_for_footnote(&self, prefix_width: usize) -> usize {
        let terminal_width = self.effective_text_width();
        let available = terminal_width.saturating_sub(prefix_width);
        available.max(1)
    }

    fn footnote_separator_line(&self) -> String {
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

    pub(super) fn register_footnotes_in_text(&mut self, text: &str) {
        static REGEX: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new(r"\[\^([^\]\s][^\]]*)\]").unwrap());

        for capture in REGEX.captures_iter(text) {
            if let Some(name) = capture.get(1) {
                self.register_footnote_reference(name.as_str());
            }
        }
    }

    pub(super) fn reset_footnote_text_scan(&mut self) {
        self.footnote_text_state = FootnoteTextState::Idle;
        self.footnote_text_buffer.clear();
    }

    pub(super) fn scan_footnotes_in_text_stream(&mut self, text: &str) {
        for ch in text.chars() {
            match self.footnote_text_state {
                FootnoteTextState::Idle => {
                    if ch == '[' {
                        self.footnote_text_state = FootnoteTextState::SawOpenBracket;
                    }
                }
                FootnoteTextState::SawOpenBracket => {
                    if ch == '^' {
                        self.footnote_text_buffer.clear();
                        self.footnote_text_state = FootnoteTextState::Collecting;
                    } else if ch != '[' {
                        self.footnote_text_state = FootnoteTextState::Idle;
                    }
                }
                FootnoteTextState::Collecting => {
                    if ch == ']' {
                        if !self.footnote_text_buffer.is_empty() {
                            let name = self.footnote_text_buffer.clone();
                            self.register_footnote_reference(&name);
                        }
                        self.footnote_text_buffer.clear();
                        self.footnote_text_state = FootnoteTextState::Idle;
                    } else if self.footnote_text_buffer.is_empty() && ch.is_whitespace() {
                        self.footnote_text_buffer.clear();
                        self.footnote_text_state = FootnoteTextState::Idle;
                    } else {
                        self.footnote_text_buffer.push(ch);
                        if self.footnote_text_buffer.len() > FOOTNOTE_NAME_MAX_LEN {
                            self.footnote_text_buffer.clear();
                            self.footnote_text_state = FootnoteTextState::Idle;
                        }
                    }
                }
            }
        }
    }

    fn ensure_placeholder_footnotes_in_order(&mut self) {
        for definition in &self.footnote_definitions {
            if matches!(definition.kind, FootnoteDefinitionKind::Normal) {
                continue;
            }
            if self
                .footnote_order
                .iter()
                .any(|name| name == &definition.name)
            {
                continue;
            }
            self.footnote_order.push(definition.name.clone());
        }
    }

    pub(super) fn extract_markdown_code_footnote_definitions(
        &self,
        code: &str,
    ) -> (String, Vec<FootnoteDefinition>) {
        let mut definitions = Vec::new();
        let mut known_names: HashSet<String> = self
            .footnote_definitions
            .iter()
            .map(|definition| definition.name.clone())
            .collect();
        let mut cleaned_lines = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            if let Some((name, kind)) = Self::parse_placeholder_footnote_line(trimmed) {
                if known_names.contains(&name) {
                    continue;
                }
                definitions.push(FootnoteDefinition {
                    name: name.clone(),
                    events: Vec::new(),
                    kind,
                });
                known_names.insert(name);
                continue;
            }
            cleaned_lines.push(line);
        }

        let mut cleaned = cleaned_lines.join("\n");
        if code.ends_with('\n') {
            cleaned.push('\n');
        }

        (cleaned, definitions)
    }

    fn extract_placeholder_footnote_definitions(
        &self,
        events: Vec<Event<'static>>,
        mut definitions: Vec<FootnoteDefinition>,
    ) -> (Vec<Event<'static>>, Vec<FootnoteDefinition>) {
        let mut cleaned = Vec::with_capacity(events.len());
        let mut known_names: HashSet<String> = definitions
            .iter()
            .map(|definition| definition.name.clone())
            .collect();

        let mut idx = 0usize;
        while idx < events.len() {
            if matches!(events[idx], Event::Start(Tag::Paragraph)) {
                if let Some((end_idx, placeholders)) =
                    Self::extract_bare_footnote_paragraph(&events, idx)
                {
                    for (name, kind) in placeholders {
                        if known_names.contains(&name) {
                            continue;
                        }
                        definitions.push(FootnoteDefinition {
                            name: name.clone(),
                            events: Vec::new(),
                            kind,
                        });
                        known_names.insert(name);
                    }
                    idx = end_idx + 1;
                    continue;
                }
            }

            cleaned.push(events[idx].clone());
            idx += 1;
        }

        (cleaned, definitions)
    }

    fn extract_bare_footnote_paragraph(
        events: &[Event<'static>],
        start_idx: usize,
    ) -> Option<(usize, Vec<(String, FootnoteDefinitionKind)>)> {
        let mut end_idx = start_idx + 1;
        while end_idx < events.len() {
            if matches!(events[end_idx], Event::End(TagEnd::Paragraph)) {
                break;
            }
            end_idx += 1;
        }

        if end_idx >= events.len() {
            return None;
        }

        let mut buffer = String::new();
        for event in &events[start_idx + 1..end_idx] {
            match event {
                Event::FootnoteReference(name) => {
                    buffer.push_str(&format!("[^{}]", name));
                }
                Event::Text(text) => {
                    buffer.push_str(text);
                }
                Event::SoftBreak | Event::HardBreak => {
                    buffer.push('\n');
                }
                _ => {
                    return None;
                }
            }
        }

        let candidate = buffer.trim();
        if candidate.is_empty() {
            return None;
        }

        let mut placeholders = Vec::new();
        for line in candidate.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let Some((name, kind)) = Self::parse_placeholder_footnote_line(line) else {
                return None;
            };
            placeholders.push((name, kind));
        }

        if placeholders.is_empty() {
            return None;
        }

        Some((end_idx, placeholders))
    }

    fn parse_placeholder_footnote_line(line: &str) -> Option<(String, FootnoteDefinitionKind)> {
        static BARE_REGEX: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
            Regex::new(r"^\[\^([^\]\s][^\]]*)\]\s*$").expect("valid bare footnote regex")
        });
        static EMPTY_REGEX: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
            Regex::new(r"^\[\^([^\]\s][^\]]*)\]:\s*$").expect("valid empty footnote regex")
        });

        if let Some(caps) = EMPTY_REGEX.captures(line) {
            return caps
                .get(1)
                .map(|name| (name.as_str().to_string(), FootnoteDefinitionKind::EmptyBody));
        }

        if let Some(caps) = BARE_REGEX.captures(line) {
            return caps.get(1).map(|name| {
                (
                    name.as_str().to_string(),
                    FootnoteDefinitionKind::InvalidSyntax,
                )
            });
        }

        None
    }
}
