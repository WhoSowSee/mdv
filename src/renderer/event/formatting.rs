use super::core::{CalloutFold, CalloutKind, CalloutState};
use super::{EventRenderer, PRETTY_ACCENT_COLOR, ThemeElement, create_style};
use crate::block_spacing::BlockElement;
use crate::inline_style::{InlineStyle, InlineStyleKind};
use crate::terminal::AnsiStyle;
use crate::utils::{WrapMode, display_width, strip_ansi, wrap_text_with_mode};
use crossterm::style::Color as CrosstermColor;

fn is_quote_prefix_char(ch: char) -> bool {
    matches!(ch, '│' | '┃')
}

fn strip_layout_metadata(line: &str) -> String {
    let clean = strip_ansi(line);
    crate::renderer::line_numbers::strip_internal_markers(&clean).0
}

const DEFAULT_UNKNOWN_CALLOUT_ICON: &str = "";

mod blockquotes;
mod borders;
mod callout_frame;
mod callout_label;
mod callout_render;
mod inline;
mod spacing;
