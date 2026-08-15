use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn highlight_code(
        &self,
        code: &str,
        language_hint: Option<&str>,
    ) -> Result<String> {
        if self.config.no_colors {
            return Ok(code.to_string());
        }

        let syntax = self.resolve_syntax(language_hint, code);

        let mut highlighter = HighlightLines::new(syntax, &self.code_theme.syntect);
        let mut result = String::new();

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter
                .highlight_line(line, self.syntax_set)
                .map_err(|e| MdvError::SyntaxError(e.to_string()))?;

            let escaped = as_terminal_escaped(&ranges[..], self.code_theme.palette());
            result.push_str(&escaped);

            if !line.ends_with('\n') {
                // Maintain the trailing newline that callers expect so wrapping keeps working.
                result.push('\n');
            }
        }

        // Reset before the trailing newline; appended after it, it becomes a phantom row when callers split on '\n'.
        if !result.is_empty() {
            if result.ends_with('\n') {
                result.pop();
            }
            result.push_str("\x1b[0m\n");
        }

        Ok(result)
    }

    pub(super) fn highlight_footnote_markers_in_ansi(&self, line: &str) -> String {
        if self.config.no_colors {
            return line.to_string();
        }

        let regex = regex!(r"\[\^([^\]\s][^\]]*)\]");

        let clean = strip_ansi(line);
        if !regex.is_match(&clean) {
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

        for capture in regex.captures_iter(&clean) {
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
}
