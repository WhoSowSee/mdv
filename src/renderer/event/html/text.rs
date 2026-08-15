use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_text(&mut self, text: &str, context: HtmlContext) -> Result<()> {
        let text = if context.preserve_whitespace {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            self.collapse_html_text(text)
        };

        if text.is_empty() {
            return Ok(());
        }

        let text = context
            .script
            .map(|script| convert_script(&text, script))
            .unwrap_or(text);

        if context.highlighted {
            self.note_paragraph_content();
            return self.process_segment_with_wrapping_and_formatting(
                &text,
                true,
                self.table_state.is_some(),
            );
        }

        self.handle_text(CowStr::from(text))
    }

    pub(super) fn collapse_html_text(&self, text: &str) -> String {
        let starts_with_whitespace = text
            .chars()
            .next()
            .map(char::is_whitespace)
            .unwrap_or(false);
        let ends_with_whitespace = text
            .chars()
            .next_back()
            .map(char::is_whitespace)
            .unwrap_or(false);

        let mut collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.is_empty() {
            if starts_with_whitespace && self.needs_html_separator_before_text() {
                return " ".to_string();
            }
            return String::new();
        }

        if starts_with_whitespace && self.needs_html_separator_before_text() {
            collapsed.insert(0, ' ');
        }
        if ends_with_whitespace {
            collapsed.push(' ');
        }

        collapsed
    }

    pub(super) fn needs_html_separator_before_text(&self) -> bool {
        let line = self
            .output
            .rsplit_once('\n')
            .map(|(_, line)| line)
            .unwrap_or(&self.output);
        let clean = strip_ansi(line);
        clean
            .chars()
            .next_back()
            .map(|ch| !ch.is_whitespace() && !matches!(ch, '(' | '[' | '{' | '/' | ' '))
            .unwrap_or(false)
    }

    pub(super) fn render_html_line_break(&mut self) {
        if let Some(ref mut table) = self.table_state {
            if !table.current_cell.ends_with('\n') {
                table.current_cell.push('\n');
            }
            return;
        }

        if self.output.is_empty() {
            return;
        }
        self.push_newline_with_context();
    }
}
