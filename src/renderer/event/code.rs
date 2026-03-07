use super::{
    CapturedReferenceBlock, CodeBlockStyle, CodeWrapIndent, CowStr, DeferredLinkReferenceBlock,
    EventRenderer, HighlightLines, LinkStyle, MarkdownProcessor, MdvError, PRETTY_ACCENT_COLOR,
    Result, ThemeElement, WrapMode, as_24_bit_terminal_escaped, create_style, detect_source_code,
};
use crate::math::is_math_language_hint;
use crate::terminal::AnsiStyle;
use crate::utils::{display_width, strip_ansi};
use once_cell::sync::Lazy;
use regex::Regex;
use syntect::parsing::SyntaxReference;
use syntect::util::LinesWithEndings;

const LANGUAGE_SEPARATORS: &[char] = &[' ', '\t', ',', ';', '|'];

const CUSTOM_LANGUAGE_LABELS: &[(&str, &str)] = &[
    ("bash", "Bash"),
    ("shell", "Shell"),
    ("shell-session", "Shell"),
    ("console", "Shell"),
    ("sh", "Shell"),
    ("objective-c", "Objective-C"),
    ("Javascript (Babel)", "JavaScript"),
];

#[derive(Debug, Clone)]
struct WrappedCodeSegment {
    text: String,
    visible_width: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CodeBlockRenderInput<'a> {
    highlighted: &'a str,
    language_label: Option<&'a str>,
    code_starts_with_blank: bool,
    should_wrap: bool,
    wrap_mode: WrapMode,
    terminal_width: usize,
    raw_code: &'a str,
}

