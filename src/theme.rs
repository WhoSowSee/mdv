use crate::error::MdvError;
use crate::inline_style::{InlineStyleKind, InlineStyleSet};
use crate::terminal::{AnsiStyle, ansi256_to_rgb, calculate_luminosity};
use crate::user_themes::parse_embedded_theme;
use anyhow::{Context, Result, anyhow, bail};
use crossterm::style::Color as CrosstermColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

mod builtin;
mod color_parse;
mod colors;
mod display;
mod element;
mod manager;
mod overrides;
mod types;

pub use colors::Color;
pub use display::{create_style, list_themes};
pub use element::ThemeElement;
pub use manager::ThemeManager;
pub use overrides::{apply_custom_code_theme, apply_custom_theme};
pub use types::{SyntaxTheme, Theme};

pub(crate) use color_parse::parse_color_value;

#[cfg(test)]
use builtin::BUILTIN_THEME_FILES;
use builtin::BUILTIN_THEMES;
use color_parse::{calculate_theme_luminosity, is_none_value, parse_color_spec};

#[cfg(test)]
mod tests;
