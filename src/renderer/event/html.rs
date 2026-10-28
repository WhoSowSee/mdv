use super::core::CalloutState;
use super::images::{media_marker, media_marker_leading_separator};
use super::{
    Alignment, CowStr, EventRenderer, HeadingLevel, HtmlBlockBuffer, HtmlBlockKind, LinkStyle,
    Result, TableState, ThemeElement, create_style,
};
use crate::math::ScriptKind;
use crate::utils::{display_width, display_width_ansi, escape_html_text, strip_ansi};
use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node as HtmlNode};

mod forms;
mod table_cells;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HtmlAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct HtmlContext {
    depth: usize,
    alignment: HtmlAlignment,
    preserve_whitespace: bool,
    highlighted: bool,
    script: Option<ScriptKind>,
    list_depth: usize,
}

#[derive(Clone, Copy, Debug)]
enum HtmlOrderedListMarkerKind {
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Clone, Copy, Debug)]
enum HtmlListMarkerState {
    Ordered {
        next: Option<i64>,
        step: i64,
        kind: HtmlOrderedListMarkerKind,
    },
    Unordered {
        marker: &'static str,
    },
}

impl HtmlListMarkerState {
    fn next_marker(&mut self, item: ElementRef<'_>) -> Result<String> {
        match self {
            Self::Ordered { next, step, kind } => {
                let value = parse_html_integer_attr(&item, "value")
                    .or(*next)
                    .ok_or_else(|| anyhow::anyhow!("HTML ordered list counter overflow"))?;
                let item_kind = html_ordered_list_marker_kind(&item).unwrap_or(*kind);
                *next = value.checked_add(*step);
                Ok(format!(
                    "{}. ",
                    format_html_ordered_marker(value, item_kind)
                ))
            }
            Self::Unordered { marker } => Ok((*marker).to_string()),
        }
    }
}

impl Default for HtmlContext {
    fn default() -> Self {
        Self {
            depth: 0,
            alignment: HtmlAlignment::Left,
            preserve_whitespace: false,
            highlighted: false,
            script: None,
            list_depth: 0,
        }
    }
}

impl HtmlContext {
    fn with_alignment(self, alignment: HtmlAlignment) -> Self {
        Self { alignment, ..self }
    }

    fn with_preserve_whitespace(self) -> Self {
        Self {
            preserve_whitespace: true,
            ..self
        }
    }

    fn with_highlighted(self) -> Self {
        Self {
            highlighted: true,
            ..self
        }
    }

    fn with_script(self, script: ScriptKind) -> Self {
        Self {
            script: Some(script),
            ..self
        }
    }

    fn in_nested_list(self) -> Self {
        Self {
            list_depth: self.list_depth + 1,
            ..self
        }
    }

    fn without_list_depth(self) -> Self {
        Self {
            list_depth: 0,
            ..self
        }
    }
}

mod blockquotes;
mod blocks;
mod buffer;
mod buffer_helpers;
mod definitions;
mod details;
mod dispatch;
mod layout;
mod list_helpers;
mod lists;
mod media;
mod spacing;
mod styles;
mod table_helpers;
mod tables;
mod text;

use buffer_helpers::*;
use details::html_details_balance;
use list_helpers::*;
use styles::*;
use table_helpers::*;
