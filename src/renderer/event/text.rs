use super::core::{CalloutFold, CalloutInfo, CalloutKind, CalloutState};
use super::{CalloutStyle, CowStr, EventRenderer, LinkStyle, Result, ThemeElement, create_style};

#[derive(Debug, Clone)]
struct HighlightSegment {
    text: String,
    highlighted: bool,
}

const CALLOUT_BUFFER_LIMIT: usize = 64;

enum CalloutBufferEval {
    Pending,
    Callout(CalloutMarker),
    NotCallout,
}

struct CalloutMarker {
    kind: CalloutKind,
    label: String,
    label_override: Option<String>,
    fold: Option<CalloutFold>,
    trailing: Option<String>,
    allow_label_override: bool,
    suppress_paragraph_break: bool,
}

enum CalloutDecision {
    RenderHeader {
        kind: CalloutKind,
        label: String,
        label_override: Option<String>,
        fold: Option<CalloutFold>,
        trailing: Option<String>,
        suppress_paragraph_break: bool,
    },
    AwaitLabelOverride,
    FlushBuffer(String),
    Pending,
}

impl<'a> EventRenderer<'a> {
    fn line_has_visible_text(line: &str) -> bool {
        line.chars()
            .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃')
    }

    pub(super) fn handle_text(&mut self, text: CowStr) -> Result<()> {
        if !self.in_code_block && !self.in_link {
            self.scan_footnotes_in_text_stream(&text);
        }

        if self.in_code_block {
            self.pending_task_marker = false;
            self.pending_task_marker_buffer.clear();
            self.code_block_content.push_str(&text);
            return Ok(());
        } else if self.in_link {
            match self.config.link_style {
                LinkStyle::Clickable => {
                    // For Clickable mode, collect link text but don't add to output yet
                    // We'll add the complete clickable link in handle_link_end
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::ClickableForced => {
                    // For ClickableForced mode, collect link text but don't add to output yet
                    // We'll add the complete clickable link in handle_link_end
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::Inline => {
                    // For Inline mode, collect link text but don't add to output yet
                    // We'll add the underlined text and URL in handle_link_end with flexible breaking
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::InlineTable => {
                    // Collect link text but don't add to output yet, similar to other modes
                    // We'll add the underlined text and reference number in handle_link_end
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::EndTable => {
                    // Collect link text for document-scoped reference handling
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::Hide => {
                    // This shouldn't happen since we don't set in_link for Hide mode anymore
                }
            }
            self.pending_task_marker = false;
            self.pending_task_marker_buffer.clear();
            return Ok(());
        }

        let raw_text = text.as_ref();
        if self.pending_callout_label_override {
            if self.pending_callout_label_buffer.is_empty() {
                let mut remaining = raw_text;

                if let Some(first) = remaining.chars().next()
                    && matches!(first, '+' | '-')
                {
                    if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut()
                        && info.fold.is_none()
                    {
                        info.fold = Some(match first {
                            '+' => CalloutFold::Expanded,
                            '-' => CalloutFold::Collapsed,
                            _ => unreachable!(),
                        });
                    }
                    remaining = &remaining[first.len_utf8()..];
                    if remaining.is_empty() {
                        return Ok(());
                    }
                }

                let starts_with_ws = remaining
                    .chars()
                    .next()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(false);

                if !starts_with_ws {
                    if self.finalize_pending_callout_label_override() {
                        self.suppress_next_soft_break = true;
                    }
                    return Ok(());
                }

                self.pending_callout_label_buffer.push_str(remaining);
                return Ok(());
            }

            self.pending_callout_label_buffer.push_str(raw_text);
            return Ok(());
        }
        let mut callout_decision = None;
        if self.blockquote_level > 0
            && self.list_stack.is_empty()
            && let Some(state) = self.callout_stack.last_mut()
            && matches!(state, CalloutState::Pending)
        {
            if self.pending_callout_marker {
                self.pending_callout_marker_buffer.push_str(raw_text);
                let evaluation = Self::evaluate_callout_buffer(&self.pending_callout_marker_buffer);
                callout_decision = Some(Self::apply_callout_buffer_evaluation(
                    state,
                    evaluation,
                    &self.pending_callout_marker_buffer,
                ));
            } else if raw_text.trim().is_empty() {
                // Keep pending until we see meaningful content.
                return Ok(());
            } else if raw_text.trim_start().starts_with('[') {
                self.pending_callout_marker = true;
                self.pending_callout_marker_buffer.clear();
                self.pending_callout_marker_buffer.push_str(raw_text);
                let evaluation = Self::evaluate_callout_buffer(&self.pending_callout_marker_buffer);
                callout_decision = Some(Self::apply_callout_buffer_evaluation(
                    state,
                    evaluation,
                    &self.pending_callout_marker_buffer,
                ));
            } else {
                *state = CalloutState::None;
            }
        }

        if let Some(decision) = callout_decision {
            match decision {
                CalloutDecision::RenderHeader {
                    kind,
                    label,
                    label_override,
                    fold,
                    trailing,
                    suppress_paragraph_break,
                } => {
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    if matches!(self.config.callout_style.style, CalloutStyle::Pretty) {
                        self.content_indent = 0;
                        self.heading_indent = 0;
                    }
                    self.note_paragraph_content();
                    self.render_callout_header(kind, &label, label_override.as_deref(), fold);
                    if suppress_paragraph_break {
                        self.suppress_next_paragraph_break = true;
                    }
                    if let Some(trailing) = trailing {
                        if !trailing.trim().is_empty() {
                            self.note_paragraph_content();
                        }
                        self.process_text_with_wrapping_and_formatting(&trailing)?;
                        self.commit_pending_heading_placeholder_if_content();
                        self.suppress_next_soft_break = false;
                    } else {
                        self.suppress_next_soft_break = true;
                    }
                    return Ok(());
                }
                CalloutDecision::AwaitLabelOverride => {
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    if matches!(self.config.callout_style.style, CalloutStyle::Pretty) {
                        self.content_indent = 0;
                        self.heading_indent = 0;
                    }
                    self.pending_callout_label_override = true;
                    self.pending_callout_label_buffer.clear();
                    return Ok(());
                }
                CalloutDecision::FlushBuffer(buffer) => {
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    if !buffer.trim().is_empty() {
                        self.note_paragraph_content();
                    }
                    self.process_text_with_wrapping_and_formatting(&buffer)?;
                    self.commit_pending_heading_placeholder_if_content();
                    return Ok(());
                }
                CalloutDecision::Pending => {
                    return Ok(());
                }
            }
        }

        self.maybe_render_callout_header();
        if self.pending_task_marker && !self.list_stack.is_empty() {
            if self.pending_task_marker_buffer.is_empty() && !raw_text.starts_with('[') {
                self.pending_task_marker = false;
                self.pending_task_marker_buffer.clear();
                // Fall through to normal text handling when this isn't a task marker.
            } else {
                self.pending_task_marker_buffer.push_str(raw_text);
                if self.pending_task_marker_buffer.chars().count() < 3 {
                    return Ok(());
                }

                self.pending_task_marker = false;
                let buffer = std::mem::take(&mut self.pending_task_marker_buffer);
                if let Some((marker, remainder)) = self.split_custom_task_marker_prefix(&buffer) {
                    self.note_paragraph_content();
                    let style = create_style(self.theme, ThemeElement::ListMarker);
                    let styled_marker = style.apply(marker, self.config.no_colors);
                    self.output.push_str(&styled_marker);
                    if !remainder.is_empty() {
                        self.process_text_with_wrapping_and_formatting(remainder)?;
                    }
                } else {
                    // Process text with wrapping and formatting
                    if !buffer.trim().is_empty() {
                        self.note_paragraph_content();
                    }
                    self.process_text_with_wrapping_and_formatting(&buffer)?;
                }
                self.commit_pending_heading_placeholder_if_content();
                return Ok(());
            }
        }

        // Process text with wrapping and formatting
        if !raw_text.trim().is_empty() {
            self.note_paragraph_content();
        }
        self.process_text_with_wrapping_and_formatting(raw_text)?;
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    fn split_custom_task_marker_prefix<'b>(&self, text: &'b str) -> Option<(&'b str, &'b str)> {
        let bytes = text.as_bytes();
        if bytes.len() < 3 || bytes[0] != b'[' || bytes[2] != b']' {
            return None;
        }

        if !Self::is_supported_task_marker(bytes[1]) {
            return None;
        }

        let mut marker_end = 3;
        if bytes.len() == marker_end {
            return Some((&text[..marker_end], &text[marker_end..]));
        }

        while marker_end < bytes.len() {
            match bytes[marker_end] {
                b' ' | b'\t' => marker_end += 1,
                _ => break,
            }
        }

        if marker_end == 3 {
            return None;
        }

        Some((&text[..marker_end], &text[marker_end..]))
    }

    fn parse_callout_marker(text: &str) -> Option<CalloutMarker> {
        let trimmed = text.trim_start();
        if !trimmed.starts_with("[!") {
            return None;
        }

        let closing = trimmed.find(']')?;
        if closing < 2 {
            return None;
        }

        let kind_raw = trimmed[2..closing].trim();
        if kind_raw.is_empty() || !Self::is_valid_callout_kind(kind_raw) {
            return None;
        }

        let (kind, label) = Self::resolve_callout_kind(kind_raw);
        let mut rest = &trimmed[closing + 1..];
        let mut fold = None;

        if let Some(first) = rest.chars().next()
            && matches!(first, '+' | '-')
        {
            fold = Some(match first {
                '+' => CalloutFold::Expanded,
                '-' => CalloutFold::Collapsed,
                _ => unreachable!(),
            });
            rest = &rest[first.len_utf8()..];
        }

        if rest.is_empty() {
            return Some(CalloutMarker {
                kind,
                label,
                label_override: None,
                fold,
                trailing: None,
                allow_label_override: true,
                suppress_paragraph_break: false,
            });
        }

        let starts_with_ws = rest
            .chars()
            .next()
            .map(|ch| ch.is_whitespace())
            .unwrap_or(false);
        if starts_with_ws {
            let label_override_raw = rest.trim();
            if label_override_raw.is_empty() {
                return Some(CalloutMarker {
                    kind,
                    label,
                    label_override: None,
                    fold,
                    trailing: None,
                    allow_label_override: false,
                    suppress_paragraph_break: false,
                });
            }
            return Some(CalloutMarker {
                kind,
                label,
                label_override: Some(label_override_raw.to_string()),
                fold,
                trailing: None,
                allow_label_override: false,
                suppress_paragraph_break: false,
            });
        }

        Some(CalloutMarker {
            kind,
            label,
            label_override: None,
            fold,
            trailing: None,
            allow_label_override: false,
            suppress_paragraph_break: true,
        })
    }

    fn evaluate_callout_buffer(buffer: &str) -> CalloutBufferEval {
        let trimmed = buffer.trim_start();
        if !trimmed.starts_with('[') {
            return CalloutBufferEval::NotCallout;
        }

        if trimmed.len() >= 2 && !trimmed.starts_with("[!") {
            return CalloutBufferEval::NotCallout;
        }

        if trimmed.contains(']') {
            return match Self::parse_callout_marker(buffer) {
                Some(marker) => CalloutBufferEval::Callout(marker),
                None => CalloutBufferEval::NotCallout,
            };
        }

        if trimmed.len() > CALLOUT_BUFFER_LIMIT {
            return CalloutBufferEval::NotCallout;
        }

        CalloutBufferEval::Pending
    }

    fn apply_callout_buffer_evaluation(
        state: &mut CalloutState,
        evaluation: CalloutBufferEval,
        buffer: &str,
    ) -> CalloutDecision {
        match evaluation {
            CalloutBufferEval::Callout(marker) => {
                let has_label_override = marker
                    .label_override
                    .as_ref()
                    .map(|text| !text.trim().is_empty())
                    .unwrap_or(false);
                let defer_label_override = marker.allow_label_override && !has_label_override;
                let info = CalloutInfo {
                    kind: marker.kind,
                    label: marker.label.clone(),
                    label_override: marker.label_override.clone(),
                    fold: marker.fold,
                    header_rendered: !defer_label_override,
                    min_heading_indent: None,
                    inline_link_counter: 0,
                    inline_links: Vec::new(),
                };
                *state = CalloutState::Active(info);
                if defer_label_override {
                    CalloutDecision::AwaitLabelOverride
                } else {
                    CalloutDecision::RenderHeader {
                        kind: marker.kind,
                        label: marker.label,
                        label_override: marker.label_override,
                        fold: marker.fold,
                        trailing: marker.trailing,
                        suppress_paragraph_break: marker.suppress_paragraph_break,
                    }
                }
            }
            CalloutBufferEval::NotCallout => {
                *state = CalloutState::None;
                CalloutDecision::FlushBuffer(buffer.to_string())
            }
            CalloutBufferEval::Pending => CalloutDecision::Pending,
        }
    }

    fn is_valid_callout_kind(kind: &str) -> bool {
        kind.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    }

    fn resolve_callout_kind(raw: &str) -> (CalloutKind, String) {
        let lower = raw.trim().to_ascii_lowercase();
        let kind = match lower.as_str() {
            "note" | "seealso" => CalloutKind::Note,
            "abstract" | "summary" | "tldr" => CalloutKind::Abstract,
            "info" => CalloutKind::Info,
            "todo" => CalloutKind::Todo,
            "tip" | "hint" | "important" => CalloutKind::Tip,
            "success" | "check" | "done" => CalloutKind::Success,
            "question" | "help" | "faq" => CalloutKind::Question,
            "warning" | "caution" | "attention" => CalloutKind::Warning,
            "failure" | "fail" | "missing" => CalloutKind::Failure,
            "danger" | "error" => CalloutKind::Danger,
            "bug" => CalloutKind::Bug,
            "example" => CalloutKind::Example,
            "quote" | "cite" => CalloutKind::Quote,
            _ => CalloutKind::Tip,
        };

        (kind, lower)
    }

    fn is_supported_task_marker(marker: u8) -> bool {
        matches!(
            marker,
            b' ' | b'x' | b'X' | b'/' | b'-' | b'?' | b'\\' | b'|'
        )
    }

    /// Process text with wrapping and formatting, handling styled text properly
    fn process_text_with_wrapping_and_formatting(&mut self, text: &str) -> Result<()> {
        if !text.contains("==") {
            return self.process_segment_with_wrapping_and_formatting(
                text,
                false,
                self.table_state.is_some(),
            );
        }

        for segment in self.split_highlight_segments(text) {
            if !segment.text.is_empty() {
                self.process_segment_with_wrapping_and_formatting(
                    &segment.text,
                    segment.highlighted,
                    self.table_state.is_some(),
                )?;
            }
        }

        Ok(())
    }

    pub(super) fn process_segment_with_wrapping_and_formatting(
        &mut self,
        text: &str,
        highlighted: bool,
        is_table_cell: bool,
    ) -> Result<()> {
        // Check if this is for a table cell
        if is_table_cell {
            // For table cells, apply formatting directly without complex wrapping
            let formatted_text = self.apply_formatting_with_highlight(text, highlighted);
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(&formatted_text);
            }
            return Ok(());
        }

        // Add blockquote prefix if we're starting new content in a blockquote
        // Check if we're at the start of a line (after newline or any whitespace-only content)
        if self.blockquote_level > 0 {
            let after_newline = self.output.ends_with('\n');
            let at_start = self.output.is_empty();
            let at_line_start = if let Some(last_newline_pos) = self.output.rfind('\n') {
                // Check if everything after the last newline is just whitespace
                self.output[last_newline_pos + 1..].trim().is_empty()
            } else {
                // No newlines, check if entire output is just whitespace
                self.output.trim().is_empty()
            };

            if after_newline || at_start || at_line_start {
                let prefix = self.current_line_prefix();
                if !prefix.is_empty() {
                    self.output.push_str(&prefix);
                }
            }
        }

        // Check if we need to wrap text. When no explicit cols are provided,
        // wrap to the detected terminal width (unless --no-wrap is set).
        let should_wrap = self.config.is_text_wrapping_enabled();

        if should_wrap && !self.formatting_stack.is_empty() {
            // For styled text, prefer continuous decoration for strike-through
            if self.formatting_stack.contains(&ThemeElement::Strikethrough) {
                self.process_strikethrough_text_with_wrapping(text, highlighted)?;
            } else {
                // Default styled processing (per-unit formatting)
                self.process_styled_text_with_wrapping(text, highlighted)?;
            }
        } else {
            // Regular text processing
            self.process_regular_text(text, should_wrap, highlighted)?;
        }

        Ok(())
    }

    fn split_highlight_segments(&self, text: &str) -> Vec<HighlightSegment> {
        let mut segments = Vec::with_capacity(4);
        let mut buffer = String::new();
        let mut highlighted = false;
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '=' && matches!(chars.peek(), Some('=')) {
                chars.next(); // consume second '='
                if !buffer.is_empty() {
                    segments.push(HighlightSegment {
                        text: std::mem::take(&mut buffer),
                        highlighted,
                    });
                }
                highlighted = !highlighted;
                continue;
            }
            buffer.push(ch);
        }

        segments.push(HighlightSegment {
            text: if highlighted {
                format!("=={}", buffer)
            } else {
                buffer
            },
            highlighted,
        });

        segments
    }

    /// Process styled text with proper character/word-level wrapping like the original logic
    fn process_styled_text_with_wrapping(&mut self, text: &str, highlighted: bool) -> Result<()> {
        let terminal_width = self.effective_text_width();

        // The effective width is the full terminal width since current_line_width
        // already includes any indentation that's been added to the current line
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters) while preserving formatting
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process each unit individually with formatting
        for (i, unit) in units.iter().enumerate() {
            if unit.trim().is_empty() && i > 0 {
                // Handle whitespace between units
                let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                } else {
                    crate::utils::strip_ansi(&self.output)
                };
                let current_line_width = crate::utils::display_width(&current_line_clean);
                let space_width = crate::utils::display_width(unit);
                if current_line_width + space_width > effective_width {
                    self.push_newline_with_context();
                } else {
                    let formatted_unit = if highlighted {
                        self.apply_formatting_with_highlight(unit, true)
                    } else {
                        unit.to_string()
                    };
                    self.output.push_str(&formatted_unit);
                }
                continue;
            }

            // Check if adding this unit would exceed line width
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };

            let current_line_width = crate::utils::display_width(&current_line_clean);
            let unit_width = crate::utils::display_width(unit);

            // For InlineTable links, account for the reference number that will be added
            let additional_width = if self.in_link
                && matches!(
                    self.config.link_style,
                    LinkStyle::InlineTable | LinkStyle::EndTable
                ) {
                // Calculate the width of the reference number like [1], [2], etc.
                let reference_index = if matches!(self.config.link_style, LinkStyle::InlineTable) {
                    match self.callout_stack.last() {
                        Some(CalloutState::Active(info)) => info.inline_link_counter,
                        _ => self.paragraph_link_counter,
                    }
                } else {
                    self.paragraph_link_counter
                };
                let ref_num_str = format!("[{}]", reference_index);
                crate::utils::display_width(&ref_num_str)
            } else {
                0
            };

            let would_exceed = current_line_width + unit_width + additional_width > effective_width;

            // Force line break if needed (but not for the first unit on a line)
            if would_exceed
                && current_line_width > 0
                && Self::line_has_visible_text(&current_line_clean)
            {
                // Check if we should break before this unit
                let should_break = match wrap_mode {
                    crate::utils::WrapMode::Word => {
                        // For word wrapping, break before words (but not before punctuation)
                        !unit.trim_start().starts_with(',')
                            && !unit.trim_start().starts_with('.')
                            && !unit.trim_start().starts_with(';')
                            && !unit.trim_start().starts_with(':')
                            && !unit.trim_start().starts_with('!')
                            && !unit.trim_start().starts_with('?')
                            && !unit.trim_start().starts_with(')')
                            && !unit.trim_start().starts_with(']')
                            && !unit.trim_start().starts_with('}')
                    }
                    crate::utils::WrapMode::Character => true, // Always break for character mode
                    crate::utils::WrapMode::None => false,
                };

                if should_break {
                    // Centralized handler adds correct indent for lists, blockquotes, headings
                    self.push_newline_with_context();
                }
            }

            // Apply formatting and add to output
            let formatted_unit = self.apply_formatting_with_highlight(unit, highlighted);

            // Add content indentation for new lines if needed
            // But don't add it if we're continuing text on the same line (like after inline links)
            let should_add_indent = (self.output.ends_with('\n') || self.output.is_empty())
                && !formatted_unit.trim().is_empty();

            // Check if we're immediately after content that shouldn't get extra indentation
            let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                let line_content = &self.output[last_newline + 1..];
                // If the line has content (not just whitespace), we're continuing on the same line
                !line_content.trim().is_empty()
            } else {
                // No newlines, check if we have any content
                !self.output.trim().is_empty()
            };

            // Don't add indentation if we're continuing on the same line OR
            // if we just processed a link (which may have wrapped URLs)
            if should_add_indent && !after_inline_content {
                self.push_indent_for_line_start();
            }

            self.output.push_str(&formatted_unit);
        }

