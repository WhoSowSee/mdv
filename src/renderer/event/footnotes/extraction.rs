use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn extract_footnote_definitions(
        &self,
        events: Vec<Event<'static>>,
    ) -> (Vec<Event<'static>>, Vec<FootnoteDefinition>) {
        let mut cleaned = Vec::new();
        let mut definitions = Vec::new();
        let mut current: Option<FootnoteDefinition> = None;

        for event in events {
            match event {
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    if let Some(def) = current.take() {
                        definitions.push(def);
                    }
                    current = Some(FootnoteDefinition {
                        name: name.to_string(),
                        events: Vec::new(),
                        kind: FootnoteDefinitionKind::Normal,
                    });
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    if let Some(def) = current.take() {
                        definitions.push(def);
                    }
                }
                other => {
                    if let Some(def) = current.as_mut() {
                        def.events.push(other);
                    } else {
                        cleaned.push(other);
                    }
                }
            }
        }

        if let Some(def) = current {
            definitions.push(def);
        }

        let (cleaned, definitions) =
            self.extract_placeholder_footnote_definitions(cleaned, definitions);

        (cleaned, definitions)
    }

    pub(in crate::renderer::event) fn register_footnote_reference(&mut self, name: &str) {
        if self.suppress_footnote_output {
            return;
        }

        self.footnote_order.push(name.to_string());

        if matches!(self.config.footnote_style, FootnoteStyle::Attached) {
            self.current_inline_footnotes.push(name.to_string());
        }
    }

    pub(in crate::renderer::event) fn should_highlight_footnote_reference(
        &self,
        name: &str,
    ) -> bool {
        if self.has_footnote_definition(name) {
            return true;
        }

        matches!(
            self.config.missing_footnote_style,
            MissingFootnoteStyle::Hide
        )
    }

    pub(in crate::renderer::event) fn has_renderable_footnotes(&self, names: &[String]) -> bool {
        names
            .iter()
            .any(|name| self.should_render_footnote_entry(name))
    }
}
