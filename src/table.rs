use std::borrow::Cow;

use crate::theme::{Color as ThemeColor, Theme, ThemeElement, create_style};
use crate::utils::{display_width, strip_ansi};
use anyhow::Result;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement, ContentLineStyle,
    LineStyle, Table, TableStyle, Width, presets::UTF8_FULL,
};
use pulldown_cmark::Alignment;

use crate::cli::{TableWrapMode, TextWrapMode};

pub(crate) const TABLE_REFERENCE_WRAP_MARKER: char = '\u{200B}';
const TABLE_GRAPHEME_WRAP_DELIMITER: char = '\0';
const COMPACT_TABLE_STYLE: TableStyle = TableStyle::new()
    .header_lines(ContentLineStyle::none().junction('│'))
    .header_separator(LineStyle::none().fill('─').junction('┼'))
    .content_lines(ContentLineStyle::none().junction('│'));
type TableBlock = (Vec<String>, Vec<Vec<String>>, Vec<Alignment>);

enum ReferenceLayout {
    Natural,
    Constrained(Vec<usize>),
    ForcedBreak,
}

mod layout;
mod links;
mod rendering;
mod whitespace;

pub use links::apply_clickable_link_replacements;

pub struct TableRenderer {
    theme: Theme,
    no_colors: bool,
    terminal_width: usize,
    table_wrap: TableWrapMode,
    text_wrap: TextWrapMode,
    pretty_table: bool,
}

#[cfg(test)]
use links::styled_wrapper;
#[cfg(test)]
use rendering::theme_color_to_comfy;

#[cfg(test)]
mod tests;
