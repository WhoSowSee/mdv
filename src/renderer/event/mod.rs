mod code;
mod core;
mod definition_lists;
mod footnotes;
mod formatting;
mod headings;
mod html;
mod images;
mod links;
mod math;
mod misc;
mod soft_breaks;
mod spacing;
mod tables;
mod text;

pub(super) use core::EventRenderer;
pub(crate) use core::{CapturedReferenceBlock, DeferredLinkReferenceBlock};
pub(super) use core::{HtmlBlockBuffer, TableInlineUrlSegment, TableInlineUrlTarget, TableState};
use definition_lists::DefinitionListState;
use footnotes::FootnoteDefinitions;

pub(super) use crate::cli::{
    CalloutStyle, CodeBlockStyle, CodeWrapIndent, FootnoteStyle, LinkStyle, LinkTruncationStyle,
    MathBlockStyle, MissingFootnoteStyle,
};
pub(super) use crate::config::Config;
pub(super) use crate::error::MdvError;
pub(super) use crate::markdown::{MarkdownProcessor, detect_source_code, extract_code_language};
pub(super) use crate::renderer::syntax_theme::as_terminal_escaped;
pub(super) use crate::table::TableRenderer;
pub(super) use crate::theme::{Theme, ThemeElement, create_style};
pub(super) use crate::utils::{WrapMode, wrap_text_with_mode};
pub(super) use anyhow::Result;
pub(super) use pulldown_cmark::{Alignment, CowStr, Event, HeadingLevel, Tag, TagEnd};
pub(super) use std::collections::HashMap;
pub(super) use syntect::easy::HighlightLines;
pub(super) use syntect::parsing::SyntaxSet;
