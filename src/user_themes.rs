//! User-defined themes loaded from `<config_dir>/themes/*.yaml`.
use crate::inline_style::{InlineStyleOverrides, InlineStyleSet};
use crate::theme::{Color, SyntaxTheme, Theme, ThemeManager, parse_color_value};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const THEMES_DIR: &str = "themes";
const THEME_EXT_YAML: &str = "yaml";
const THEME_EXT_YML: &str = "yml";

mod loading;
mod schema;

pub use loading::load_user_themes;
pub(crate) use schema::{ThemeFile, parse_embedded_theme};

#[cfg(test)]
mod tests;
