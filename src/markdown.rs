use crate::config::Config;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use std::mem;
use std::ops::Range;

pub(crate) const BLANK_LINE_MARKER: &str = "MDV_BLANK_LINE_MARKER";

/// Markdown processor that parses markdown and prepares it for rendering
pub struct MarkdownProcessor {
    config: Config,
    options: Options,
}

impl MarkdownProcessor {
    pub fn new(config: &Config) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_MATH);

        Self {
            config: config.clone(),
            options,
        }
    }

    pub fn parse(&self, markdown: &str) -> Result<Vec<Event<'static>>> {
        let content = self.preprocess_content(markdown)?;
        let parser = Parser::new_ext(&content, self.options).into_offset_iter();

        let events: Vec<(Event, Range<usize>)> = parser.collect();
        let events = self.postprocess_events(&content, events)?;
        let events = if self.config.reverse {
            self.reverse_events(events)
        } else {
            events
        };

        Ok(events)
    }

    fn preprocess_content(&self, content: &str) -> Result<String> {
        let mut processed = content.to_string();

        if let Some(from_text) = &self.config.from_text {
            processed = self.filter_from_text(&processed, from_text)?;
        }

        processed = self.normalize_tab_indented_fences(&processed);
        processed = self.normalize_explicit_blank_lines(&processed);
        processed = self.ensure_task_list_termination(&processed);
        processed = self.convert_admonitions_to_callouts(&processed);
        processed = self.separate_callout_markers_from_setext(&processed);
        processed = self.preprocess_blockquotes(&processed);

        Ok(processed)
    }

    fn filter_from_text(&self, content: &str, from_text: &str) -> Result<String> {
        // Parse from_text format: "Some Head:10" -> displays 10 lines after 'Some Head'
        let (search_text, max_lines) = if let Some((text, lines)) = from_text.split_once(':') {
            let max_lines = lines.parse::<usize>().unwrap_or(usize::MAX);
            (text, Some(max_lines))
        } else {
            (from_text, None)
        };

        let lines: Vec<&str> = content.lines().collect();

        let start_idx = if search_text.is_empty() {
            0
        } else {
            lines
                .iter()
                .position(|line| line.contains(search_text))
                .unwrap_or(0)
        };

        let end_idx = if let Some(max_lines) = max_lines {
            std::cmp::min(start_idx + max_lines, lines.len())
        } else {
            lines.len()
        };

        Ok(lines[start_idx..end_idx].join("\n"))
    }

    /// Preprocess blockquotes to ensure proper nesting behavior
    /// This fixes the issue where nested blockquotes with different levels
    /// are not properly closed by the markdown parser
    fn preprocess_blockquotes(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut last_level = 0;

        for line in lines {
            let (level, rest) = Self::split_blockquote_prefix(line);
            let rest_trimmed = rest.trim();

            // If this line has a blockquote but at a lower level than the previous,
            // add empty lines to properly close the higher levels
            if level > 0 && level < last_level {
                // Add empty lines to close the higher levels
                for _ in level..last_level {
                    result.push(String::new());
                }
            }
            // If this line has no blockquote but the previous line was a blockquote,
            // and this line is not empty, add an empty line to close the blockquote
            else if level == 0 && last_level > 0 && !rest_trimmed.is_empty() {
                // Add empty lines to close all blockquote levels
                for _ in 0..last_level {
                    result.push(String::new());
                }
            }

            result.push(line.to_string());

            if level > 0 {
                last_level = level;
            } else if !rest_trimmed.is_empty() {
                // Reset level when we encounter non-blockquote content
                last_level = 0;
            }
        }

        result.join("\n")
    }

    fn split_blockquote_prefix(line: &str) -> (usize, &str) {
        let (level, _prefix, rest) = Self::split_blockquote_prefix_parts(line);
        (level, rest)
    }

    fn normalize_explicit_blank_lines(&self, content: &str) -> String {
        let mut result = Vec::new();
        let mut in_fence = false;
        let mut fence_char = '\0';
        let mut fence_len = 0usize;
        let mut last_blank = false;

        for raw_line in content.lines() {
            let line = raw_line.trim_end_matches('\r');
            let trimmed_start = line.trim_start();
            let indent_columns = Self::leading_indent_columns(line);

            if indent_columns <= 3
                && let Some((marker, count)) = Self::detect_fence_marker(trimmed_start)
            {
                if in_fence && marker == fence_char && count >= fence_len {
                    in_fence = false;
                    fence_char = '\0';
                    fence_len = 0;
                } else if !in_fence {
                    in_fence = true;
                    fence_char = marker;
                    fence_len = count;
                }
                result.push(line.to_string());
                continue;
            }

            if in_fence {
                result.push(line.to_string());
                last_blank = false;
                continue;
            }

            let trimmed_end = line.trim_end();
            let trimmed = trimmed_end.trim();

            if trimmed == "\\" {
                let (level, prefix, _rest) = Self::split_blockquote_prefix_parts(line);
                let prefix = if level > 0 { prefix } else { String::new() };
                self.push_explicit_blank_line_marker(&mut result, &mut last_blank, &prefix);
                continue;
            }

            if trimmed_end.ends_with('\\') && trimmed_end.len() > 1 {
                let line_without_backslash = trimmed_end[..trimmed_end.len() - 1].to_string();
                let (level, prefix, rest) =
                    Self::split_blockquote_prefix_parts(&line_without_backslash);
                let prefix = if level > 0 { prefix } else { String::new() };
                if !rest.trim().is_empty() {
                    result.push(line_without_backslash);
                    last_blank = false;
                }
                self.push_explicit_blank_line_marker(&mut result, &mut last_blank, &prefix);
                continue;
            }

            if trimmed.is_empty() {
                result.push(String::new());
                last_blank = true;
                continue;
            }

            result.push(line.to_string());
            last_blank = false;
        }

        result.join("\n")
    }

    fn push_explicit_blank_line_marker(
        &self,
        result: &mut Vec<String>,
        last_blank: &mut bool,
        prefix: &str,
    ) {
        let prefix = prefix.to_string();
        if !*last_blank {
            result.push(prefix.clone());
        }
        if prefix.is_empty() {
            result.push(BLANK_LINE_MARKER.to_string());
            result.push(String::new());
        } else {
            result.push(format!("{}{}", prefix, BLANK_LINE_MARKER));
            result.push(prefix);
        }
        *last_blank = true;
    }

    fn split_blockquote_prefix_parts(line: &str) -> (usize, String, &str) {
        let trimmed = line.trim_start();
        let leading_ws_len = line.len().saturating_sub(trimmed.len());
        let bytes = trimmed.as_bytes();
        let mut idx = 0usize;
        let mut level = 0usize;

        while idx < bytes.len() && bytes[idx] == b'>' {
            level += 1;
            idx += 1;
            if idx < bytes.len() && bytes[idx] == b' ' {
                idx += 1;
            }
        }

        let prefix_len = leading_ws_len + idx;
        let prefix = line.get(..prefix_len).unwrap_or("").to_string();
        (level, prefix, &trimmed[idx..])
    }

    fn detect_fence_marker(line: &str) -> Option<(char, usize)> {
        let mut chars = line.chars();
        let first = chars.next()?;
        if first != '`' && first != '~' {
            return None;
        }

        let count = 1 + chars.take_while(|ch| *ch == first).count();
        if count >= 3 {
            Some((first, count))
        } else {
            None
        }
    }

    fn leading_indent_columns(line: &str) -> usize {
        let mut columns = 0usize;
        for ch in line.chars() {
            match ch {
                ' ' => columns += 1,
                '\t' => columns += 4 - (columns % 4),
                _ => break,
            }
        }
        columns
    }

    fn leading_tab_count(line: &str) -> usize {
        line.as_bytes()
            .iter()
            .take_while(|&&byte| byte == b'\t')
            .count()
    }

    fn strip_leading_tabs(line: &str, tabs: usize) -> Option<&str> {
        if Self::leading_tab_count(line) < tabs {
            None
        } else {
            Some(&line[tabs..])
        }
    }

    fn strip_up_to_tabs(line: &str, tabs: usize) -> &str {
        let to_strip = Self::leading_tab_count(line).min(tabs);
        &line[to_strip..]
    }

    fn canonical_fence_closing_line(marker: char, fence_len: usize) -> String {
        marker.to_string().repeat(fence_len.max(3))
    }

    fn is_fence_closing_line(line: &str, marker: char, min_len: usize) -> bool {
        let trimmed = line.trim_start();
        let mut chars = trimmed.chars();
        let count = chars.by_ref().take_while(|ch| *ch == marker).count();
        if count < min_len {
            return false;
        }

        chars.all(|ch| ch.is_whitespace())
    }

    fn normalize_tab_indented_fences(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len());
        let mut idx = 0usize;

        while idx < lines.len() {
            let line = lines[idx].trim_end_matches('\r');
            let opening_tabs = Self::leading_tab_count(line);
            if opening_tabs == 0 {
                result.push(line.to_string());
                idx += 1;
                continue;
            }

            let Some(opening_line) = Self::strip_leading_tabs(line, opening_tabs) else {
                result.push(line.to_string());
                idx += 1;
                continue;
            };

            let opening_trimmed = opening_line.trim_start();
            let Some((marker, fence_len)) = Self::detect_fence_marker(opening_trimmed) else {
                result.push(line.to_string());
                idx += 1;
                continue;
            };

            let mut closing_idx = None;
            let mut probe = idx + 1;
            while probe < lines.len() {
                let candidate = lines[probe].trim_end_matches('\r');
                let candidate_without_tabs = Self::strip_up_to_tabs(candidate, opening_tabs);
                if Self::is_fence_closing_line(candidate_without_tabs, marker, fence_len) {
                    closing_idx = Some(probe);
                    break;
                }
                probe += 1;
            }

            if let Some(close) = closing_idx {
                for (line_idx, block_line_raw) in lines.iter().enumerate().take(close + 1).skip(idx)
                {
                    let block_line = block_line_raw.trim_end_matches('\r');
                    if line_idx == close {
                        // Canonicalize closing fence so parser always recognizes it.
                        result.push(Self::canonical_fence_closing_line(marker, fence_len));
                    } else {
                        result.push(Self::strip_up_to_tabs(block_line, opening_tabs).to_string());
                    }
                }
                idx = close + 1;
                continue;
            }

            result.push(line.to_string());
            idx += 1;
        }

        result.join("\n")
    }

    fn ensure_task_list_termination(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len().saturating_add(4));
        let mut in_fence = false;
        let mut fence_char = '\0';
        let mut fence_len = 0usize;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed_start = line.trim_start();
            let indent_columns = Self::leading_indent_columns(line);

            if indent_columns <= 3
                && let Some((marker, count)) = Self::detect_fence_marker(trimmed_start)
            {
                if in_fence && marker == fence_char && count >= fence_len {
                    in_fence = false;
                    fence_char = '\0';
                    fence_len = 0;
                } else if !in_fence {
                    in_fence = true;
                    fence_char = marker;
                    fence_len = count;
                }
            }

            result.push((*line).to_string());

            if in_fence {
                continue;
            }

            if indent_columns > 0 || !Self::is_task_list_item(trimmed_start) {
                continue;
            }

            let mut next_idx = idx + 1;
            while next_idx < lines.len() && lines[next_idx].trim().is_empty() {
                next_idx += 1;
            }

            if next_idx >= lines.len() {
                continue;
            }

            let next_line = lines[next_idx];
            if next_line.trim() == BLANK_LINE_MARKER {
                continue;
            }

            let next_trimmed = next_line.trim_start();
            let next_indent_columns = Self::leading_indent_columns(next_line);
            if next_indent_columns == 0
                && !Self::is_list_item(next_trimmed)
                && !matches!(result.last(), Some(last) if last.is_empty())
            {
                result.push(String::new());
            }
        }

        result.join("\n")
    }

    fn convert_admonitions_to_callouts(&self, content: &str) -> String {
        enum AdmonitionState {
            Colon { fence_len: usize, base_ws: String },
            Bang { base_ws: String },
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len().saturating_add(4));
        let mut in_fence = false;
        let mut fence_char = '\0';
        let mut fence_len = 0usize;
        let mut admonition: Option<AdmonitionState> = None;

        for raw_line in lines {
            let line = raw_line.trim_end_matches('\r');
            let trimmed_start = line.trim_start();
            let leading_ws_len = line.len().saturating_sub(trimmed_start.len());
            let indent_columns = Self::leading_indent_columns(line);

            if indent_columns <= 3
                && let Some((marker, count)) = Self::detect_fence_marker(trimmed_start)
            {
                if in_fence && marker == fence_char && count >= fence_len {
                    in_fence = false;
                    fence_char = '\0';
                    fence_len = 0;
                } else if !in_fence {
                    in_fence = true;
                    fence_char = marker;
                    fence_len = count;
                }
                result.push(line.to_string());
                continue;
            }

            if in_fence {
                result.push(line.to_string());
                continue;
            }

            if let Some(state) = &admonition {
                match state {
                    AdmonitionState::Colon { fence_len, base_ws } => {
                        if Self::is_colon_fence_line(trimmed_start, *fence_len) {
                            admonition = None;
                            continue;
                        }

                        let content_line = line.strip_prefix(base_ws).unwrap_or(line);
                        if content_line.trim().is_empty() {
                            result.push(format!("{}>", base_ws));
                        } else {
                            result.push(format!("{}> {}", base_ws, content_line));
                        }
                        continue;
                    }
                    AdmonitionState::Bang { base_ws } => {
                        if trimmed_start.is_empty() {
                            admonition = None;
                            result.push(line.to_string());
                            continue;
                        }

                        let content_line = line.strip_prefix(base_ws).unwrap_or(line);
                        if content_line.trim().is_empty() {
                            result.push(format!("{}>", base_ws));
                        } else {
                            result.push(format!("{}> {}", base_ws, content_line));
                        }
                        continue;
                    }
                }
            }

            if let Some((kind, title, fence_len)) =
                Self::parse_colon_admonition_start(trimmed_start)
            {
                let base_ws = &line[..leading_ws_len];
                result.push(Self::format_callout_marker_line(
                    base_ws,
                    &kind,
                    title.as_deref(),
                ));
                admonition = Some(AdmonitionState::Colon {
                    fence_len,
                    base_ws: base_ws.to_string(),
                });
                continue;
            }

            if let Some((kind, title)) = Self::parse_bang_admonition_start(trimmed_start) {
                let base_ws = &line[..leading_ws_len];
                result.push(Self::format_callout_marker_line(
                    base_ws,
                    &kind,
                    title.as_deref(),
                ));
                admonition = Some(AdmonitionState::Bang {
                    base_ws: base_ws.to_string(),
                });
                continue;
            }

            result.push(line.to_string());
        }

        result.join("\n")
    }

    fn parse_colon_admonition_start(line: &str) -> Option<(String, Option<String>, usize)> {
        let mut count = 0usize;
        for ch in line.chars() {
            if ch == ':' {
                count += 1;
            } else {
                break;
            }
        }

        if count < 3 {
            return None;
        }

        let rest = line[count..].trim_start();
        if rest.is_empty() {
            return None;
        }

        let (kind, title) = Self::parse_admonition_kind_and_title(rest)?;
        Some((kind, title, count))
    }

    fn parse_bang_admonition_start(line: &str) -> Option<(String, Option<String>)> {
        let mut count = 0usize;
        for ch in line.chars() {
            if ch == '!' {
                count += 1;
            } else {
                break;
            }
        }

        if count < 3 {
            return None;
        }

        let rest = line[count..].trim_start();
        if rest.is_empty() {
            return None;
        }

        Self::parse_admonition_kind_and_title(rest)
    }

    fn parse_admonition_kind_and_title(input: &str) -> Option<(String, Option<String>)> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix('{') {
            let end = rest.find('}')?;
            let kind = rest[..end].trim();
            if kind.is_empty() || !Self::is_valid_callout_kind(kind) {
                return None;
            }
            let title_raw = rest[end + 1..].trim();
            let title = if title_raw.is_empty() {
                None
            } else {
                Some(title_raw.to_string())
            };
            return Some((kind.to_string(), title));
        }

        let mut split_idx = None;
        for (idx, ch) in trimmed.char_indices() {
            if ch.is_whitespace() {
                split_idx = Some(idx);
                break;
            }
        }

        let (kind, title_raw) = match split_idx {
            Some(idx) => (&trimmed[..idx], trimmed[idx..].trim()),
            None => (trimmed, ""),
        };

        if kind.is_empty() || !Self::is_valid_callout_kind(kind) {
            return None;
        }

        let title = if title_raw.is_empty() {
            None
        } else {
            Some(title_raw.to_string())
        };

        Some((kind.to_string(), title))
    }

    fn format_callout_marker_line(base_ws: &str, kind: &str, title: Option<&str>) -> String {
        let mut line = String::new();
        line.push_str(base_ws);
        line.push_str("> [!");
        line.push_str(kind);
        line.push(']');
        if let Some(title) = title {
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                line.push(' ');
                line.push_str(trimmed);
            }
        }
        line
    }

    fn is_valid_callout_kind(kind: &str) -> bool {
        kind.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    }

    fn is_colon_fence_line(line: &str, fence_len: usize) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        let count = trimmed.chars().filter(|ch| *ch == ':').count();
        count >= fence_len && trimmed.chars().all(|ch| ch == ':')
    }

    fn separate_callout_markers_from_setext(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len().saturating_add(4));
        let mut in_fence = false;
        let mut fence_char = '\0';
        let mut fence_len = 0usize;

        for idx in 0..lines.len() {
            let line = lines[idx];
            let trimmed_start = line.trim_start();
            let indent_columns = Self::leading_indent_columns(line);

            if indent_columns <= 3
                && let Some((marker, count)) = Self::detect_fence_marker(trimmed_start)
            {
                if in_fence && marker == fence_char && count >= fence_len {
                    in_fence = false;
                    fence_char = '\0';
                    fence_len = 0;
                } else if !in_fence {
                    in_fence = true;
                    fence_char = marker;
                    fence_len = count;
                }
                result.push(line.to_string());
                continue;
            }

            result.push(line.to_string());

            if in_fence {
                continue;
            }

            let (level, rest) = Self::split_blockquote_prefix(line);
            if level == 0 {
                continue;
            }

            let rest_trimmed = rest.trim();
            if !Self::is_callout_marker_line(rest_trimmed) {
                continue;
            }

            let next_idx = idx + 1;
            let underline_idx = idx + 2;
            if underline_idx >= lines.len() {
                continue;
            }

            let (next_level, next_rest) = Self::split_blockquote_prefix(lines[next_idx]);
            let (underline_level, underline_rest) =
                Self::split_blockquote_prefix(lines[underline_idx]);
            if next_level != level || underline_level != level {
                continue;
            }

            if next_rest.trim().is_empty() {
                continue;
            }

            if !Self::is_setext_underline_line(underline_rest.trim()) {
                continue;
            }

            let leading_ws_len = line.len().saturating_sub(line.trim_start().len());
            let leading_ws = &line[..leading_ws_len];
            let mut blank = String::new();
            blank.push_str(leading_ws);
            blank.push_str(&">".repeat(level));
            result.push(blank);
        }

        result.join("\n")
    }

    fn is_task_list_item(line: &str) -> bool {
        let mut chars = line.chars();
        let first = match chars.next() {
            Some(ch) => ch,
            None => return false,
        };

        if !matches!(first, '-' | '+' | '*') {
            return false;
        }

        if chars.next() != Some(' ') {
            return false;
        }

        if chars.next() != Some('[') {
            return false;
        }

        let marker = match chars.next() {
            Some(ch) => ch,
            None => return false,
        };

        if !matches!(marker, ' ' | 'x' | 'X' | '/' | '-' | '?' | '\\' | '|') {
            return false;
        }

        if chars.next() != Some(']') {
            return false;
        }

        matches!(chars.next(), Some(' ') | Some('\t'))
    }

    fn is_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("+ ") || trimmed.starts_with("* ") {
            return true;
        }

        let mut chars = trimmed.chars().peekable();
        let mut saw_digit = false;
        while let Some(ch) = chars.peek().copied() {
            if ch.is_ascii_digit() {
                saw_digit = true;
                chars.next();
            } else {
                break;
            }
        }

        if !saw_digit || chars.next() != Some('.') {
            return false;
        }

        matches!(chars.next(), Some(' ') | Some('\t'))
    }

    fn is_callout_marker_line(line: &str) -> bool {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("[!") {
            return false;
        }

        match trimmed.find(']') {
            Some(idx) => idx >= 2,
            None => false,
        }
    }

    fn is_setext_underline_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }

        let mut chars = trimmed.chars();
        let first = match chars.next() {
            Some(ch) => ch,
            None => return false,
        };

        if first != '-' && first != '=' {
            return false;
        }

        chars.all(|ch| ch == first)
    }

    fn postprocess_events(
        &self,
        content: &str,
        events: Vec<(Event, Range<usize>)>,
    ) -> Result<Vec<Event<'static>>> {
        let mut processed = Vec::with_capacity(events.len());

        let mut idx = 0usize;
        while idx < events.len() {
            if let Some((Event::Start(Tag::Paragraph), _)) = events.get(idx)
                && let (Some((Event::Text(text), _)), Some((Event::End(TagEnd::Paragraph), _))) =
                    (events.get(idx + 1), events.get(idx + 2))
                && text.as_ref().trim() == BLANK_LINE_MARKER
            {
                processed.push(Event::Html(BLANK_LINE_MARKER.into()));
                idx += 3;
                continue;
            }

            if let Some((Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)), start_range)) =
                events.get(idx)
                && let Some(end_idx) = Self::find_code_block_end_index(&events, idx + 1)
                && Self::is_plain_indented_code_block_start(content, start_range.start)
            {
                self.push_demoted_code_block_events(&mut processed, &events[idx + 1..end_idx])?;
                idx = end_idx + 1;
                continue;
            }

            let (event, _range) = &events[idx];
            match event {
                Event::Start(tag) => {
                    processed.push(Event::Start(self.convert_tag_to_static(tag.clone())));
                }
                Event::End(tag_end) => {
                    processed.push(Event::End(*tag_end));
                }
                Event::Text(text) => {
                    let processed_text = self.process_text(text);
                    processed.push(Event::Text(processed_text.to_string().into()));
                }
                Event::Code(code) => {
                    processed.push(Event::Code(self.expand_tabs(code.as_ref()).into()));
                }
                other => processed.push(self.convert_to_static(other.clone())),
            }
            idx += 1;
        }

        Ok(processed)
    }

    fn find_code_block_end_index(
        events: &[(Event, Range<usize>)],
        start_idx: usize,
    ) -> Option<usize> {
        events
            .iter()
            .enumerate()
            .skip(start_idx)
            .find_map(|(idx, (event, _))| {
                if matches!(event, Event::End(TagEnd::CodeBlock)) {
                    Some(idx)
                } else {
                    None
                }
            })
    }

    fn is_plain_indented_code_block_start(content: &str, start_offset: usize) -> bool {
        let safe_start = start_offset.min(content.len());
        let line_start = content[..safe_start].rfind('\n').map_or(0, |idx| idx + 1);
        let indent = &content[line_start..safe_start];

        !indent.is_empty() && indent.chars().all(|ch| ch == ' ' || ch == '\t')
    }

    fn push_demoted_code_block_events(
        &self,
        processed: &mut Vec<Event<'static>>,
        events: &[(Event, Range<usize>)],
    ) -> Result<()> {
        let mut text = String::new();
        for (event, _) in events {
            match event {
                Event::Text(chunk) => text.push_str(chunk.as_ref()),
                Event::SoftBreak | Event::HardBreak => text.push('\n'),
                _ => {}
            }
        }

        let text = text.trim_end_matches('\n');
        if text.is_empty() {
            return Ok(());
        }

        let parser = Parser::new_ext(text, self.options).into_offset_iter();
        let reparsed_events: Vec<(Event, Range<usize>)> = parser.collect();
        processed.extend(self.postprocess_events(text, reparsed_events)?);
        Ok(())
    }

    fn reverse_events(&self, events: Vec<Event<'static>>) -> Vec<Event<'static>> {
        if events.is_empty() {
            return events;
        }

        let mut segments: Vec<Vec<Event<'static>>> = Vec::new();
        let mut current: Vec<Event<'static>> = Vec::new();
        let mut depth = 0usize;

        for event in events {
            match event {
                Event::Start(_) => {
                    if depth == 0 && !current.is_empty() {
                        segments.push(mem::take(&mut current));
                    }
                    depth += 1;
                    current.push(event);
                }
                Event::End(_) => {
                    current.push(event);
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        segments.push(mem::take(&mut current));
                    }
                }
                _ => {
                    current.push(event);
                    if depth == 0 {
                        segments.push(mem::take(&mut current));
                    }
                }
            }
        }

        if !current.is_empty() {
            segments.push(current);
        }

        segments.reverse();
        segments.into_iter().flatten().collect()
    }

    fn convert_to_static(&self, event: Event) -> Event<'static> {
        match event {
            Event::Start(tag) => Event::Start(self.convert_tag_to_static(tag)),
            Event::End(tag_end) => Event::End(tag_end),
            Event::Text(text) => Event::Text(text.to_string().into()),
            Event::Code(code) => Event::Code(code.to_string().into()),
            Event::Html(html) => Event::Html(html.to_string().into()),
            Event::InlineHtml(html) => Event::InlineHtml(html.to_string().into()),
            Event::FootnoteReference(name) => Event::FootnoteReference(name.to_string().into()),
            Event::SoftBreak => Event::SoftBreak,
            Event::HardBreak => Event::HardBreak,
            Event::Rule => Event::Rule,
            Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
            Event::InlineMath(math) => Event::InlineMath(math.to_string().into()),
            Event::DisplayMath(math) => Event::DisplayMath(math.to_string().into()),
        }
    }

    fn convert_tag_to_static(&self, tag: Tag) -> Tag<'static> {
        match tag {
            Tag::Paragraph => Tag::Paragraph,
            Tag::Heading {
                level,
                id,
                classes,
                attrs,
            } => Tag::Heading {
                level,
                id: id.map(|s| s.to_string().into()),
                classes: classes.into_iter().map(|s| s.to_string().into()).collect(),
                attrs: attrs
                    .into_iter()
                    .map(|(k, v)| (k.to_string().into(), v.map(|s| s.to_string().into())))
                    .collect(),
            },
            Tag::BlockQuote(kind) => Tag::BlockQuote(kind),
            Tag::CodeBlock(kind) => {
                let static_kind = match kind {
                    CodeBlockKind::Indented => CodeBlockKind::Indented,
                    CodeBlockKind::Fenced(lang) => CodeBlockKind::Fenced(lang.to_string().into()),
                };
                Tag::CodeBlock(static_kind)
            }
            Tag::List(start) => Tag::List(start),
            Tag::Item => Tag::Item,
            Tag::FootnoteDefinition(name) => Tag::FootnoteDefinition(name.to_string().into()),
            Tag::Table(alignments) => Tag::Table(alignments),
            Tag::TableHead => Tag::TableHead,
            Tag::TableRow => Tag::TableRow,
            Tag::TableCell => Tag::TableCell,
            Tag::Emphasis => Tag::Emphasis,
            Tag::Strong => Tag::Strong,
            Tag::Strikethrough => Tag::Strikethrough,
            Tag::Superscript => Tag::Superscript,
            Tag::Subscript => Tag::Subscript,
            Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Link {
                link_type,
                dest_url: dest_url.to_string().into(),
                title: title.to_string().into(),
                id: id.to_string().into(),
            },
            Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Image {
                link_type,
                dest_url: dest_url.to_string().into(),
                title: title.to_string().into(),
                id: id.to_string().into(),
            },
            Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
            Tag::HtmlBlock => Tag::HtmlBlock,
            Tag::DefinitionList => Tag::DefinitionList,
            Tag::DefinitionListTitle => Tag::DefinitionListTitle,
            Tag::DefinitionListDefinition => Tag::DefinitionListDefinition,
        }
    }

    fn process_text<'a>(&self, text: &CowStr<'a>) -> CowStr<'a> {
        if text.as_ref().contains('\t') {
            self.expand_tabs(text.as_ref()).into()
        } else {
            text.clone()
        }
    }

    fn expand_tabs(&self, text: &str) -> String {
        text.replace('\t', &" ".repeat(self.config.tab_length))
    }
}

