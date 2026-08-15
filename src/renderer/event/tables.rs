use super::{
    EventRenderer, LinkStyle, LinkTruncationStyle, PRETTY_ACCENT_COLOR, Result,
    TableInlineUrlTarget, TableRenderer, TableState,
};
use crate::block_spacing::BlockElement;
use crate::terminal::AnsiStyle;
use crate::utils::{display_width, strip_ansi};
use pulldown_cmark::Alignment;

const TABLE_COLUMN_OVERHEAD: usize = 3;
const TABLE_BORDER_OVERHEAD: usize = 1;
const TABLE_REFERENCE_WRAP_DELIMITER: char = '\u{200B}';
pub(super) const HTML_TABLE_HORIZONTAL_RULE: &str = "\u{E000}MDV_HTML_HR\u{E001}";

mod layout;
mod rendering;
