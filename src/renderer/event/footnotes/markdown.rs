use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn extract_markdown_code_footnote_definitions(
        &self,
        code: &str,
    ) -> (String, Vec<FootnoteDefinition>) {
        let mut definitions = Vec::new();
        let mut known_names: HashSet<String> = self
            .footnote_definitions
            .iter()
            .map(|definition| definition.name.clone())
            .collect();
        let mut cleaned_lines = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            if let Some((name, kind)) = Self::parse_placeholder_footnote_line(trimmed) {
                if known_names.contains(&name) {
                    continue;
                }
                definitions.push(FootnoteDefinition {
                    name: name.clone(),
                    events: Vec::new(),
                    kind,
                });
                known_names.insert(name);
                continue;
            }
            cleaned_lines.push(line);
        }

        let mut cleaned = cleaned_lines.join("\n");
        if code.ends_with('\n') {
            cleaned.push('\n');
        }

        (cleaned, definitions)
    }

    pub(super) fn extract_placeholder_footnote_definitions(
        &self,
        events: Vec<Event<'static>>,
        mut definitions: Vec<FootnoteDefinition>,
    ) -> (Vec<Event<'static>>, Vec<FootnoteDefinition>) {
        let mut cleaned = Vec::with_capacity(events.len());
        let mut known_names: HashSet<String> = definitions
            .iter()
            .map(|definition| definition.name.clone())
            .collect();

        let mut idx = 0usize;
        while idx < events.len() {
            if matches!(events[idx], Event::Start(Tag::Paragraph))
                && let Some((end_idx, placeholders)) =
                    Self::extract_bare_footnote_paragraph(&events, idx)
            {
                for (name, kind) in placeholders {
                    if known_names.contains(&name) {
                        continue;
                    }
                    definitions.push(FootnoteDefinition {
                        name: name.clone(),
                        events: Vec::new(),
                        kind,
                    });
                    known_names.insert(name);
                }
                idx = end_idx + 1;
                continue;
            }

            cleaned.push(events[idx].clone());
            idx += 1;
        }

        (cleaned, definitions)
    }

    pub(super) fn extract_bare_footnote_paragraph(
        events: &[Event<'static>],
        start_idx: usize,
    ) -> Option<(usize, Vec<(String, FootnoteDefinitionKind)>)> {
        let mut end_idx = start_idx + 1;
        while end_idx < events.len() {
            if matches!(events[end_idx], Event::End(TagEnd::Paragraph)) {
                break;
            }
            end_idx += 1;
        }

        if end_idx >= events.len() {
            return None;
        }

        let mut buffer = String::new();
        for event in &events[start_idx + 1..end_idx] {
            match event {
                Event::FootnoteReference(name) => {
                    buffer.push_str(&format!("[^{}]", name));
                }
                Event::Text(text) => {
                    buffer.push_str(text);
                }
                Event::SoftBreak | Event::HardBreak => {
                    buffer.push('\n');
                }
                _ => {
                    return None;
                }
            }
        }

        let candidate = buffer.trim();
        if candidate.is_empty() {
            return None;
        }

        let mut placeholders = Vec::new();
        for line in candidate.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let (name, kind) = Self::parse_placeholder_footnote_line(line)?;
            placeholders.push((name, kind));
        }

        if placeholders.is_empty() {
            return None;
        }

        Some((end_idx, placeholders))
    }

    pub(super) fn parse_placeholder_footnote_line(
        line: &str,
    ) -> Option<(String, FootnoteDefinitionKind)> {
        let bare_regex = regex!(r"^\[\^([^\]\s][^\]]*)\]\s*$");
        let empty_regex = regex!(r"^\[\^([^\]\s][^\]]*)\]:\s*$");

        if let Some(caps) = empty_regex.captures(line) {
            return caps
                .get(1)
                .map(|name| (name.as_str().to_string(), FootnoteDefinitionKind::EmptyBody));
        }

        if let Some(caps) = bare_regex.captures(line) {
            return caps.get(1).map(|name| {
                (
                    name.as_str().to_string(),
                    FootnoteDefinitionKind::InvalidSyntax,
                )
            });
        }

        None
    }
}