/// Extract language from code block
pub fn extract_code_language(kind: &CodeBlockKind) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(lang) => {
            let lang = lang.trim();
            if lang.is_empty() {
                None
            } else {
                // Handle language-specific prefixes
                let lang = if let Some(stripped) = lang.strip_prefix("language-") {
                    stripped
                } else {
                    lang
                };
                Some(lang.to_string())
            }
        }
        CodeBlockKind::Indented => None,
    }
}

/// Check if content looks like source code based on file extension or content
pub fn detect_source_code(content: &str, filename: Option<&str>) -> Option<String> {
    // Check file extension first
    if let Some(filename) = filename
        && let Some(ext) = std::path::Path::new(filename).extension()
        && let Some(ext_str) = ext.to_str()
    {
        return match ext_str.to_lowercase().as_str() {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" => Some("typescript".to_string()),
            "go" => Some("go".to_string()),
            "c" => Some("c".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "java" => Some("java".to_string()),
            "rb" => Some("ruby".to_string()),
            "php" => Some("php".to_string()),
            "sh" | "bash" => Some("bash".to_string()),
            "sql" => Some("sql".to_string()),
            "json" => Some("json".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "xml" => Some("xml".to_string()),
            "html" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            _ => None,
        };
    }

    // Try to detect from content patterns
    let lines: Vec<&str> = content.lines().take(10).collect();

    // Look for shebangs
    if let Some(first_line) = lines.first()
        && first_line.starts_with("#!")
    {
        if first_line.contains("python") {
            return Some("python".to_string());
        } else if first_line.contains("bash") || first_line.contains("sh") {
            return Some("bash".to_string());
        } else if first_line.contains("node") {
            return Some("javascript".to_string());
        }
    }

    // Look for common patterns
    for line in &lines {
        let line = line.trim();

        // Python patterns
        if line.starts_with("def ")
            || line.starts_with("class ")
            || line.starts_with("import ")
            || line.starts_with("from ")
        {
            return Some("python".to_string());
        }

        // Rust patterns
        if line.starts_with("fn ")
            || line.starts_with("struct ")
            || line.starts_with("impl ")
            || line.starts_with("use ")
        {
            return Some("rust".to_string());
        }

        // JavaScript/TypeScript patterns
        if line.starts_with("function ")
            || line.starts_with("const ")
            || line.starts_with("let ")
            || line.starts_with("var ")
        {
            return Some("javascript".to_string());
        }

        // Go patterns
        if line.starts_with("package ") || line.starts_with("func ") {
            return Some("go".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use pulldown_cmark::HeadingLevel;

    #[test]
    fn test_markdown_parsing() {
        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);

        let markdown = "# Hello\n\nThis is **bold** text.";
        let events = processor.parse(markdown).unwrap();

        assert!(!events.is_empty());
    }

    #[test]
    fn test_filter_from_text() {
        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);

        let content = "Line 1\nTarget Line\nLine 3\nLine 4";
        let result = processor.filter_from_text(content, "Target:2").unwrap();

        assert_eq!(result, "Target Line\nLine 3");
    }

    #[test]
    fn test_extract_code_language() {
        let fenced = CodeBlockKind::Fenced("rust".into());
        assert_eq!(extract_code_language(&fenced), Some("rust".to_string()));

        let indented = CodeBlockKind::Indented;
        assert_eq!(extract_code_language(&indented), None);
    }

    #[test]
    fn test_detect_source_code() {
        // Test file extension detection
        assert_eq!(
            detect_source_code("", Some("test.rs")),
            Some("rust".to_string())
        );
        assert_eq!(
            detect_source_code("", Some("test.py")),
            Some("python".to_string())
        );

        // Test content detection
        let python_code = "def hello():\n    print('world')";
        assert_eq!(
            detect_source_code(python_code, None),
            Some("python".to_string())
        );

        let rust_code = "fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(
            detect_source_code(rust_code, None),
            Some("rust".to_string())
        );
    }

    #[test]
    fn setext_dashes_create_h2() {
        use pulldown_cmark::{Event, Tag, TagEnd};

        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);

        let markdown = "Text\n---\nNext\n";
        let events = processor.parse(markdown).unwrap();

        assert!(matches!(
            events.as_slice(),
            [
                Event::Start(Tag::Heading { level: HeadingLevel::H2, .. }),
                Event::Text(first),
                Event::End(TagEnd::Heading(HeadingLevel::H2)),
                Event::Start(Tag::Paragraph),
                Event::Text(second),
                Event::End(TagEnd::Paragraph)
            ] if first.as_ref() == "Text" && second.as_ref() == "Next"
        ));
    }

    #[test]
    fn setext_equals_create_h1() {
        use pulldown_cmark::{Event, Tag, TagEnd};

        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);

        let markdown = "Title\n===\nBody\n";
        let events = processor.parse(markdown).unwrap();

        assert!(matches!(
            events.as_slice(),
            [
                Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }),
                Event::Text(first),
                Event::End(TagEnd::Heading(HeadingLevel::H1)),
                Event::Start(Tag::Paragraph),
                Event::Text(second),
                Event::End(TagEnd::Paragraph)
            ] if first.as_ref() == "Title" && second.as_ref() == "Body"
        ));
    }

    #[test]
    fn top_level_tab_indented_text_is_not_code_block() {
        use pulldown_cmark::{Event, Tag, TagEnd};

        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);
        let events = processor.parse("\tTest text\n").unwrap();

        assert!(matches!(
            events.as_slice(),
            [
                Event::Start(Tag::Paragraph),
                Event::Text(text),
                Event::End(TagEnd::Paragraph)
            ] if text.as_ref() == "Test text"
        ));
    }

    #[test]
    fn top_level_space_indented_text_is_not_code_block() {
        use pulldown_cmark::{Event, Tag, TagEnd};

        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);
        let events = processor.parse("    Test text\n").unwrap();

        assert!(matches!(
            events.as_slice(),
            [
                Event::Start(Tag::Paragraph),
                Event::Text(text),
                Event::End(TagEnd::Paragraph)
            ] if text.as_ref() == "Test text"
        ));
    }

    #[test]
    fn top_level_indented_atx_headings_are_headings() {
        use pulldown_cmark::{Event, Tag, TagEnd};

        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);
        let events = processor
            .parse("    # Space heading\n\t# Tab heading\n")
            .unwrap();

        assert!(matches!(
            events.as_slice(),
            [
                Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }),
                Event::Text(first),
                Event::End(TagEnd::Heading(HeadingLevel::H1)),
                Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }),
                Event::Text(second),
                Event::End(TagEnd::Heading(HeadingLevel::H1))
            ] if first.as_ref() == "Space heading" && second.as_ref() == "Tab heading"
        ));
    }
}
