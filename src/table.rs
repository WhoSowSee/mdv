use crate::theme::{Color as ThemeColor, Theme, ThemeElement, create_style};
use crate::utils::{display_width, strip_ansi};
use anyhow::Result;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, ContentLineStyle, LineStyle, Table,
    TableStyle, presets::UTF8_FULL,
};
use pulldown_cmark::Alignment;

use crate::cli::TableWrapMode;

const TABLE_REFERENCE_WRAP_DELIMITER: char = '\u{200B}';
const COMPACT_TABLE_STYLE: TableStyle = TableStyle::new()
    .header_lines(ContentLineStyle::none().junction('│'))
    .header_separator(LineStyle::none().fill('─').junction('┼'))
    .content_lines(ContentLineStyle::none().junction('│'));
type TableBlock = (Vec<String>, Vec<Vec<String>>, Vec<Alignment>);

mod layout;
mod links;
mod rendering;

pub use links::apply_clickable_link_replacements;

pub struct TableRenderer {
    theme: Theme,
    no_colors: bool,
    terminal_width: usize,
    table_wrap: TableWrapMode,
    pretty_table: bool,
}

#[cfg(test)]
use links::styled_wrapper;
#[cfg(test)]
use rendering::theme_color_to_comfy;

#[cfg(test)]
mod tests;
