use super::core::FootnoteTextState;
use super::{
    CapturedReferenceBlock, Event, EventRenderer, FootnoteStyle, MissingFootnoteStyle,
    PRETTY_ACCENT_COLOR, Result, Tag, TagEnd, ThemeElement, create_style, wrap_text_with_mode,
};
use crate::block_spacing::BlockElement;
use crate::terminal::AnsiStyle;
use regex::regex;
use std::collections::HashSet;

const MISSING_FOOTNOTE_PLACEHOLDER: &str = "Missing footnote definition";
const INVALID_FOOTNOTE_SYNTAX_MESSAGE: &str = "Invalid footnote syntax";
const EMPTY_FOOTNOTE_CONTENT_MESSAGE: &str = "Empty footnote content";
const FOOTNOTE_NAME_MAX_LEN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FootnoteDefinitionKind {
    Normal,
    EmptyBody,
    InvalidSyntax,
}

#[derive(Debug, Clone)]
pub(crate) struct FootnoteDefinition {
    pub name: String,
    pub events: Vec<Event<'static>>,
    pub kind: FootnoteDefinitionKind,
}

mod extraction;
mod markdown;
mod rendering;
mod scanning;
