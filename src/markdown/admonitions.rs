use super::MarkdownProcessor;
use pulldown_cmark::Event;

mod attributes;
mod boundaries;
mod containers;
mod metadata;
mod protected;
mod scanner;
mod syntax;

use metadata::Metadata;
pub(crate) use metadata::{CalloutOptions, options_from_event};
use syntax::{Dialect, Opening, Terminator};

#[derive(Clone)]
struct Line {
    text: String,
    source: Option<usize>,
}

impl Line {
    fn with_text(&self, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            source: self.source,
        }
    }

    fn blank() -> Self {
        Self {
            text: String::new(),
            source: None,
        }
    }
}

pub(super) struct Admonitions {
    prefix: String,
    metadata: Vec<CalloutOptions>,
}

impl Admonitions {
    pub(super) fn new(content: &str) -> Self {
        let mut prefix = "MDVCALLOUTMETA".to_string();
        while content.contains(&prefix) {
            prefix.push('X');
        }
        Self {
            prefix,
            metadata: Vec::new(),
        }
    }

    fn marker(&mut self, metadata: &Metadata) -> String {
        let index = self.metadata.len();
        self.metadata.push(metadata.options);
        let fold = metadata.fold.map_or(String::new(), |fold| fold.to_string());
        let title = metadata.title.as_deref().unwrap_or_default();
        format!(
            "{}{}END [!{}]{} {}",
            self.prefix, index, metadata.kind, fold, title
        )
    }

    pub(super) fn restore_events(&self, events: &mut Vec<(Event<'_>, std::ops::Range<usize>)>) {
        let mut restored = Vec::with_capacity(events.len());
        for (event, range) in events.drain(..) {
            if let Event::Text(text) = &event
                && let Some(rest) = text.strip_prefix(&self.prefix)
                && let Some((index, rest)) = rest.split_once("END ")
                && let Some(options) = index
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| self.metadata.get(i))
            {
                restored.push((metadata::options_event(*options), range.clone()));
                if !rest.is_empty() {
                    restored.push((Event::Text(rest.to_string().into()), range));
                }
            } else {
                restored.push((event, range));
            }
        }
        *events = restored;
    }
}

impl MarkdownProcessor {
    pub(super) fn convert_admonitions_to_callouts(
        &self,
        content: &str,
        source_map: Option<&mut Vec<Option<usize>>>,
        admonitions: &mut Admonitions,
    ) -> String {
        let lines = content
            .lines()
            .enumerate()
            .map(|(index, text)| Line {
                text: text.trim_end_matches('\r').to_string(),
                source: source_map
                    .as_ref()
                    .and_then(|map| map.get(index).copied().flatten()),
            })
            .collect::<Vec<_>>();
        let converted = scanner::convert(&lines, admonitions);
        if let Some(map) = source_map {
            *map = converted.iter().map(|line| line.source).collect();
        }
        converted
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests;
