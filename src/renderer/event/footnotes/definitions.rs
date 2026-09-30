use super::FootnoteDefinition;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Default)]
pub(in crate::renderer::event) struct FootnoteDefinitions(Arc<Definitions>);

#[derive(Clone, Default)]
struct Definitions {
    entries: Vec<FootnoteDefinition>,
    names: HashMap<String, Vec<usize>>,
}

impl FootnoteDefinitions {
    pub(in crate::renderer::event) fn iter(&self) -> std::slice::Iter<'_, FootnoteDefinition> {
        self.0.entries.iter()
    }

    pub(in crate::renderer::event) fn contains(&self, name: &str) -> bool {
        self.0.names.contains_key(name)
    }

    pub(in crate::renderer::event) fn get(
        &self,
        name: &str,
        occurrence: usize,
    ) -> Option<&FootnoteDefinition> {
        let indices = self.0.names.get(name)?;
        let index = indices.get(occurrence).or_else(|| indices.last())?;
        self.0.entries.get(*index)
    }

    pub(in crate::renderer::event) fn extend(
        &mut self,
        definitions: impl IntoIterator<Item = FootnoteDefinition>,
    ) {
        let store = Arc::make_mut(&mut self.0);
        for definition in definitions {
            store
                .names
                .entry(definition.name.clone())
                .or_default()
                .push(store.entries.len());
            store.entries.push(definition);
        }
    }

    pub(in crate::renderer::event) fn merge(&mut self, entries: Vec<FootnoteDefinition>) {
        if entries.is_empty() {
            return;
        }
        let mut merged = Self::default();
        merged.extend(entries);
        for existing in self.iter() {
            if !merged.contains(&existing.name) {
                merged.extend([existing.clone()]);
            }
        }
        *self = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::super::FootnoteDefinitionKind;
    use super::*;

    #[test]
    fn duplicate_occurrences_keep_order_and_reuse_the_last_definition() {
        let mut definitions = FootnoteDefinitions::default();
        definitions.extend([
            FootnoteDefinition {
                name: "b".into(),
                events: vec![],
                kind: FootnoteDefinitionKind::Normal,
            },
            FootnoteDefinition {
                name: "a".into(),
                events: vec![],
                kind: FootnoteDefinitionKind::Normal,
            },
            FootnoteDefinition {
                name: "b".into(),
                events: vec![],
                kind: FootnoteDefinitionKind::EmptyBody,
            },
        ]);
        assert_eq!(
            definitions.get("b", 0).unwrap().kind,
            FootnoteDefinitionKind::Normal
        );
        assert_eq!(
            definitions.get("b", 1).unwrap().kind,
            FootnoteDefinitionKind::EmptyBody
        );
        assert_eq!(
            definitions.get("b", 9).unwrap().kind,
            FootnoteDefinitionKind::EmptyBody
        );
        assert!(definitions.get("missing", 0).is_none());
        assert_eq!(
            definitions
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            ["b", "a", "b"]
        );
        let shared = definitions.clone();
        definitions.merge(vec![]);
        assert!(Arc::ptr_eq(&shared.0, &definitions.0));
    }
}
