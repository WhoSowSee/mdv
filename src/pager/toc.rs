use crate::markdown::{SourceLineMarker, source_line_from_event};
use minus::TocEntry;
use pulldown_cmark::{Event, Tag, TagEnd};

pub(super) fn headings(events: &[Event<'_>]) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut current: Option<TocEntry> = None;
    for event in events {
        if let Some(SourceLineMarker::Content(line)) = source_line_from_event(event) {
            if let Some(entry) = &mut current
                && entry.source_line == 0
            {
                entry.source_line = line;
            }
            continue;
        }
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some(TocEntry {
                    title: String::new(),
                    source_line: 0,
                    depth: *level as u8,
                });
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(mut entry) = current.take() {
                    entry.title = entry.title.split_whitespace().collect::<Vec<_>>().join(" ");
                    if entry.source_line > 0 {
                        entries.push(entry);
                    }
                }
            }
            Event::Text(text) | Event::Code(text) | Event::InlineMath(text) => {
                if let Some(entry) = &mut current {
                    entry.title.extend(text.chars().filter(|c| !c.is_control()));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(entry) = &mut current {
                    entry.title.push(' ');
                }
            }
            _ => {}
        }
    }
    entries
}

pub(super) fn visible_levels(mut entries: Vec<TocEntry>) -> Vec<TocEntry> {
    let mut levels = entries.iter().map(|e| e.depth).collect::<Vec<_>>();
    levels.sort_unstable();
    levels.dedup();
    let Some(&top) = levels.first() else {
        return entries;
    };
    let root_index =
        usize::from(levels.len() > 1 && entries.iter().filter(|e| e.depth == top).count() == 1);
    let root = levels[root_index];
    let sub = levels.get(root_index + 1).copied();
    entries.retain(|entry| entry.depth == root || Some(entry.depth) == sub);
    for entry in &mut entries {
        entry.depth = u8::from(entry.depth != root);
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cli::{LineNumberOptions, LineNumberTarget},
        config::Config,
        markdown::MarkdownProcessor,
    };

    #[test]
    fn outline_preserves_source_labels_and_promotes_only_one_level() {
        let config = Config {
            line_numbers: Some(LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: false,
            }),
            ..Config::default()
        };
        for (source, expected) in [
            (
                "---\ntitle: ignored\n---\n# Document\n\n## **First** `code`\n\n```md\n# Not a heading\n```\n\n### [Child](https://example.com)\n\n## First\n\nSetext\n------\n",
                vec![
                    ("First code", 6, 0),
                    ("Child", 12, 1),
                    ("First", 14, 0),
                    ("Setext", 16, 0),
                ],
            ),
            (
                "# Title\n\n### Root\n\n##### Child\n\n###### Deep\n",
                vec![("Root", 3, 0), ("Child", 5, 1)],
            ),
            ("", vec![]),
        ] {
            let parsed = MarkdownProcessor::new(&config)
                .parse_document(source)
                .unwrap();
            let entries = visible_levels(headings(&parsed.events));
            let actual = entries
                .iter()
                .map(|e| (e.title.as_str(), e.source_line, e.depth))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{source}");
        }
    }
}
