use std::borrow::Cow;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

pub(super) enum TextUnits<'a> {
    Words(std::vec::IntoIter<Cow<'a, str>>),
    Characters(Graphemes<'a>),
    Single(Option<&'a str>),
}

impl<'a> TextUnits<'a> {
    pub(super) fn characters(text: &'a str) -> Self {
        Self::Characters(text.graphemes(true))
    }
}

impl<'a> Iterator for TextUnits<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Words(words) => words.next(),
            Self::Characters(graphemes) => graphemes.next().map(Cow::Borrowed),
            Self::Single(text) => text.take().map(Cow::Borrowed),
        }
    }
}
