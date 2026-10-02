//! User-defined themes loaded from `<config_dir>/themes/*.yaml`.
use crate::inline_style::{InlineStyleOverrides, InlineStyleSet};
use crate::theme::{
    CalloutPalette, CalloutTheme, CodeBlockTheme, Color, FrontMatterTheme, InlineTheme,
    LineNumberTheme, ListTheme, MathTheme, PagerTheme, SyntaxTheme, TableTheme, Theme,
    ThemeManager, TodoTheme, parse_color_value,
};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::Path;

const THEMES_DIR: &str = "themes";
const THEME_EXT_YAML: &str = "yaml";
const THEME_EXT_YML: &str = "yml";

mod colors;
mod complete;
mod groups;
mod inline;
mod loading;
mod pager;
mod schema;

use colors::{ColorYaml, OptionalColorYaml, pick, required};
use groups::*;
use inline::{HighlightFile, InlineFile};
use pager::PagerFile;

pub use loading::load_user_themes;
pub(crate) use schema::{ThemeFile, parse_embedded_theme};

#[cfg(test)]
mod tests;
