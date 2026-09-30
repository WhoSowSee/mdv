use std::{borrow::Cow, str::CharIndices};

pub(super) enum TextUnits<'a> {
    Words(std::vec::IntoIter<Cow<'a, str>>),
    Characters {
        text: &'a str,
        indices: CharIndices<'a>,
    },
    Single(Option<&'a str>),
}

impl<'a> TextUnits<'a> {
    pub(super) fn characters(text: &'a str) -> Self {
        Self::Characters {
            text,
            indices: text.char_indices(),
        }
    }
}

impl<'a> Iterator for TextUnits<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Words(words) => words.next(),
            Self::Characters { text, indices } => indices
                .next()
                .map(|(index, ch)| Cow::Borrowed(&text[index..index + ch.len_utf8()])),
            Self::Single(text) => text.take().map(Cow::Borrowed),
        }
    }
}
