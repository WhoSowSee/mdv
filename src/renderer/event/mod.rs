mod code;
mod core;
mod formatting;
mod headings;
mod images;
mod links;
mod misc;
mod tables;
mod text;

use crossterm::style::Color as CrosstermColor;

pub(super) use core::TableState;
pub(crate) use core::{CapturedReferenceBlock, EventRenderer};

pub(super) use crate::cli::{CodeBlockStyle, LinkStyle, LinkTruncationStyle};
pub(super) use crate::config::Config;
pub(super) use crate::error::MdvError;
pub(super) use crate::markdown::{MarkdownProcessor, detect_source_code, extract_code_language};
pub(super) use crate::table::TableRenderer;
pub(super) use crate::theme::{Theme, ThemeElement, create_style};
pub(super) use crate::utils::{WrapMode, wrap_text_with_mode};
pub(super) use anyhow::Result;
pub(super) use pulldown_cmark::{Alignment, CowStr, Event, HeadingLevel, Tag, TagEnd};
pub(super) use std::collections::HashMap;
pub(super) use syntect::easy::HighlightLines;
pub(super) use syntect::parsing::SyntaxSet;
pub(super) use syntect::util::as_24_bit_terminal_escaped;

pub(super) const PRETTY_ACCENT_COLOR: CrosstermColor = CrosstermColor::Rgb {
    r: 0x8f,
    g: 0x93,
    b: 0xa2,
};
