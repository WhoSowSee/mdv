use crate::terminal::ansi256_to_rgb;
use crate::theme::{Color, Theme};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::str::FromStr;
use std::sync::LazyLock;
use syntect::highlighting::ScopeSelectors;
use syntect::highlighting::{
    Color as SyntectColor, FontStyle, Style, StyleModifier, Theme as SyntectTheme, ThemeItem,
    ThemeSet,
};

/// Global cache of themes
static DEFAULT_THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

pub(crate) fn default_theme_set() -> &'static ThemeSet {
    &DEFAULT_THEME_SET
}

/// Syntect theme plus a reverse RGB→Color map so the escaper restores palette
/// codes instead of truecolor. External `.tmTheme` themes use an empty map.
pub(crate) struct CodeHighlightTheme {
    pub syntect: SyntectTheme,
    palette: HashMap<(u8, u8, u8), Color>,
}

impl CodeHighlightTheme {
    pub(crate) fn syntect_only(theme: SyntectTheme) -> Self {
        Self {
            syntect: theme,
            palette: HashMap::new(),
        }
    }

    pub(crate) fn palette(&self) -> &HashMap<(u8, u8, u8), Color> {
        &self.palette
    }
}

mod builder;
mod terminal;

pub(crate) use builder::build_syntect_theme;
pub(crate) use terminal::as_terminal_escaped;

#[cfg(test)]
use builder::transparent_for_reset;
#[cfg(test)]
mod tests;
