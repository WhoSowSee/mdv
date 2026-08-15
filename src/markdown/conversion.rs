use super::*;

impl MarkdownProcessor {
    pub(super) fn reverse_events(&self, events: Vec<Event<'static>>) -> Vec<Event<'static>> {
        if events.is_empty() {
            return events;
        }

        let mut segments: Vec<Vec<Event<'static>>> = Vec::new();
        let mut current: Vec<Event<'static>> = Vec::new();
        let mut depth = 0usize;

        let mut pending_source_markers = Vec::new();
        for event in events {
            if source_line_from_event(&event).is_some() {
                pending_source_markers.push(event);
                continue;
            }

            match event {
                Event::Start(_) => {
                    if depth == 0 && !current.is_empty() {
                        segments.push(mem::take(&mut current));
                    }
                    current.append(&mut pending_source_markers);
                    depth += 1;
                    current.push(event);
                }
                Event::End(_) => {
                    current.append(&mut pending_source_markers);
                    current.push(event);
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        segments.push(mem::take(&mut current));
                    }
                }
                _ => {
                    current.append(&mut pending_source_markers);
                    current.push(event);
                    if depth == 0 {
                        segments.push(mem::take(&mut current));
                    }
                }
            }
        }

        current.append(&mut pending_source_markers);
        if !current.is_empty() {
            segments.push(current);
        }

        segments.reverse();
        segments.into_iter().flatten().collect()
    }

    pub(super) fn convert_to_static(&self, event: Event) -> Event<'static> {
        match event {
            Event::Start(tag) => Event::Start(self.convert_tag_to_static(tag)),
            Event::End(tag_end) => Event::End(tag_end),
            Event::Text(text) => Event::Text(text.to_string().into()),
            Event::Code(code) => Event::Code(code.to_string().into()),
            Event::Html(html) => Event::Html(html.to_string().into()),
            Event::InlineHtml(html) => Event::InlineHtml(html.to_string().into()),
            Event::FootnoteReference(name) => Event::FootnoteReference(name.to_string().into()),
            Event::SoftBreak => Event::SoftBreak,
            Event::HardBreak => Event::HardBreak,
            Event::Rule => Event::Rule,
            Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
            Event::InlineMath(math) => Event::InlineMath(math.to_string().into()),
            Event::DisplayMath(math) => Event::DisplayMath(math.to_string().into()),
        }
    }

    pub(super) fn convert_tag_to_static(&self, tag: Tag) -> Tag<'static> {
        match tag {
            Tag::Paragraph => Tag::Paragraph,
            Tag::Heading {
                level,
                id,
                classes,
                attrs,
            } => Tag::Heading {
                level,
                id: id.map(|s| s.to_string().into()),
                classes: classes.into_iter().map(|s| s.to_string().into()).collect(),
                attrs: attrs
                    .into_iter()
                    .map(|(k, v)| (k.to_string().into(), v.map(|s| s.to_string().into())))
                    .collect(),
            },
            Tag::BlockQuote(kind) => Tag::BlockQuote(kind),
            Tag::CodeBlock(kind) => {
                let static_kind = match kind {
                    CodeBlockKind::Indented => CodeBlockKind::Indented,
                    CodeBlockKind::Fenced(lang) => CodeBlockKind::Fenced(lang.to_string().into()),
                };
                Tag::CodeBlock(static_kind)
            }
            Tag::List(start) => Tag::List(start),
            Tag::Item => Tag::Item,
            Tag::FootnoteDefinition(name) => Tag::FootnoteDefinition(name.to_string().into()),
            Tag::Table(alignments) => Tag::Table(alignments),
            Tag::TableHead => Tag::TableHead,
            Tag::TableRow => Tag::TableRow,
            Tag::TableCell => Tag::TableCell,
            Tag::Emphasis => Tag::Emphasis,
            Tag::Strong => Tag::Strong,
            Tag::Strikethrough => Tag::Strikethrough,
            Tag::Superscript => Tag::Superscript,
            Tag::Subscript => Tag::Subscript,
            Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Link {
                link_type,
                dest_url: dest_url.to_string().into(),
                title: title.to_string().into(),
                id: id.to_string().into(),
            },
            Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Image {
                link_type,
                dest_url: dest_url.to_string().into(),
                title: title.to_string().into(),
                id: id.to_string().into(),
            },
            Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
            Tag::HtmlBlock => Tag::HtmlBlock,
            Tag::DefinitionList => Tag::DefinitionList,
            Tag::DefinitionListTitle => Tag::DefinitionListTitle,
            Tag::DefinitionListDefinition => Tag::DefinitionListDefinition,
        }
    }

    pub(super) fn process_text<'a>(&self, text: &CowStr<'a>) -> CowStr<'a> {
        if text.as_ref().contains('\t') {
            self.expand_tabs(text.as_ref()).into()
        } else {
            text.clone()
        }
    }

    pub(super) fn expand_tabs(&self, text: &str) -> String {
        text.replace('\t', &" ".repeat(self.config.tab_length))
    }
}
