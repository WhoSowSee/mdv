use super::*;

impl MarkdownProcessor {
    pub(super) fn ensure_task_list_termination(&self, content: &str) -> String {
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

    pub(super) fn normalize_backslash_checkbox(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len());
        let mut in_fence = false;
        let mut fence_char = '\0';
        let mut fence_len = 0usize;

        for line in &lines {
            let trimmed_start = line.trim_start();
            let indent_cols = Self::leading_indent_columns(line);

            if indent_cols <= 3
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

            if in_fence {
                result.push((*line).to_string());
                continue;
            }

            result.push(Self::fix_backslash_checkbox_line(line));
        }

        result.join("\n")
    }

    pub(super) fn fix_backslash_checkbox_line(line: &str) -> String {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
            i += 1;
        }
        if i >= bytes.len() {
            return line.to_string();
        }

        if matches!(bytes[i], b'-' | b'*' | b'+') {
            i += 1;
        } else if bytes[i].is_ascii_digit() {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if !matches!(bytes.get(i), Some(b'.') | Some(b')')) {
                return line.to_string();
            }
            i += 1;
        } else {
            return line.to_string();
        }

        let marker_end = i;
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i == marker_end {
            return line.to_string();
        }

        let is_backslash_checkbox = bytes.get(i) == Some(&b'[')
            && bytes.get(i + 1) == Some(&b'\\')
            && bytes.get(i + 2) == Some(&b']')
            && matches!(bytes.get(i + 3), None | Some(b' ') | Some(b'\t'));

        if !is_backslash_checkbox {
            return line.to_string();
        }

        let mut fixed = String::with_capacity(line.len() + 1);
        fixed.push_str(&line[..=i]);
        fixed.push('\\');
        fixed.push_str(&line[i + 1..]);
        fixed
    }
}
