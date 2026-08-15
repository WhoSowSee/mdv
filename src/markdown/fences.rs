use super::*;

impl MarkdownProcessor {
    pub(super) fn detect_fence_marker(line: &str) -> Option<(char, usize)> {
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

    pub(super) fn leading_indent_columns(line: &str) -> usize {
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

    pub(super) fn leading_tab_count(line: &str) -> usize {
        line.as_bytes()
            .iter()
            .take_while(|&&byte| byte == b'\t')
            .count()
    }

    pub(super) fn strip_leading_tabs(line: &str, tabs: usize) -> Option<&str> {
        if Self::leading_tab_count(line) < tabs {
            None
        } else {
            Some(&line[tabs..])
        }
    }

    pub(super) fn strip_up_to_tabs(line: &str, tabs: usize) -> &str {
        let to_strip = Self::leading_tab_count(line).min(tabs);
        &line[to_strip..]
    }

    pub(super) fn canonical_fence_closing_line(marker: char, fence_len: usize) -> String {
        marker.to_string().repeat(fence_len.max(3))
    }

    pub(super) fn is_fence_closing_line(line: &str, marker: char, min_len: usize) -> bool {
        let trimmed = line.trim_start();
        let mut chars = trimmed.chars();
        let count = chars.by_ref().take_while(|ch| *ch == marker).count();
        if count < min_len {
            return false;
        }

        chars.all(|ch| ch.is_whitespace())
    }

    pub(super) fn normalize_tab_indented_fences(&self, content: &str) -> String {
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
}
