use super::*;

impl MarkdownProcessor {
    pub(super) fn extended_math_ranges(
        &self,
        content: &str,
    ) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
        let mut options = self.options;
        options.remove(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(content, options);
        let mut ranges = parser
            .reference_definitions()
            .iter()
            .map(|(_, definition)| definition.span.clone())
            .collect::<Vec<_>>();
        let events = parser.into_offset_iter().collect::<Vec<_>>();
        let mut code_blocks = Vec::new();
        let mut containers = Vec::new();
        let mut index = 0usize;
        while index < events.len() {
            if let Some((_, range, end_index)) =
                raw_html::coalesce_raw_text_container(content, &events, index)
            {
                ranges.push(range);
                index = end_index + 1;
            } else {
                index += 1;
            }
        }

        for (event, range) in &events {
            match event {
                Event::Start(Tag::CodeBlock(_)) | Event::Start(Tag::HtmlBlock) => {
                    code_blocks.push(range.start);
                }
                Event::End(TagEnd::CodeBlock) | Event::End(TagEnd::HtmlBlock) => {
                    if let Some(start) = code_blocks.pop() {
                        ranges.push(start..range.end);
                    }
                }
                Event::Code(_) | Event::Html(_) | Event::InlineHtml(_) => {
                    ranges.push(range.clone())
                }
                Event::InlineMath(_) | Event::DisplayMath(_) => ranges.push(range.clone()),
                Event::Start(Tag::Paragraph) => containers.push(range.clone()),
                Event::Start(Tag::Heading { .. }) => {
                    containers.push(range.clone());
                    if let Some(attributes) = heading_attribute_range(content, range.clone()) {
                        ranges.push(attributes);
                    }
                }
                Event::Start(Tag::Link { link_type, .. })
                | Event::Start(Tag::Image { link_type, .. }) => {
                    if matches!(
                        link_type,
                        pulldown_cmark::LinkType::Inline
                            | pulldown_cmark::LinkType::Autolink
                            | pulldown_cmark::LinkType::Email
                    ) && let Some(destination) = link_destination_range(content, range.clone())
                    {
                        ranges.push(destination);
                    }
                }
                _ => {}
            }
        }

        (merge_ranges(ranges), containers)
    }
}

fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && range.start <= previous.end
        {
            previous.end = previous.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    merged
}

fn heading_attribute_range(content: &str, range: Range<usize>) -> Option<Range<usize>> {
    let source = content.get(range.clone())?;
    let closing = source.rfind('}')?;
    let mut depth = 0usize;
    for (index, character) in source[..=closing].char_indices().rev() {
        match character {
            '}' => depth += 1,
            '{' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let attributes = &source[index + 1..closing];
                    let looks_like_attributes = attributes
                        .split_whitespace()
                        .any(|token| token.starts_with(['#', '.']) || token.contains('='));
                    return looks_like_attributes
                        .then_some(range.start + index..range.start + closing + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn link_destination_range(content: &str, range: Range<usize>) -> Option<Range<usize>> {
    let source = content.get(range.clone())?;
    if source.starts_with('<') && source.ends_with('>') {
        return Some(range);
    }
    let destination_start = source.rfind("](")? + 2;
    let mut depth = 1usize;
    let mut cursor = destination_start;
    while cursor < source.len() {
        let character = source[cursor..].chars().next()?;
        if character == '\\' {
            cursor += character.len_utf8();
            if let Some(escaped) = source[cursor..].chars().next() {
                cursor += escaped.len_utf8();
            }
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(range.start + destination_start..range.start + cursor);
                }
            }
            _ => {}
        }
        cursor += character.len_utf8();
    }
    None
}
