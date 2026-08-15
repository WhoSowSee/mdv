use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn register_footnotes_in_text(&mut self, text: &str) {
        let regex = regex!(r"\[\^([^\]\s][^\]]*)\]");

        for capture in regex.captures_iter(text) {
            if let Some(name) = capture.get(1) {
                self.register_footnote_reference(name.as_str());
            }
        }
    }

    pub(in crate::renderer::event) fn reset_footnote_text_scan(&mut self) {
        self.footnote_text_state = FootnoteTextState::Idle;
        self.footnote_text_buffer.clear();
    }

    pub(in crate::renderer::event) fn scan_footnotes_in_text_stream(&mut self, text: &str) {
        for ch in text.chars() {
            match self.footnote_text_state {
                FootnoteTextState::Idle => {
                    if ch == '[' {
                        self.footnote_text_state = FootnoteTextState::SawOpenBracket;
                    }
                }
                FootnoteTextState::SawOpenBracket => {
                    if ch == '^' {
                        self.footnote_text_buffer.clear();
                        self.footnote_text_state = FootnoteTextState::Collecting;
                    } else if ch != '[' {
                        self.footnote_text_state = FootnoteTextState::Idle;
                    }
                }
                FootnoteTextState::Collecting => {
                    if ch == ']' {
                        if !self.footnote_text_buffer.is_empty() {
                            let name = self.footnote_text_buffer.clone();
                            self.register_footnote_reference(&name);
                        }
                        self.footnote_text_buffer.clear();
                        self.footnote_text_state = FootnoteTextState::Idle;
                    } else if self.footnote_text_buffer.is_empty() && ch.is_whitespace() {
                        self.footnote_text_buffer.clear();
                        self.footnote_text_state = FootnoteTextState::Idle;
                    } else {
                        self.footnote_text_buffer.push(ch);
                        if self.footnote_text_buffer.len() > FOOTNOTE_NAME_MAX_LEN {
                            self.footnote_text_buffer.clear();
                            self.footnote_text_state = FootnoteTextState::Idle;
                        }
                    }
                }
            }
        }
    }

    pub(super) fn ensure_placeholder_footnotes_in_order(&mut self) {
        for definition in &self.footnote_definitions {
            if matches!(definition.kind, FootnoteDefinitionKind::Normal) {
                continue;
            }
            if self
                .footnote_order
                .iter()
                .any(|name| name == &definition.name)
            {
                continue;
            }
            self.footnote_order.push(definition.name.clone());
        }
    }
}
