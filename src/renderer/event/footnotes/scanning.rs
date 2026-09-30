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
        self.footnotes.text_state = FootnoteTextState::Idle;
        self.footnotes.text_buffer.clear();
    }

    pub(in crate::renderer::event) fn scan_footnotes_in_text_stream(&mut self, text: &str) {
        for ch in text.chars() {
            match self.footnotes.text_state {
                FootnoteTextState::Idle => {
                    if ch == '[' {
                        self.footnotes.text_state = FootnoteTextState::SawOpenBracket;
                    }
                }
                FootnoteTextState::SawOpenBracket => {
                    if ch == '^' {
                        self.footnotes.text_buffer.clear();
                        self.footnotes.text_state = FootnoteTextState::Collecting;
                    } else if ch != '[' {
                        self.footnotes.text_state = FootnoteTextState::Idle;
                    }
                }
                FootnoteTextState::Collecting => {
                    if ch == ']' {
                        if !self.footnotes.text_buffer.is_empty() {
                            let name = self.footnotes.text_buffer.clone();
                            self.register_footnote_reference(&name);
                        }
                        self.footnotes.text_buffer.clear();
                        self.footnotes.text_state = FootnoteTextState::Idle;
                    } else if self.footnotes.text_buffer.is_empty() && ch.is_whitespace() {
                        self.footnotes.text_buffer.clear();
                        self.footnotes.text_state = FootnoteTextState::Idle;
                    } else {
                        self.footnotes.text_buffer.push(ch);
                        if self.footnotes.text_buffer.len() > FOOTNOTE_NAME_MAX_LEN {
                            self.footnotes.text_buffer.clear();
                            self.footnotes.text_state = FootnoteTextState::Idle;
                        }
                    }
                }
            }
        }
    }

    pub(super) fn ensure_placeholder_footnotes_in_order(&mut self) {
        if self
            .footnotes
            .definitions
            .iter()
            .all(|definition| definition.kind == FootnoteDefinitionKind::Normal)
        {
            return;
        }
        let mut known_names: HashSet<String> = self.footnotes.order.iter().cloned().collect();
        for definition in self.footnotes.definitions.iter() {
            if matches!(definition.kind, FootnoteDefinitionKind::Normal) {
                continue;
            }
            if !known_names.insert(definition.name.clone()) {
                continue;
            }
            self.footnotes.order.push(definition.name.clone());
        }
    }
}