        Ok(())
    }

    /// Split text into words for word-based wrapping (for styled text)
    fn split_text_into_words_styled(&self, text: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut in_whitespace = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !in_whitespace && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
                in_whitespace = true;
            } else {
                if in_whitespace && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
                in_whitespace = false;
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
    }

    /// Split text into characters for character-based wrapping (for styled text)
    fn split_text_into_characters_styled(&self, text: &str) -> Vec<String> {
        text.chars().map(|c| c.to_string()).collect()
    }

    /// Calculate proper indentation for list content continuation lines
    pub(super) fn calculate_list_content_indent(&self) -> usize {
        let mut total_indent = 0;

        // Add heading content indentation
        total_indent += self.content_indent;

        // Add list nesting indentation (2 spaces per level)
        let indent_level = self.list_stack.len().saturating_sub(1);
        total_indent += indent_level * 2;

        // Add space for the list marker
        if let Some(list_state) = self.list_stack.last() {
            let marker_width = if list_state.is_ordered {
                // For ordered lists: "1. ", "2. ", etc. - typically 3 characters
                3
            } else {
                // For unordered lists: "- " - 2 characters
                2
            };
            total_indent += marker_width;
        }

        total_indent
    }

    /// Process text with underline formatting applied to continuous fragments between line breaks
    pub(super) fn process_underlined_text_with_wrapping(&mut self, text: &str) -> Result<()> {
        let should_wrap = self.config.is_text_wrapping_enabled();

        if !should_wrap {
            // No wrapping - just apply underline to entire text
            let formatted_text = if !self.config.no_colors {
                format!("\x1b[4m{}\x1b[0m", text)
            } else {
                text.to_string()
            };
            self.output.push_str(&formatted_text);
            return Ok(());
        }

        let terminal_width = self.effective_text_width();
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters)
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process units in groups - each group becomes one continuous underlined fragment
        let mut current_fragment = String::new();

        // Get initial line width
        let initial_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };
        let mut fragment_start_line_width = crate::utils::display_width(&initial_line_clean);

        // If there's no space left on the current line, move to a new one before adding any underlined text
        // If only 0 or 1 cells remain, start on a fresh line to avoid placing
        // a single dangling character at the line edge (which looks like overflow).
        if effective_width.saturating_sub(fragment_start_line_width) <= 1 && !text.trim().is_empty()
        {
            self.push_newline_with_context();

            // Account for full visual prefix on the new line (heading indent, list content
            // indent, blockquote pipes, etc.)
            fragment_start_line_width = self.compute_line_start_context_width();
        }

        for (i, unit) in units.iter().enumerate() {
            let is_ws = unit.trim().is_empty();
            let unit_width = crate::utils::display_width(unit);
            let current_fragment_width = crate::utils::display_width(&current_fragment);
            let would_exceed =
                fragment_start_line_width + current_fragment_width + unit_width > effective_width;

            // Special handling for whitespace: never allow trailing spaces to cause overflow
            if is_ws && i > 0 {
                if would_exceed && !current_fragment.trim().is_empty() {
                    // Flush current fragment and break line; drop the whitespace (no leading spaces)
                    let fragment_to_format = current_fragment.trim_end();
                    let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                    let formatted_fragment = if !self.config.no_colors {
                        format!("\x1b[4m{}\x1b[0m{}", fragment_to_format, trailing_spaces)
                    } else {
                        current_fragment.clone()
                    };
                    self.output.push_str(&formatted_fragment);

                    // Start new visual line with proper indent/prefix
                    self.push_newline_with_context();

                    fragment_start_line_width = self.compute_line_start_context_width();

                    current_fragment.clear();
                    continue; // Skip adding whitespace at the start of the new line
                } else {
                    // Safe to keep whitespace in the fragment
                    current_fragment.push_str(unit);
                    continue;
                }
            }

            if would_exceed && !current_fragment.trim().is_empty() {
                // We need to break - output current fragment first
                // Remove trailing spaces before applying underline to avoid underlined spaces at line end
                let fragment_to_format = current_fragment.trim_end();
                let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                let formatted_fragment = if !self.config.no_colors {
                    format!("\x1b[4m{}\x1b[0m{}", fragment_to_format, trailing_spaces)
                } else {
                    current_fragment.clone()
                };
                self.output.push_str(&formatted_fragment);

                // Check if we should break before this unit
                let should_break = match wrap_mode {
                    crate::utils::WrapMode::Word => {
                        // For word wrapping, break before words (but not before punctuation)
                        !unit.trim_start().starts_with(',')
                            && !unit.trim_start().starts_with('.')
                            && !unit.trim_start().starts_with(';')
                            && !unit.trim_start().starts_with(':')
                            && !unit.trim_start().starts_with('!')
                            && !unit.trim_start().starts_with('?')
                            && !unit.trim_start().starts_with(')')
                            && !unit.trim_start().starts_with(']')
                            && !unit.trim_start().starts_with('}')
                    }
                    crate::utils::WrapMode::Character => true, // Always break for character mode
                    crate::utils::WrapMode::None => false,
                };

                if should_break {
                    self.push_newline_with_context();

                    // Reset fragment tracking for new visual line
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                // Start new fragment with current unit
                current_fragment = unit.clone();
            } else {
                if would_exceed {
                    // Nothing in fragment yet but even this unit would exceed the line.
                    // Break the line first, then start with this unit.
                    self.push_newline_with_context();

                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                // Add unit to current fragment
                current_fragment.push_str(unit);
            }
        }

        // Output remaining fragment if any
        if !current_fragment.is_empty() {
            // Remove trailing spaces before applying underline to avoid underlined spaces at line end
            let fragment_to_format = current_fragment.trim_end();
            let trailing_spaces = &current_fragment[fragment_to_format.len()..];

            let formatted_fragment = if !self.config.no_colors {
                format!("\x1b[4m{}\x1b[0m{}", fragment_to_format, trailing_spaces)
            } else {
                current_fragment
            };
            self.output.push_str(&formatted_fragment);
        }

        Ok(())
    }

    /// Process text with strikethrough formatting applied as a continuous run (includes spaces)
    fn process_strikethrough_text_with_wrapping(
        &mut self,
        text: &str,
        highlighted: bool,
    ) -> Result<()> {
        let should_wrap = self.config.is_text_wrapping_enabled();

        if !should_wrap {
            // No wrapping - apply full formatting (including strikethrough) to entire text
            let formatted_text = self.apply_formatting_with_highlight(text, highlighted);
            self.output.push_str(&formatted_text);
            return Ok(());
        }

        let terminal_width = self.effective_text_width();
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters)
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process units in groups - each group becomes one continuous struck fragment
        let mut current_fragment = String::new();

        // Initial line width (without ANSI)
        let initial_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };
        let mut fragment_start_line_width = crate::utils::display_width(&initial_line_clean);

        // If little space left on the current line, move to a new one before adding any struck text
        if effective_width.saturating_sub(fragment_start_line_width) <= 1 && !text.trim().is_empty()
        {
            self.push_newline_with_context();
            fragment_start_line_width = self.compute_line_start_context_width();
        }

        for (i, unit) in units.iter().enumerate() {
            let is_ws = unit.trim().is_empty();
            let unit_width = crate::utils::display_width(unit);
            let current_fragment_width = crate::utils::display_width(&current_fragment);
            let would_exceed =
                fragment_start_line_width + current_fragment_width + unit_width > effective_width;

            // Whitespace handling: keep inside fragment unless it would overflow the line
            if is_ws && i > 0 {
                if would_exceed && !current_fragment.trim().is_empty() {
                    // Flush current fragment and break line; drop whitespace at new line start
                    let fragment_to_format = current_fragment.trim_end();
                    let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                    // Apply full formatting (includes strike) to the fragment; keep spaces highlighted when needed
                    let formatted_fragment = if highlighted {
                        self.apply_formatting_with_highlight(&current_fragment, true)
                    } else {
                        format!(
                            "{}{}",
                            self.apply_formatting(fragment_to_format),
                            trailing_spaces
                        )
                    };
                    self.output.push_str(&formatted_fragment);

                    // Start new visual line with correct context indentation
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();

                    current_fragment.clear();
                    continue;
                } else {
                    current_fragment.push_str(unit);
                    continue;
                }
            }

            if would_exceed && !current_fragment.trim().is_empty() {
                // Break: output current fragment first
                let fragment_to_format = current_fragment.trim_end();
                let trailing_spaces = &current_fragment[fragment_to_format.len()..];
                let formatted_fragment = if highlighted {
                    self.apply_formatting_with_highlight(&current_fragment, true)
                } else {
                    format!(
                        "{}{}",
                        self.apply_formatting(fragment_to_format),
                        trailing_spaces
                    )
                };
                self.output.push_str(&formatted_fragment);

                // Decide if we break before this unit (word wrap rules)
                let should_break = match wrap_mode {
                    crate::utils::WrapMode::Word => {
                        !unit.trim_start().starts_with(',')
                            && !unit.trim_start().starts_with('.')
                            && !unit.trim_start().starts_with(';')
                            && !unit.trim_start().starts_with(':')
                            && !unit.trim_start().starts_with('!')
                            && !unit.trim_start().starts_with('?')
                            && !unit.trim_start().starts_with(')')
                            && !unit.trim_start().starts_with(']')
                            && !unit.trim_start().starts_with('}')
                    }
                    crate::utils::WrapMode::Character => true,
                    crate::utils::WrapMode::None => false,
                };

                if should_break {
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment = unit.clone();
            } else {
                if would_exceed {
                    // Nothing in fragment yet, but unit would exceed -> break line first
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment.push_str(unit);
            }
        }

        // Output remaining fragment if any
        if !current_fragment.is_empty() {
            let fragment_to_format = current_fragment.trim_end();
            let trailing_spaces = &current_fragment[fragment_to_format.len()..];
            let formatted_fragment = if highlighted {
                self.apply_formatting_with_highlight(&current_fragment, true)
            } else {
                format!(
                    "{}{}",
                    self.apply_formatting(fragment_to_format),
                    trailing_spaces
                )
            };
            self.output.push_str(&formatted_fragment);
        }

        Ok(())
    }
    fn process_regular_text(
        &mut self,
        text: &str,
        should_wrap: bool,
        highlighted: bool,
    ) -> Result<()> {
        // Use the same word-by-word logic as styled text for consistent behavior
        if should_wrap {
            let terminal_width = self.effective_text_width();

            // Use full terminal width as effective width since current_line_width already includes indents
            let effective_width = terminal_width;

            // Determine wrap mode based on config
            let wrap_mode = self.config.text_wrap_mode();

            // Split text into wrappable units (words or characters)
            let units = match wrap_mode {
                crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
                crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
                crate::utils::WrapMode::None => vec![text.to_string()],
            };

            // Process each unit individually
            for unit in units.iter() {
                if unit.trim().is_empty() {
                    // Handle whitespace cautiously: don't let a trailing space overflow the line
                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };
                    let current_line_width = crate::utils::display_width(&current_line_clean);
                    let space_width = crate::utils::display_width(unit);
                    if current_line_width + space_width > effective_width {
                        // Break visual line and skip adding whitespace at start of next line
                        self.push_newline_with_context();
                    } else {
                        let formatted_unit = if highlighted {
                            self.apply_formatting_with_highlight(unit, true)
                        } else {
                            unit.to_string()
                        };
                        self.output.push_str(&formatted_unit);
                    }
                    continue;
                }

                // Check if adding this unit would exceed line width
                let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                } else {
                    crate::utils::strip_ansi(&self.output)
                };

                let current_line_width = crate::utils::display_width(&current_line_clean);
                let unit_width = crate::utils::display_width(unit);

                // For InlineTable links, account for the reference number that will be added
                let additional_width = if self.in_link
                    && matches!(
                        self.config.link_style,
                        LinkStyle::InlineTable | LinkStyle::EndTable
                    ) {
                    // Calculate the width of the reference number like [1], [2], etc.
                    let reference_index =
                        if matches!(self.config.link_style, LinkStyle::InlineTable) {
                            match self.callout_stack.last() {
                                Some(CalloutState::Active(info)) => info.inline_link_counter,
                                _ => self.paragraph_link_counter,
                            }
                        } else {
                            self.paragraph_link_counter
                        };
                    let ref_num_str = format!("[{}]", reference_index);
                    crate::utils::display_width(&ref_num_str)
                } else {
                    0
                };

                let would_exceed =
                    current_line_width + unit_width + additional_width > effective_width;

                // Force line break if needed (but not for the first unit on a line)
                if would_exceed
                    && current_line_width > 0
                    && Self::line_has_visible_text(&current_line_clean)
                {
                    // Check if we should break before this unit
                    let should_break = match wrap_mode {
                        crate::utils::WrapMode::Word => {
                            // For word wrapping, break before words (but not before punctuation)
                            !unit.trim_start().starts_with(',')
                                && !unit.trim_start().starts_with('.')
                                && !unit.trim_start().starts_with(';')
                                && !unit.trim_start().starts_with(':')
                                && !unit.trim_start().starts_with('!')
                                && !unit.trim_start().starts_with('?')
                                && !unit.trim_start().starts_with(')')
                                && !unit.trim_start().starts_with(']')
                                && !unit.trim_start().starts_with('}')
                        }
                        crate::utils::WrapMode::Character => true, // Always break for character mode
                        crate::utils::WrapMode::None => false,
                    };

                    if should_break {
                        self.push_newline_with_context();
                    }
                }

                // Apply formatting (no-op for regular text) and add to output
                let formatted_unit = self.apply_formatting_with_highlight(unit, highlighted);

                // Add content indentation for new lines if needed
                // But don't add it if we're continuing text on the same line (like after inline links)
                let should_add_indent = (self.output.ends_with('\n') || self.output.is_empty())
                    && !formatted_unit.trim().is_empty();

                // Check if we're immediately after content that shouldn't get extra indentation
                let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                    let line_content = &self.output[last_newline + 1..];
                    // If the line has content (not just whitespace), we're continuing on the same line
                    !line_content.trim().is_empty()
                } else {
                    // No newlines, check if we have any content
                    !self.output.trim().is_empty()
                };

                if should_add_indent && !after_inline_content {
                    self.push_indent_for_line_start();
                }

                self.output.push_str(&formatted_unit);
            }
        } else {
            // No wrapping - still ensure correct indentation at visual line starts
            let final_text = self.apply_formatting_with_highlight(text, highlighted);

            // Add content indentation for new visual lines when appropriate
            if (self.output.ends_with('\n') || self.output.is_empty())
                && !final_text.trim().is_empty()
            {
                // If the current line (after the last newline) already contains
                // non-whitespace content, we are continuing on the same line and
                // must not add extra indentation.
                let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                    let line_content = &self.output[last_newline + 1..];
                    !line_content.trim().is_empty()
                } else {
                    !self.output.trim().is_empty()
                };

                if !after_inline_content {
                    self.push_indent_for_line_start();
                }
            }

            self.output.push_str(&final_text);
        }

        Ok(())
    }
}
