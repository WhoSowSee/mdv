use super::*;

impl MarkdownProcessor {
    pub(super) fn separate_callout_markers_from_setext(&self, content: &str) -> String {
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

    pub(super) fn is_task_list_item(line: &str) -> bool {
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

    pub(super) fn is_list_item(line: &str) -> bool {
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

    pub(super) fn is_callout_marker_line(line: &str) -> bool {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("[!") {
            return false;
        }

        match trimmed.find(']') {
            Some(idx) => idx >= 2,
            None => false,
        }
    }

    pub(super) fn is_setext_underline_line(line: &str) -> bool {
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
}
