use crate::config::Config;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use serde_yaml::Mapping;
use std::mem;
use std::ops::Range;

mod raw_html;
mod source_lines;

pub(crate) const BLANK_LINE_MARKER: &str = "MDV_BLANK_LINE_MARKER";
pub(crate) use source_lines::{Marker as SourceLineMarker, from_event as source_line_from_event};

/// Markdown processor that parses markdown and prepares it for rendering
pub struct MarkdownProcessor {
    config: Config,
    options: Options,
}

pub(crate) struct ParsedDocument {
    pub(crate) events: Vec<Event<'static>>,
    pub(crate) front_matter: Option<FrontMatter>,
}

pub(crate) struct FrontMatter {
    pub(crate) raw: String,
    pub(crate) properties: Mapping,
}

mod admonitions;
mod blockquotes;
mod conversion;
mod detection;
mod events;
mod fences;
mod parsing;
mod structure;
mod task_lists;

pub use detection::{detect_source_code, extract_code_language};

#[cfg(test)]
mod tests;
