use super::input::skip_comment;
use super::*;
use crate::math::ast::MathEnvironment;

impl MathParser<'_> {
    pub(super) fn parse_environment(&mut self, start: usize) -> MathNode {
        if self.depth >= MAX_MATH_DEPTH {
            self.diagnostic("maximum environment depth exceeded", true);
            return MathNode::Unsupported(self.input[start..].to_string());
        }
        let (name, _) = self.parse_required_raw_group_with_offset("begin");
        if name.is_empty() {
            return MathNode::Unsupported(self.input[start..self.pos].to_string());
        }
        let content_start = self.pos;
        let content = self.consume_environment_content(&name);
        if !is_supported_environment(&name) {
            self.diagnostic(&format!("unsupported environment {name}"), false);
            return MathNode::Unsupported(self.input[start..self.pos].to_string());
        }

        let (alignment, content, content_offset) = if name == "array" {
            take_array_alignment(&content)
        } else {
            (None, content.as_str(), 0)
        };
        self.environment_from_content(
            &name,
            alignment,
            content,
            self.base_offset + content_start + content_offset,
        )
    }

    pub(super) fn environment_from_content(
        &mut self,
        name: &str,
        alignment: Option<String>,
        content: &str,
        content_offset: usize,
    ) -> MathNode {
        let rows = split_environment(content)
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|range| {
                        let mut parser =
                            MathParser::new(&content[range.clone()], content_offset + range.start);
                        parser.depth = self.depth.saturating_add(1);
                        parser.preserve_whitespace = self.preserve_whitespace;
                        let node = parser.parse_sequence(None, false);
                        self.diagnostics.extend(parser.diagnostics);
                        node
                    })
                    .collect()
            })
            .collect();

        MathNode::Environment(MathEnvironment {
            name: name.to_string(),
            alignment,
            rows,
        })
    }

    fn consume_environment_content(&mut self, expected: &str) -> String {
        let start = self.pos;
        let mut cursor = self.pos;
        let mut stack = vec![expected.to_string()];

        while cursor < self.input.len() {
            let character = self.input[cursor..]
                .chars()
                .next()
                .expect("valid character boundary");
            if character == '%' {
                skip_comment(self.input, &mut cursor);
                continue;
            }
            if character != '\\' {
                cursor += character.len_utf8();
                continue;
            }

            if self.input[cursor..].starts_with("\\\\") {
                cursor += 2;
                continue;
            }
            let Some((opening, name, command_end)) = environment_marker(self.input, cursor) else {
                cursor += character.len_utf8();
                if let Some(escaped) = self.input[cursor..].chars().next() {
                    cursor += escaped.len_utf8();
                }
                continue;
            };

            if opening {
                stack.push(name.to_string());
            } else if stack.last().is_some_and(|current| current == name) {
                stack.pop();
                if stack.is_empty() {
                    let content = self.input[start..cursor].to_string();
                    self.pos = command_end;
                    return content;
                }
            } else {
                self.pos = command_end;
                self.diagnostic(&format!("mismatched \\end{{{name}}}"), true);
            }
            cursor = command_end;
        }

        self.pos = self.input.len();
        self.diagnostic(&format!("missing \\end{{{expected}}}"), true);
        self.input[start..].to_string()
    }
}

fn is_supported_environment(name: &str) -> bool {
    matches!(
        name,
        "align"
            | "align*"
            | "aligned"
            | "eqnarray"
            | "split"
            | "gather"
            | "gather*"
            | "gathered"
            | "matrix"
            | "pmatrix"
            | "bmatrix"
            | "bmatrix*"
            | "Bmatrix"
            | "Bmatrix*"
            | "vmatrix"
            | "vmatrix*"
            | "Vmatrix"
            | "Vmatrix*"
            | "cases"
            | "array"
            | "substack"
    )
}

fn environment_marker(input: &str, start: usize) -> Option<(bool, &str, usize)> {
    let tail = &input[start..];
    let (opening, prefix_len) = if tail.starts_with("\\begin{") {
        (true, "\\begin{".len())
    } else if tail.starts_with("\\end{") {
        (false, "\\end{".len())
    } else {
        return None;
    };
    let name_start = start + prefix_len;
    let relative_end = input[name_start..].find('}')?;
    let name_end = name_start + relative_end;
    Some((opening, &input[name_start..name_end], name_end + 1))
}

fn take_array_alignment(content: &str) -> (Option<String>, &str, usize) {
    let leading = content.len().saturating_sub(content.trim_start().len());
    let trimmed = &content[leading..];
    if !trimmed.starts_with('{') {
        return (None, content, 0);
    }
    let mut depth = 0usize;
    for (offset, ch) in trimmed.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = offset + ch.len_utf8();
                    return (
                        Some(trimmed[1..offset].to_string()),
                        &trimmed[end..],
                        leading + end,
                    );
                }
            }
            _ => {}
        }
    }
    (None, content, 0)
}

fn split_environment(content: &str) -> Vec<Vec<std::ops::Range<usize>>> {
    let mut rows = vec![Vec::new()];
    let mut cell_start = 0usize;
    let mut cursor = 0usize;
    let mut brace_depth = 0usize;
    let mut environment_depth = 0usize;

    while cursor < content.len() {
        if content[cursor..].starts_with('%') {
            skip_comment(content, &mut cursor);
            continue;
        }
        if content[cursor..].starts_with("\\begin{") {
            environment_depth += 1;
            cursor = environment_marker(content, cursor)
                .map(|(_, _, end)| end)
                .unwrap_or(cursor + 1);
            continue;
        }
        if content[cursor..].starts_with("\\end{") {
            environment_depth = environment_depth.saturating_sub(1);
            cursor = environment_marker(content, cursor)
                .map(|(_, _, end)| end)
                .unwrap_or(cursor + 1);
            continue;
        }
        if brace_depth == 0 && environment_depth == 0 && content[cursor..].starts_with("\\\\") {
            rows.last_mut()
                .expect("environment has a row")
                .push(cell_start..cursor);
            rows.push(Vec::new());
            cursor += 2;
            cell_start = cursor;
            continue;
        }

        let ch = content[cursor..]
            .chars()
            .next()
            .expect("valid character boundary");
        if brace_depth == 0 && environment_depth == 0 && ch == '&' {
            rows.last_mut()
                .expect("environment has a row")
                .push(cell_start..cursor);
            cursor += ch.len_utf8();
            cell_start = cursor;
            continue;
        }
        match ch {
            '\\' => {
                cursor += ch.len_utf8();
                if let Some(escaped) = content[cursor..].chars().next() {
                    cursor += escaped.len_utf8();
                }
            }
            '{' => {
                brace_depth += 1;
                cursor += ch.len_utf8();
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                cursor += ch.len_utf8();
            }
            _ => cursor += ch.len_utf8(),
        }
    }

    rows.last_mut()
        .expect("environment has a row")
        .push(cell_start..content.len());
    rows.retain(|row| {
        row.iter()
            .any(|range| !content[range.clone()].trim().is_empty())
    });
    rows
}