impl<'a> CodeBlockRenderInput<'a> {
    pub(super) fn new(
        highlighted: &'a str,
        language_label: Option<&'a str>,
        code_starts_with_blank: bool,
        should_wrap: bool,
        wrap_mode: WrapMode,
        terminal_width: usize,
        raw_code: &'a str,
    ) -> Self {
        Self {
            highlighted,
            language_label,
            code_starts_with_blank,
            should_wrap,
            wrap_mode,
            terminal_width,
            raw_code,
        }
    }
}

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_inline_code(&mut self, code: CowStr) -> Result<()> {
        // Render inline code as a single token but with correct wrapping.
        // We color only foreground (no background) to keep width calculations stable.
        let mut style = crate::terminal::AnsiStyle::new();
        style = style.fg(self.theme.code.clone().into());

        self.register_footnotes_in_text(&code);

        let raw_code = format!("`{}`", code);
        self.note_paragraph_content();

        // Table cells: let the table renderer decide about wrapping; just push styled.
        if let Some(ref mut table) = self.table_state {
            let styled_code = style.apply(&raw_code, self.config.no_colors);
            table.current_cell.push_str(&styled_code);
            return Ok(());
        }

        // If wrapping is disabled, just push styled text
        let should_wrap = self.config.is_text_wrapping_enabled();
        if !should_wrap {
            let styled_code = style.apply(&raw_code, self.config.no_colors);
            self.output.push_str(&styled_code);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(());
        }

        let terminal_width = self.config.get_terminal_width();
        let wrap_mode = self.config.text_wrap_mode();

        // Remaining visible text to place (without ANSI)
        let mut remaining = raw_code.clone();

        while !remaining.is_empty() {
            // Compute available width on the current visual line (without ANSI)
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let available = terminal_width.saturating_sub(current_line_width);

            // If there's no room left on this line, start a new one with proper indentation
            if available == 0 {
                self.push_newline_with_context();
                continue;
            }

            let line_indent_width = self.compute_line_start_context_width();
            let effective_indent = line_indent_width.min(current_line_width);
            let has_line_content = current_line_width > effective_indent;
            let remaining_width = crate::utils::display_width(&remaining);

            match wrap_mode {
                WrapMode::Word => {
                    if remaining_width <= available {
                        // Fits entirely on this line
                        let styled = style.apply(&remaining, self.config.no_colors);
                        self.output.push_str(&styled);
                        remaining.clear();
                    } else if has_line_content {
                        // Current line already has visible content; move the code span to the next line
                        self.push_newline_with_context();
                    } else {
                        // Too long even for a fresh line – fall back to character splitting
                        let (chunk, rest) = self.take_prefix_by_width(&remaining, available);
                        let styled = style.apply(&chunk, self.config.no_colors);
                        self.output.push_str(&styled);
                        remaining = rest;
                        if !remaining.is_empty() {
                            self.push_newline_with_context();
                        }
                    }
                }
                WrapMode::Character | WrapMode::None => {
                    // Fill current line up to available width
                    let (chunk, rest) = self.take_prefix_by_width(&remaining, available);
                    let styled = style.apply(&chunk, self.config.no_colors);
                    self.output.push_str(&styled);
                    remaining = rest;
                    if !remaining.is_empty() {
                        self.push_newline_with_context();
                    }
                }
            }
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_code_block_end(&mut self) -> Result<()> {
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

        let language_label = if !self.config.no_code_language {
            Some(match language_hint.as_deref() {
                Some(raw) => {
                    let syntax = self.resolve_syntax(Some(raw), &raw_code);
                    Self::resolve_language_label(raw, syntax)
                }
                None => "Text".to_string(),
            })
        } else {
            None
        };

        self.code_block_language = None;

        let should_wrap = self.config.is_text_wrapping_enabled();
        let wrap_mode = self.config.text_wrap_mode();

        // Ensure exactly one contextual blank line before the block.
        let code_block_prefix = self.current_code_block_prefix();
        self.ensure_contextual_blank_line_with_prefix(&code_block_prefix);

        let render_input = CodeBlockRenderInput::new(
            &highlighted,
            language_label.as_deref(),
            code_starts_with_blank,
            should_wrap,
            wrap_mode,
            self.config.get_terminal_width(),
            &raw_code,
        );

        match self.config.code_block_style {
            CodeBlockStyle::Simple => {
                self.render_code_block_simple(render_input)?;
            }
            CodeBlockStyle::Pretty => {
                self.render_code_block_pretty(render_input)?;
            }
        }

        self.ensure_contextual_blank_line_with_prefix(&code_block_prefix);

        if !deferred_reference_blocks.is_empty() {
            let in_list = !self.list_stack.is_empty();
            for block in deferred_reference_blocks {
                self.trim_trailing_blank_lines();
                self.render_link_reference_blocks(
                    &block.links,
                    block.add_trailing_newline,
                    in_list,
                    false,
                    0,
                );
            }
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn render_code_block_simple(
        &mut self,
        input: CodeBlockRenderInput<'_>,
    ) -> Result<()> {
        let prefix = self.render_code_block_border();
        let raw_lines: Vec<&str> = input.raw_code.lines().collect();

        if let Some(label) = input.language_label {
            let trimmed_label = label.trim();
            let base_label = if trimmed_label.is_empty() {
                "Text"
            } else {
                trimmed_label
            };

            let context_width = self.compute_code_block_context_width();
            let border_visible_width = display_width(&strip_ansi(&prefix));
            let available_width = input
                .terminal_width
                .saturating_sub(context_width + border_visible_width);

            let wrapped_label = if input.should_wrap && available_width > 0 {
                crate::utils::wrap_text_with_mode(base_label, available_width, input.wrap_mode)
            } else {
                base_label.to_string()
            };

            for part in wrapped_label.split('\n') {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&prefix);
                self.output.push_str(&self.style_pretty_accent(part));
                self.output.push('\n');
            }

            if !input.code_starts_with_blank {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&prefix);
                self.output.push('\n');
            }
        }

        for (idx, line) in input.highlighted.lines().enumerate() {
            let context_width = self.compute_code_block_context_width();
            let border_visible_width = 2usize;
            let available = input
                .terminal_width
                .saturating_sub(context_width + border_visible_width);

            let raw_line = raw_lines.get(idx).copied();

            let segments = self.wrap_code_line_segments(
                line,
                raw_line,
                available,
                input.should_wrap,
                input.wrap_mode,
            );

            for segment in segments {
                self.push_code_block_indent_for_line_start();
                self.output.push_str(&prefix);
                let decorated = self.highlight_footnote_markers_in_ansi(&segment.text);
                self.output.push_str(&decorated);
                self.output.push('\n');
            }
        }

        Ok(())
    }

    pub(super) fn render_code_block_pretty(
        &mut self,
        input: CodeBlockRenderInput<'_>,
    ) -> Result<()> {
        let left_padding = 1usize;
        let right_padding = 1usize;

        let context_width = self.compute_code_block_context_width();
        let available_frame_width = input.terminal_width.saturating_sub(context_width);
        if available_frame_width <= 4 {
            return self.render_code_block_simple(input);
        }

        let max_inner_box_width = available_frame_width;
        let max_text_width_allowed = max_inner_box_width.saturating_sub(2);
        if max_text_width_allowed < left_padding + right_padding + 1 {
            return self.render_code_block_simple(input);
        }

        let highlight_lines: Vec<&str> = input.highlighted.lines().collect();
        let raw_code_lines: Vec<&str> = input.raw_code.lines().collect();
        let mut max_line_width = 0usize;
        for line in &highlight_lines {
            max_line_width = max_line_width.max(display_width(&strip_ansi(line)));
        }

        let wrap_width_allowed =
            max_text_width_allowed.saturating_sub(left_padding + right_padding);
        let needs_wrap = input.should_wrap
            && max_line_width + left_padding + right_padding > max_text_width_allowed;

        let mut rendered_lines: Vec<String> = Vec::new();
        let mut max_part_width = 0usize;

        if needs_wrap {
            if wrap_width_allowed == 0 {
                return self.render_code_block_simple(input);
            }

            for (idx, line) in highlight_lines.iter().enumerate() {
                let raw_line = raw_code_lines.get(idx).copied();
                let segments = self.wrap_code_line_segments_pretty(
                    line,
                    raw_line,
                    wrap_width_allowed,
                    input.should_wrap,
                    input.wrap_mode,
                );

                for segment in segments {
                    max_part_width = max_part_width.max(segment.visible_width);
                    rendered_lines.push(segment.text);
                }
            }

            if max_part_width > wrap_width_allowed {
                return self.render_code_block_simple(input);
            }
        } else {
            if highlight_lines.is_empty() {
                rendered_lines.push(String::new());
            } else {
                for (idx, line) in highlight_lines.iter().enumerate() {
                    let raw_line = raw_code_lines.get(idx).copied();
                    let segments = self.wrap_code_line_segments_pretty(
                        line,
                        raw_line,
                        wrap_width_allowed,
                        false,
                        input.wrap_mode,
                    );

                    for segment in segments {
                        max_part_width = max_part_width.max(segment.visible_width);
                        rendered_lines.push(segment.text);
                    }
                }
            }

            if max_part_width + left_padding + right_padding > max_text_width_allowed {
                return self.render_code_block_simple(input);
            }
        }

        if rendered_lines.is_empty() {
            rendered_lines.push(String::new());
        }

        let block_is_empty = rendered_lines
            .iter()
            .all(|line| strip_ansi(line).trim().is_empty());

        let mut text_width = left_padding + max_part_width + right_padding;
        let mut inner_box_width = text_width + 2;

        if let Some(label) = input.language_label {
            let trimmed = label.trim();
            if !trimmed.is_empty() {
                if block_is_empty {
                    let label_width = display_width(trimmed);
                    let required_inner_width = label_width + 6;
                    if required_inner_width > max_inner_box_width {
                        return self.render_code_block_simple(input);
                    }
                }

                let label_width = display_width(trimmed);
                // Ensure at least one trailing dash after the label on the top border
                // so frames like an empty "Text" block appear balanced: "╭─ Text ─╮".
                let required_inner_width = (label_width + 6).min(max_inner_box_width);
                if inner_box_width < required_inner_width {
                    inner_box_width = required_inner_width;
                    text_width = inner_box_width.saturating_sub(2);
                }
            }
        }

        self.push_code_block_indent_for_line_start();
        let top_line = self.render_pretty_top_border(inner_box_width, input.language_label);
        self.output.push_str(&top_line);
        self.output.push('\n');

        for part in rendered_lines {
            self.push_code_block_indent_for_line_start();
            let decorated = self.highlight_footnote_markers_in_ansi(&part);
            let content_line = self.render_pretty_content_line(text_width, &decorated);
            self.output.push_str(&content_line);
            self.output.push('\n');
        }

        self.push_code_block_indent_for_line_start();
        let bottom_line = self.render_pretty_bottom_border(inner_box_width);
        self.output.push_str(&bottom_line);
        self.output.push('\n');

        Ok(())
    }

    fn wrap_code_line_segments_pretty(
        &self,
        highlighted_line: &str,
        raw_line: Option<&str>,
        width: usize,
        should_wrap: bool,
        wrap_mode: WrapMode,
    ) -> Vec<WrappedCodeSegment> {
        let mut segments =
            self.wrap_code_line_segments(highlighted_line, raw_line, width, should_wrap, wrap_mode);

        if should_wrap && width > 0 && matches!(wrap_mode, WrapMode::Word) {
            let has_overflow = segments.iter().any(|segment| segment.visible_width > width);
            if has_overflow {
                // Fall back to character wrapping to keep the pretty frame consistent.
                segments = self.wrap_code_line_segments(
                    highlighted_line,
                    raw_line,
                    width,
                    should_wrap,
                    WrapMode::Character,
                );
            }
        }

        segments
    }

    fn wrap_code_line_segments(
        &self,
        highlighted_line: &str,
        raw_line: Option<&str>,
        width: usize,
        should_wrap: bool,
        wrap_mode: WrapMode,
    ) -> Vec<WrappedCodeSegment> {
        let base_indent = if let Some(line) = raw_line {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        } else {
            let stripped = strip_ansi(highlighted_line);
            stripped
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        };

        let continuation_indent = match self.config.code_wrap_indent {
            CodeWrapIndent::None => String::new(),
            CodeWrapIndent::Base => base_indent.clone(),
            CodeWrapIndent::Double => {
                let mut indent = base_indent.clone();
                indent.push_str("  ");
                indent
            }
        };

        let raw_wrapped = if should_wrap && width > 0 {
            crate::utils::wrap_text_with_mode(highlighted_line, width, wrap_mode)
        } else {
            highlighted_line.to_string()
        };

        let mut segments_raw: Vec<String> = raw_wrapped
            .split('\n')
            .map(|part| part.to_string())
            .collect();
        if segments_raw.is_empty() {
            segments_raw.push(String::new());
        }

        let mut segments = Vec::with_capacity(segments_raw.len());
        for (idx, mut segment) in segments_raw.into_iter().enumerate() {
            let mut visible_width = display_width(&strip_ansi(&segment));

            if idx > 0 && !continuation_indent.is_empty() {
                let candidate = format!("{}{}", continuation_indent, segment);
                let candidate_width = display_width(&strip_ansi(&candidate));
                if should_wrap && width > 0 && candidate_width > width {
                    // Not enough room to apply hanging indent - retain original segment.
                    visible_width = display_width(&strip_ansi(&segment));
                } else {
                    segment = candidate;
                    visible_width = candidate_width;
                }
            }

            segments.push(WrappedCodeSegment {
                text: segment,
                visible_width,
            });
        }

        segments
    }

    fn render_pretty_top_border(&self, inner_box_width: usize, label: Option<&str>) -> String {
        let mut line = String::from("╭");
        if inner_box_width <= 1 {
            return self.style_pretty_accent(&line);
        }

        let mut middle_width = inner_box_width.saturating_sub(2);

        if middle_width > 0 {
            line.push('─');
            middle_width = middle_width.saturating_sub(1);
        }

        if let Some(raw_label) = label {
            let trimmed = raw_label.trim();
            if !trimmed.is_empty() && middle_width > 0 {
                line.push(' ');
                middle_width = middle_width.saturating_sub(1);

                if middle_width > 0 {
                    let mut label_text = trimmed.to_string();
                    if display_width(&label_text) > middle_width {
                        label_text = self.take_prefix_by_width(&label_text, middle_width).0;
                    }

                    let label_width = display_width(&label_text);
                    if label_width > 0 && label_width <= middle_width {
                        line.push_str(&label_text);
                        middle_width = middle_width.saturating_sub(label_width);
                        if middle_width > 0 {
                            line.push(' ');
                            middle_width = middle_width.saturating_sub(1);
                        }
                    } else {
                        // Not enough room for the label – remove the preceding space
                        if line.ends_with(' ') {
                            line.pop();
                            middle_width = middle_width.saturating_add(1);
                        }
                    }
                }
            }
        }

        while middle_width > 0 {
            line.push('─');
            middle_width = middle_width.saturating_sub(1);
        }

        line.push('╮');

        self.style_pretty_accent(&line)
    }

    fn render_pretty_bottom_border(&self, inner_box_width: usize) -> String {
        let mut line = String::from("╰");
        if inner_box_width > 1 {
            let repeat = inner_box_width.saturating_sub(2);
            if repeat > 0 {
                line.push_str(&"─".repeat(repeat));
            }
            line.push('╯');
        } else {
            line.push('╯');
        }

        self.style_pretty_accent(&line)
    }

    fn render_pretty_content_line(&self, text_width: usize, part: &str) -> String {
        let content_width = display_width(&strip_ansi(part));
        let inner_width = (1 + content_width).max(2);
        let mandatory_right_pad = inner_width - (1 + content_width);
        let trailing_pad = text_width.saturating_sub(inner_width);

        let mut line = String::new();
        line.push_str(&self.style_pretty_accent("│"));
        line.push(' ');
        line.push_str(part);
        if mandatory_right_pad > 0 {
            line.push_str(&" ".repeat(mandatory_right_pad));
        }
        if trailing_pad > 0 {
            line.push_str(&" ".repeat(trailing_pad));
        }
        line.push_str(&self.style_pretty_accent("│"));
        line
    }

    fn style_pretty_accent(&self, text: &str) -> String {
        if self.config.no_colors {
            text.to_string()
        } else {
            AnsiStyle::new()
                .fg(PRETTY_ACCENT_COLOR)
                .apply(text, self.config.no_colors)
        }
    }

    pub(super) fn highlight_code(&self, code: &str, language_hint: Option<&str>) -> Result<String> {
        if self.config.no_colors {
            return Ok(code.to_string());
        }

        let syntax = self.resolve_syntax(language_hint, code);

        let mut highlighter = HighlightLines::new(syntax, self.code_theme);
        let mut result = String::new();

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter
                .highlight_line(line, self.syntax_set)
                .map_err(|e| MdvError::SyntaxError(e.to_string()))?;

            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            result.push_str(&escaped);

            if !line.ends_with('\n') {
                // Maintain the trailing newline that callers expect so wrapping keeps working.
                result.push('\n');
            }
        }

        Ok(result)
    }

    fn highlight_footnote_markers_in_ansi(&self, line: &str) -> String {
        if self.config.no_colors {
            return line.to_string();
        }

        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\[\^([^\]\s][^\]]*)\]").expect("valid footnote regex"));

        let clean = strip_ansi(line);
        if !REGEX.is_match(&clean) {
            return line.to_string();
        }

        // Build mapping from visible char index to byte range and last SGR sequence.
        let mut mapping: Vec<(usize, usize, Option<String>)> = Vec::new();
        let mut current_sgr: Option<String> = None;
        let bytes = line.as_bytes();
        let mut i = 0usize;
        while i < line.len() {
            if bytes[i] == 0x1b
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'['
                && let Some(rel) = line[i + 2..].find('m')
            {
                let end = i + 2 + rel;
                current_sgr = Some(line[i..=end].to_string());
                i = end + 1;
                continue;
            }

            let ch = line[i..].chars().next().unwrap_or('\0');
            let start = i;
            i += ch.len_utf8();
            mapping.push((start, i, current_sgr.clone()));
        }

        let style = create_style(self.theme, ThemeElement::Link);
        let mut result = String::new();
        let mut prev_end = 0usize;

        for capture in REGEX.captures_iter(&clean) {
            let Some(matched) = capture.get(0) else {
                continue;
            };

            let start_v = matched.start();
            let end_v = matched.end();

            if start_v >= mapping.len() || end_v == 0 || end_v > mapping.len() {
                continue;
            }

            let name = capture
                .get(1)
                .map(|group| group.as_str())
                .unwrap_or_default();

            let start_byte = mapping[start_v].0;
            let end_byte = mapping[end_v - 1].1;
            let restore = mapping[start_v].2.clone();

            // Append text before the marker
            result.push_str(&line[prev_end..start_byte]);

            let marker = &line[start_byte..end_byte];
            if self.should_highlight_footnote_reference(name) {
                let mut styled = style.apply(marker, self.config.no_colors);
                if let Some(sgr) = restore {
                    styled.push_str(&sgr);
                }
                result.push_str(&styled);
            } else {
                result.push_str(marker);
            }

            prev_end = end_byte;
        }

        result.push_str(&line[prev_end..]);
        result
    }

    fn is_markdown_language_hint(hint: Option<&str>) -> bool {
        let Some(raw) = hint else {
            return false;
        };
        let normalized = raw.trim().to_ascii_lowercase();
        matches!(normalized.as_str(), "md" | "markdown")
    }

    fn should_render_code_block_as_plaintext(&self, language_hint: Option<&str>) -> bool {
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

    fn render_plaintext_code_block(&self, code: &str) -> Result<PlaintextRenderResult> {
        let mut nested_config = self.config.clone();
        nested_config.from_text = None;

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
        rendered = rendered.trim_end_matches('\n').to_string();

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

    fn estimate_plaintext_block_width(&self) -> Option<usize> {
        let terminal_width = self.config.get_terminal_width();
        if terminal_width == 0 {
            return None;
        }

        let context_width = self.compute_code_block_context_width();
        let available = terminal_width.saturating_sub(context_width);
        if available == 0 {
            return None;
        }

        let width = match self.config.code_block_style {
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

    fn embed_captured_reference_blocks_in_code_body(
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

    fn resolve_syntax<'s>(
        &'s self,
        language_hint: Option<&str>,
        code: &str,
    ) -> &'s SyntaxReference {
        let mut seen: Vec<String> = Vec::new();

        if let Some(lang) = language_hint {
            let candidates = Self::split_language_hint(lang);
            if let Some(hit) = self.try_lookup(&candidates, &mut seen) {
                return hit;
            }

            if !self.config.code_guessing {
                return self.syntax_set.find_syntax_plain_text();
            }
        }

        if !self.config.code_guessing {
            return self.syntax_set.find_syntax_plain_text();
        }

        if let Some(first_line_match) = self.syntax_set.find_syntax_by_first_line(code) {
            return first_line_match;
        }

        if let Some(guessed) = detect_source_code(code, None)
            && let Some(hit) = self.try_lookup(&[guessed], &mut seen)
        {
            return hit;
        }

        self.syntax_set.find_syntax_plain_text()
    }

    fn resolve_language_label(raw_hint: &str, syntax: &SyntaxReference) -> String {
        let syntax_name = syntax.name.trim();
        let syntax_name_lower = syntax_name.to_ascii_lowercase();

        if let Some(label) = Self::custom_language_label(raw_hint, &syntax_name_lower) {
            return label;
        }

        if syntax_name_lower.contains("plain text") {
            return Self::fallback_language_label(raw_hint).unwrap_or_else(|| "Text".to_string());
        }

        syntax_name.to_string()
    }

    fn fallback_language_label(raw_hint: &str) -> Option<String> {
        let tokens = Self::split_language_hint(raw_hint);
        for token in tokens {
            if token.is_empty() {
                continue;
            }

            if Self::is_plain_language(&token) {
                return Some("Text".to_string());
            }

            let label = Self::humanize_language_token(&token);
            if !label.is_empty() {
                return Some(label);
            }
        }

        None
    }

    fn custom_language_label(raw_hint: &str, syntax_name_lower: &str) -> Option<String> {
        if let Some(label) = Self::lookup_custom_label(syntax_name_lower) {
            return Some(label.to_string());
        }

        for token in Self::split_language_hint(raw_hint) {
            if let Some(label) = Self::lookup_custom_label(&token) {
                return Some(label.to_string());
            }
        }

        None
    }

    fn lookup_custom_label(key: &str) -> Option<&'static str> {
        let normalized = key.trim().to_ascii_lowercase();
        for (candidate, label) in CUSTOM_LANGUAGE_LABELS {
            if candidate.eq_ignore_ascii_case(&normalized) {
                return Some(*label);
            }
        }
        None
    }

    fn humanize_language_token(token: &str) -> String {
        if token.is_empty() {
            return String::new();
        }

        if token.contains(['-', '_', '/', '.']) {
            let parts: Vec<String> = token
                .split(['-', '_', '/', '.'])
                .filter(|part| !part.is_empty())
                .map(Self::humanize_language_token)
                .filter(|part| !part.is_empty())
                .collect();
            if parts.is_empty() {
                return String::new();
            }
            return parts.join(" ");
        }

        if token.len() <= 3 && token.chars().all(|c| c.is_ascii_alphabetic()) {
            return token.to_ascii_uppercase();
        }

        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            let mut result = String::new();
            result.extend(first.to_uppercase());
            result.push_str(chars.as_str());
            return result;
        }

        String::new()
    }

    fn try_lookup<'s>(
        &'s self,
        tokens: &[String],
        seen: &mut Vec<String>,
    ) -> Option<&'s SyntaxReference> {
        for token in tokens {
            if token.is_empty() {
                continue;
            }

            if seen
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(token))
            {
                continue;
            }
            seen.push(token.clone());

            if Self::is_plain_language(token) {
                return Some(self.syntax_set.find_syntax_plain_text());
            }

            for candidate in Self::expand_language_aliases(token) {
                if let Some(syntax) = self.lookup_syntax(&candidate) {
                    return Some(syntax);
                }
            }
        }

        None
    }

    fn lookup_syntax<'s>(&'s self, token: &str) -> Option<&'s SyntaxReference> {
        if token.is_empty() {
            return None;
        }

        self.syntax_set
            .find_syntax_by_token(token)
            .or_else(|| self.syntax_set.find_syntax_by_name(token))
            .or_else(|| self.syntax_set.find_syntax_by_extension(token))
    }

    fn split_language_hint(hint: &str) -> Vec<String> {
        let mut parts = Vec::new();

        let trimmed = hint.trim();
        if trimmed.is_empty() {
            return parts;
        }

        for fragment in trimmed.split(LANGUAGE_SEPARATORS) {
            let mut piece = fragment.trim();
            if piece.is_empty() {
                continue;
            }

            if let Some((_, value)) = piece.split_once('=') {
                piece = value.trim();
            }

            if piece.starts_with('{') && piece.ends_with('}') && piece.len() > 2 {
                piece = &piece[1..piece.len() - 1];
            }

            let piece = piece
                .trim()
                .trim_matches(|c: char| matches!(c, '{' | '}' | '"' | '\'' | '`' | '.' | '!'));

            if piece.is_empty() {
                continue;
            }

            let piece = piece.strip_prefix("language-").unwrap_or(piece);
            let piece = piece.strip_prefix('.').unwrap_or(piece);

            let normalized = piece.trim();
            if normalized.is_empty() {
                continue;
            }

            let normalized = normalized.to_lowercase();
            if !parts
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&normalized))
            {
                parts.push(normalized);
            }
        }

        parts
    }

    fn expand_language_aliases(token: &str) -> Vec<String> {
        let mut aliases = Vec::new();
        Self::push_candidate(&mut aliases, token);

        let lower = token.to_lowercase();
        if lower != token {
            Self::push_candidate(&mut aliases, &lower);
        }

        match lower.as_str() {
            "rs" | "rust" => {
                Self::push_candidate(&mut aliases, "rs");
                Self::push_candidate(&mut aliases, "rust");
                Self::push_candidate(&mut aliases, "Rust");
            }
            "py" | "python" => {
                Self::push_candidate(&mut aliases, "py");
                Self::push_candidate(&mut aliases, "python");
                Self::push_candidate(&mut aliases, "Python");
            }
            "js" | "javascript" | "node" | "nodejs" | "ecmascript" => {
                Self::push_candidate(&mut aliases, "js");
                Self::push_candidate(&mut aliases, "javascript");
                Self::push_candidate(&mut aliases, "JavaScript");
                Self::push_candidate(&mut aliases, "JavaScript (Babel)");
            }
            "jsx" => {
                Self::push_candidate(&mut aliases, "jsx");
                Self::push_candidate(&mut aliases, "JavaScript (Babel)");
            }
            "ts" | "typescript" => {
                Self::push_candidate(&mut aliases, "ts");
                Self::push_candidate(&mut aliases, "typescript");
                Self::push_candidate(&mut aliases, "TypeScript");
            }
            "tsx" | "typescriptreact" => {
                Self::push_candidate(&mut aliases, "tsx");
                Self::push_candidate(&mut aliases, "TypeScriptReact");
                Self::push_candidate(&mut aliases, "TypeScript");
            }
            "c" => {
                Self::push_candidate(&mut aliases, "c");
                Self::push_candidate(&mut aliases, "C");
            }
            "h" => {
                Self::push_candidate(&mut aliases, "c");
                Self::push_candidate(&mut aliases, "C");
            }
            "cpp" | "c++" | "cxx" | "hpp" => {
                Self::push_candidate(&mut aliases, "cpp");
                Self::push_candidate(&mut aliases, "c++");
                Self::push_candidate(&mut aliases, "C++");
                Self::push_candidate(&mut aliases, "cxx");
            }
            "objc" | "objective-c" | "objectivec" => {
                Self::push_candidate(&mut aliases, "objc");
                Self::push_candidate(&mut aliases, "Objective-C");
                Self::push_candidate(&mut aliases, "Objectivec");
            }
            "objcpp" | "objective-c++" => {
                Self::push_candidate(&mut aliases, "objective-c++");
                Self::push_candidate(&mut aliases, "Objective-C++");
                Self::push_candidate(&mut aliases, "objcpp");
            }
            "cs" | "csharp" | "c#" => {
                Self::push_candidate(&mut aliases, "cs");
                Self::push_candidate(&mut aliases, "csharp");
                Self::push_candidate(&mut aliases, "C#");
            }
            "go" | "golang" => {
                Self::push_candidate(&mut aliases, "go");
                Self::push_candidate(&mut aliases, "Go");
            }
            "java" => {
                Self::push_candidate(&mut aliases, "java");
                Self::push_candidate(&mut aliases, "Java");
            }
            "kotlin" | "kt" => {
                Self::push_candidate(&mut aliases, "kt");
                Self::push_candidate(&mut aliases, "kotlin");
                Self::push_candidate(&mut aliases, "Kotlin");
            }
            "swift" => {
                Self::push_candidate(&mut aliases, "swift");
                Self::push_candidate(&mut aliases, "Swift");
            }
            "scala" => {
                Self::push_candidate(&mut aliases, "scala");
                Self::push_candidate(&mut aliases, "Scala");
            }
            "php" => {
                Self::push_candidate(&mut aliases, "php");
                Self::push_candidate(&mut aliases, "PHP");
            }
            "rb" | "ruby" => {
                Self::push_candidate(&mut aliases, "rb");
                Self::push_candidate(&mut aliases, "ruby");
                Self::push_candidate(&mut aliases, "Ruby");
            }
            "perl" | "pl" => {
                Self::push_candidate(&mut aliases, "pl");
                Self::push_candidate(&mut aliases, "Perl");
            }
            "lua" => {
                Self::push_candidate(&mut aliases, "lua");
                Self::push_candidate(&mut aliases, "Lua");
            }
            "r" => {
                Self::push_candidate(&mut aliases, "r");
                Self::push_candidate(&mut aliases, "R");
            }
            "dart" => {
                Self::push_candidate(&mut aliases, "dart");
                Self::push_candidate(&mut aliases, "Dart");
            }
            "haskell" | "hs" => {
                Self::push_candidate(&mut aliases, "hs");
                Self::push_candidate(&mut aliases, "Haskell");
            }
            "clj" | "clojure" => {
                Self::push_candidate(&mut aliases, "clj");
                Self::push_candidate(&mut aliases, "Clojure");
            }
            "elixir" => {
                Self::push_candidate(&mut aliases, "elixir");
                Self::push_candidate(&mut aliases, "Elixir");
            }
            "erlang" => {
                Self::push_candidate(&mut aliases, "erlang");
                Self::push_candidate(&mut aliases, "Erlang");
            }
            "fsharp" | "fs" | "f#" => {
                Self::push_candidate(&mut aliases, "F#");
                Self::push_candidate(&mut aliases, "FSharp");
                Self::push_candidate(&mut aliases, "fs");
            }
            "sql" | "sqlite" | "postgres" | "mysql" => {
                Self::push_candidate(&mut aliases, "sql");
                Self::push_candidate(&mut aliases, "SQL");
            }
            "yaml" | "yml" => {
                Self::push_candidate(&mut aliases, "yaml");
                Self::push_candidate(&mut aliases, "YAML");
                Self::push_candidate(&mut aliases, "yml");
            }
            "json" | "jsonc" | "json5" => {
                Self::push_candidate(&mut aliases, "json");
                Self::push_candidate(&mut aliases, "JSON");
            }
            "toml" => {
                Self::push_candidate(&mut aliases, "toml");
                Self::push_candidate(&mut aliases, "TOML");
            }
            "ini" | "cfg" | "conf" => {
                Self::push_candidate(&mut aliases, "ini");
                Self::push_candidate(&mut aliases, "INI");
            }
            "md" | "markdown" => {
                Self::push_candidate(&mut aliases, "md");
                Self::push_candidate(&mut aliases, "markdown");
                Self::push_candidate(&mut aliases, "Markdown");
            }
            "html" | "htm" | "xhtml" => {
                Self::push_candidate(&mut aliases, "html");
                Self::push_candidate(&mut aliases, "HTML");
            }
            "xml" => {
                Self::push_candidate(&mut aliases, "xml");
                Self::push_candidate(&mut aliases, "XML");
            }
            "css" => {
                Self::push_candidate(&mut aliases, "css");
                Self::push_candidate(&mut aliases, "CSS");
            }
            "scss" => {
                Self::push_candidate(&mut aliases, "scss");
                Self::push_candidate(&mut aliases, "SCSS");
            }
            "less" => {
                Self::push_candidate(&mut aliases, "less");
                Self::push_candidate(&mut aliases, "LESS");
            }
            "bash" | "sh" | "shell" | "zsh" | "shell-session" | "console" => {
                Self::push_candidate(&mut aliases, "bash");
                Self::push_candidate(&mut aliases, "Bash");
                Self::push_candidate(&mut aliases, "shell");
                Self::push_candidate(&mut aliases, "Shell");
                Self::push_candidate(&mut aliases, "Shell-Unix-Generic");
                Self::push_candidate(&mut aliases, "sh");
            }
            "fish" => {
                Self::push_candidate(&mut aliases, "fish");
                Self::push_candidate(&mut aliases, "Fish");
            }
            "powershell" | "ps" | "ps1" => {
                Self::push_candidate(&mut aliases, "powershell");
                Self::push_candidate(&mut aliases, "PowerShell");
                Self::push_candidate(&mut aliases, "ps1");
            }
            "cmd" | "batch" | "bat" => {
                Self::push_candidate(&mut aliases, "Batchfile");
                Self::push_candidate(&mut aliases, "batch");
                Self::push_candidate(&mut aliases, "bat");
            }
            "make" | "makefile" => {
                Self::push_candidate(&mut aliases, "make");
                Self::push_candidate(&mut aliases, "Makefile");
            }
            "cmake" => {
                Self::push_candidate(&mut aliases, "cmake");
                Self::push_candidate(&mut aliases, "CMake");
            }
            "docker" | "dockerfile" => {
                Self::push_candidate(&mut aliases, "docker");
                Self::push_candidate(&mut aliases, "Dockerfile");
            }
            "graphql" | "gql" => {
                Self::push_candidate(&mut aliases, "graphql");
                Self::push_candidate(&mut aliases, "GraphQL");
            }
            "proto" | "protobuf" => {
                Self::push_candidate(&mut aliases, "proto");
                Self::push_candidate(&mut aliases, "Protocol Buffer");
            }
            "plantuml" | "uml" => {
                Self::push_candidate(&mut aliases, "plantuml");
                Self::push_candidate(&mut aliases, "PlantUML");
            }
            "mermaid" => {
                Self::push_candidate(&mut aliases, "mermaid");
                Self::push_candidate(&mut aliases, "Mermaid");
            }
            "diff" | "patch" | "gdiff" => {
                Self::push_candidate(&mut aliases, "diff");
                Self::push_candidate(&mut aliases, "Diff");
                Self::push_candidate(&mut aliases, "patch");
            }
            "log" => {
                Self::push_candidate(&mut aliases, "Log");
            }
            "latex" | "tex" => {
                Self::push_candidate(&mut aliases, "latex");
                Self::push_candidate(&mut aliases, "LaTeX");
                Self::push_candidate(&mut aliases, "tex");
                Self::push_candidate(&mut aliases, "TeX");
            }
            "rst" | "restructuredtext" => {
                Self::push_candidate(&mut aliases, "rst");
                Self::push_candidate(&mut aliases, "reStructuredText");
            }
            "adoc" | "asciidoc" => {
                Self::push_candidate(&mut aliases, "adoc");
                Self::push_candidate(&mut aliases, "AsciiDoc");
            }
            "matlab" | "octave" => {
                Self::push_candidate(&mut aliases, "matlab");
                Self::push_candidate(&mut aliases, "Matlab");
                Self::push_candidate(&mut aliases, "Octave");
            }
            "vb" | "visualbasic" => {
                Self::push_candidate(&mut aliases, "vb");
                Self::push_candidate(&mut aliases, "Visual Basic");
                Self::push_candidate(&mut aliases, "VB.NET");
            }
            "zig" => {
                Self::push_candidate(&mut aliases, "zig");
                Self::push_candidate(&mut aliases, "Zig");
            }
            "nim" => {
                Self::push_candidate(&mut aliases, "nim");
                Self::push_candidate(&mut aliases, "Nim");
            }
            "solidity" | "sol" => {
                Self::push_candidate(&mut aliases, "solidity");
                Self::push_candidate(&mut aliases, "Solidity");
            }
            "proto3" => {
                Self::push_candidate(&mut aliases, "proto3");
                Self::push_candidate(&mut aliases, "Protocol Buffer");
            }
            "assembly" | "asm" => {
                Self::push_candidate(&mut aliases, "asm");
                Self::push_candidate(&mut aliases, "Assembly");
            }
            "wasm" | "wat" => {
                Self::push_candidate(&mut aliases, "wat");
                Self::push_candidate(&mut aliases, "WebAssembly");
            }
            _ => {}
        }

        aliases
    }

    fn push_candidate(target: &mut Vec<String>, candidate: &str) {
        if candidate.is_empty() {
            return;
        }

        if target
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(candidate))
        {
            return;
        }

        target.push(candidate.to_string());
    }

    fn is_plain_language(token: &str) -> bool {
        matches!(
            token.to_lowercase().as_str(),
            "text"
                | "plain"
                | "plaintext"
                | "plain_text"
                | "txt"
                | "output"
                | "nohighlight"
                | "none"
        )
    }
}

struct PlaintextRenderResult {
    body: String,
    references: Vec<CapturedReferenceBlock>,
    deferred_references: Vec<DeferredLinkReferenceBlock>,
    document_links: Vec<(String, String)>,
    reference_counter: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::theme::Theme;
    use syntect::highlighting::Theme as SyntectTheme;
    use syntect::parsing::SyntaxSet;

    #[test]
    fn resolve_syntax_returns_plain_text_when_guessing_disabled() {
        let config = Config {
            code_guessing: false,
            ..Config::default()
        };

        let theme = Theme::default();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let code_theme = SyntectTheme::default();

        let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);

        let syntax_with_hint = renderer.resolve_syntax(Some("unknownlang"), "fn main() {}");
        assert_eq!(syntax_with_hint.name, "Plain Text");

        let syntax_without_hint = renderer.resolve_syntax(None, "fn main() {}");
        assert_eq!(syntax_without_hint.name, "Plain Text");
    }
}
