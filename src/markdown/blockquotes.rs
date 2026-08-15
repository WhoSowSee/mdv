use super::*;

impl MarkdownProcessor {
    /// Preprocess blockquotes to ensure proper nesting behavior.
    pub(super) fn preprocess_blockquotes(&self, content: &str) -> String {
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

    pub(super) fn split_blockquote_prefix(line: &str) -> (usize, &str) {
        let (level, _prefix, rest) = Self::split_blockquote_prefix_parts(line);
        (level, rest)
    }

    pub(super) fn normalize_explicit_blank_lines(&self, content: &str) -> String {
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

    pub(super) fn push_explicit_blank_line_marker(
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

    pub(super) fn split_blockquote_prefix_parts(line: &str) -> (usize, String, &str) {
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
}
