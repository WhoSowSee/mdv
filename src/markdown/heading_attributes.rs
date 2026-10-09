use super::*;

impl MarkdownProcessor {
    pub(super) fn normalize_heading_attributes(&self, content: String) -> String {
        if !content.contains('}') {
            return content;
        }

        let mut pending = None;
        let mut literal_closing_braces = Vec::new();
        for (event, range) in Parser::new_ext(&content, self.options).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading {
                    id, classes, attrs, ..
                }) => {
                    let source = &content[range.clone()];
                    pending = if id.is_none()
                        && classes.is_empty()
                        && attrs.iter().all(|(_, value)| value.is_none())
                        && let Some(closing) = source.rfind('}')
                        && let Some(opening) = source[..closing].rfind('{')
                    {
                        Some((
                            range.start + opening..range.start + closing + 1,
                            range.start,
                        ))
                    } else {
                        None
                    };
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some((braces, visible_end)) = pending.take()
                        && visible_end <= braces.start
                    {
                        literal_closing_braces.push(braces.end - 1);
                    }
                }
                _ => {
                    if let Some((_, visible_end)) = pending.as_mut() {
                        *visible_end = (*visible_end).max(range.end);
                    }
                }
            }
        }

        if literal_closing_braces.is_empty() {
            return content;
        }

        let mut normalized = String::with_capacity(content.len() + literal_closing_braces.len());
        let mut cursor = 0;
        for brace in literal_closing_braces {
            normalized.push_str(&content[cursor..brace]);
            // pulldown-cmark still strips attribute blocks with escaped opening braces.
            normalized.push('\\');
            cursor = brace;
        }
        normalized.push_str(&content[cursor..]);
        normalized
    }
}
