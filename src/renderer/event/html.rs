use super::core::CalloutState;
use super::images::{media_marker, media_marker_leading_separator};
use super::{
    Alignment, CowStr, EventRenderer, HeadingLevel, HtmlBlockBuffer, LinkStyle, Result, TableState,
    ThemeElement, create_style,
};
use crate::math::{ScriptKind, convert_script};
use crate::utils::{display_width, escape_html_text, strip_ansi};
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
        current: i64,
        step: i64,
        kind: HtmlOrderedListMarkerKind,
    },
    Unordered {
        marker: &'static str,
    },
}

impl HtmlListMarkerState {
    fn next_marker(&mut self, item: ElementRef<'_>) -> String {
        match self {
            Self::Ordered {
                current,
                step,
                kind,
            } => {
                if let Some(value) = parse_html_integer_attr(&item, "value") {
                    *current = value;
                }
                let item_kind = html_ordered_list_marker_kind(&item).unwrap_or(*kind);
                let marker = format!("{}. ", format_html_ordered_marker(*current, item_kind));
                *current += *step;
                marker
            }
            Self::Unordered { marker } => (*marker).to_string(),
        }
    }
}

impl Default for HtmlContext {
    fn default() -> Self {
        Self {
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
mod dispatch;
mod layout;
mod list_helpers;
mod lists;
mod media;
mod media_helpers;
mod styles;
mod table_helpers;
mod tables;
mod text;
mod text_helpers;

use buffer_helpers::*;
use list_helpers::*;
use media_helpers::*;
use styles::*;
use table_helpers::*;
use text_helpers::*;
