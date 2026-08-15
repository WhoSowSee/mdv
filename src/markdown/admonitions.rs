use super::*;

impl MarkdownProcessor {
    pub(super) fn convert_admonitions_to_callouts(&self, content: &str) -> String {
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

    pub(super) fn parse_colon_admonition_start(
        line: &str,
    ) -> Option<(String, Option<String>, usize)> {
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

    pub(super) fn parse_bang_admonition_start(line: &str) -> Option<(String, Option<String>)> {
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

    pub(super) fn parse_admonition_kind_and_title(input: &str) -> Option<(String, Option<String>)> {
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

    pub(super) fn format_callout_marker_line(
        base_ws: &str,
        kind: &str,
        title: Option<&str>,
    ) -> String {
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

    pub(super) fn is_valid_callout_kind(kind: &str) -> bool {
        kind.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    }

    pub(super) fn is_colon_fence_line(line: &str, fence_len: usize) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        let count = trimmed.chars().filter(|ch| *ch == ':').count();
        count >= fence_len && trimmed.chars().all(|ch| ch == ':')
    }
}
