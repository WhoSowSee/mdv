use crate::block_spacing::BlockSpacingOverrides;
use crate::callout::{CustomCalloutStyle, parse_custom_callouts};
use crate::cli::{
    CalloutStyleConfig, CheckboxShape, Cli, CodeBlockStyleConfig, CodeWrapIndent, FootnoteStyle,
    FrontMatterMode, HeadingLayout, HorizontalMargins, LineNumberOptions, LineNumberTarget,
    LinkStyle, LinkTruncationStyle, MissingFootnoteStyle, PrettyDefinitionStyle, TableWrapMode,
    TextWrapMode,
};
use crate::custom_code_block::{CustomCodeBlock, parse_custom_code_blocks};
use crate::error::MdvError;
use crate::inline_style::InlineStyleOverrides;
use crate::list_marker::{ListMarkerConfig, PrettyListStyle, UniformListMarker};
use crate::preset;
use anyhow::Result;
use clap::{ArgMatches, parser::ValueSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const CONFIG_FILE_ENV: &str = "MDV_CONFIG_PATH";
const NO_COLOR_ENV: &str = "MDV_NO_COLOR";
const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../assets/config/config.yaml");
const DEFAULT_CONFIG_FILE_NAME: &str = "config.yaml";

fn arg_has_user_value(matches: &ArgMatches, id: &str) -> bool {
    matches
        .value_source(id)
        .map(|source| matches!(source, ValueSource::CommandLine | ValueSource::EnvVariable))
        .unwrap_or(false)
}

/// Expand a leading `~` to the user's home directory.
fn expand_tilde(path: &Path) -> PathBuf {
    let mut components = path.components();
    let Some(Component::Normal(first)) = components.next() else {
        return path.to_path_buf();
    };
    if first != OsStr::new("~") {
        return path.to_path_buf();
    }
    let Some(home) = dirs::home_dir() else {
        return path.to_path_buf();
    };
    components.fold(home, |mut acc, component| {
        match component {
            Component::Normal(part) => acc.push(part),
            Component::RootDir | Component::Prefix(_) => {}
            Component::CurDir | Component::ParentDir => acc.push(component.as_os_str()),
        }
        acc
    })
}

/// Validate that a user-supplied config path is a directory (or a path that can become one).
fn resolve_config_dir(path: &Path) -> Result<PathBuf> {
    let path = expand_tilde(path);
    if path.exists() && !path.is_dir() {
        anyhow::bail!(
            "Config path must be a directory, got a file: {}",
            path.display()
        );
    }
    Ok(path)
}

fn resolve_config_relative_path(path: &Path, config_dir: Option<&Path>) -> PathBuf {
    let path = expand_tilde(path);
    if path.is_absolute() {
        path
    } else if let Some(config_dir) = config_dir {
        config_dir.join(path)
    } else {
        path
    }
}

fn default_config_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        dirs::home_dir().map(|home_dir| home_dir.join(".config").join("mdv"))
    } else {
        dirs::config_dir().map(|config_dir| config_dir.join("mdv"))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LineNumberSetting {
    Enabled(bool),
    Options(String),
}

fn deserialize_line_numbers<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<LineNumberOptions>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Option::<LineNumberSetting>::deserialize(deserializer)? {
        None | Some(LineNumberSetting::Enabled(false)) => Ok(None),
        Some(LineNumberSetting::Enabled(true)) => Ok(Some(LineNumberOptions::default())),
        Some(LineNumberSetting::Options(options)) => LineNumberOptions::parse(&options)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

fn serialize_line_numbers<S>(
    options: &Option<LineNumberOptions>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match options {
        None => serializer.serialize_bool(false),
        Some(options) if *options == LineNumberOptions::default() => {
            serializer.serialize_bool(true)
        }
        Some(options) => serializer.serialize_str(&options.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // Display options
    pub no_colors: bool,
    pub cols: Option<usize>,
    #[serde(skip)]
    pub cols_from_cli: bool,
    pub margin: HorizontalMargins,
    pub tab_length: usize,
    pub theme_info: bool,
    pub wrap: TextWrapMode,
    pub table_wrap: TableWrapMode,
    pub pretty_table: bool,
    pub reflow: bool,
    pub heading_layout: HeadingLayout,
    pub show_heading_markers: bool,
    // Smart heading indentation (applies only to HeadingLayout::Level)
    pub smart_indent: bool,
    pub table_smart_indent: bool,
    pub block_spacing: BlockSpacingOverrides,
    pub hide_comments: bool,
    pub front_matter: FrontMatterMode,
    pub render_html: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_line_numbers",
        serialize_with = "serialize_line_numbers"
    )]
    pub line_numbers: Option<LineNumberOptions>,
    #[serde(
        default,
        deserialize_with = "deserialize_line_numbers",
        serialize_with = "serialize_line_numbers"
    )]
    pub code_line_numbers: Option<LineNumberOptions>,
    #[serde(skip)]
    pub(crate) code_line_number_width: usize,
    #[serde(skip)]
    pub(crate) line_number_gutter_width: usize,
    pub show_empty_elements: bool,
    pub code_guessing: bool,
    pub syntaxes_dir: Option<PathBuf>,
    pub code_block_style: CodeBlockStyleConfig,
    pub callout_style: CalloutStyleConfig,
    pub pretty_checkbox: Option<CheckboxShape>,
    #[serde(default, deserialize_with = "structured::deserialize_custom_checkbox")]
    pub custom_checkbox: Option<String>,
    #[serde(skip)]
    pub(crate) checkbox_overrides: HashMap<char, crate::checkbox_override::CheckboxOverride>,
    pub pretty_list: Option<PrettyListStyle>,
    pub pretty_definition: Option<PrettyDefinitionStyle>,
    pub uniform_list_marker: Option<UniformListMarker>,
    #[serde(default, deserialize_with = "structured::deserialize_custom_list")]
    pub custom_list: Option<String>,
    #[serde(skip)]
    pub(crate) list_marker: ListMarkerConfig,
    pub code_wrap_indent: CodeWrapIndent,
    pub reverse: bool,

    // Theme configuration
    pub theme: String,
    pub code_theme: Option<String>,
    #[serde(default, deserialize_with = "structured::deserialize_theme_overrides")]
    pub custom_theme: Option<String>,
    pub inline_style: InlineStyleOverrides,
    #[serde(default, deserialize_with = "structured::deserialize_theme_overrides")]
    pub custom_code_theme: Option<String>,
    #[serde(default, deserialize_with = "structured::deserialize_custom_callout")]
    pub custom_callout: Option<String>,
    #[serde(skip)]
    pub(crate) custom_callouts: HashMap<String, CustomCalloutStyle>,
    #[serde(
        default,
        deserialize_with = "structured::deserialize_custom_code_block"
    )]
    pub custom_code_block: Option<String>,
    #[serde(skip)]
    pub(crate) custom_code_blocks: HashMap<String, CustomCodeBlock>,
    #[serde(skip)]
    pub(crate) custom_code_default_icon: Option<String>,

    // Link handling
    pub link_style: LinkStyle,
    pub link_truncation: LinkTruncationStyle,
    pub footnote_style: FootnoteStyle,
    pub missing_footnote_style: MissingFootnoteStyle,

    // Content filtering
    pub from_text: Option<String>,

    // File paths
    #[serde(skip)]
    pub config_file: Option<PathBuf>,
    /// Directory used to look up `themes/*.yaml` and `presets/*.yaml`. Mirrors the directory
    /// that produced `config_file` (or the default config dir).
    #[serde(skip)]
    pub config_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            no_colors: false,
            cols: None,
            cols_from_cli: false,
            margin: HorizontalMargins::default(),
            tab_length: 4,
            theme_info: false,
            wrap: TextWrapMode::Char,
            table_wrap: TableWrapMode::Fit,
            pretty_table: false,
            reflow: false,
            heading_layout: HeadingLayout::Level,
            show_heading_markers: false,
            smart_indent: false,
            table_smart_indent: false,
            block_spacing: BlockSpacingOverrides::default(),
            hide_comments: false,
            front_matter: FrontMatterMode::Hidden,
            render_html: false,
            line_numbers: None,
            code_line_numbers: None,
            code_line_number_width: 0,
            line_number_gutter_width: 0,
            show_empty_elements: false,
            code_guessing: true,
            syntaxes_dir: None,
            code_block_style: CodeBlockStyleConfig::default(),
            callout_style: CalloutStyleConfig::default(),
            pretty_list: None,
            pretty_definition: None,
            uniform_list_marker: None,
            custom_list: None,
            list_marker: ListMarkerConfig::default(),
            pretty_checkbox: None,
            custom_checkbox: None,
            checkbox_overrides: HashMap::new(),
            code_wrap_indent: CodeWrapIndent::Double,
            reverse: false,
            theme: "terminal".to_string(),
            code_theme: None,
            custom_theme: None,
            inline_style: InlineStyleOverrides::default(),
            custom_code_theme: None,
            custom_callout: None,
            custom_callouts: HashMap::new(),
            custom_code_block: None,
            custom_code_blocks: HashMap::new(),
            custom_code_default_icon: None,
            link_style: LinkStyle::Clickable,
            link_truncation: LinkTruncationStyle::Wrap,
            footnote_style: FootnoteStyle::Endnotes,
            missing_footnote_style: MissingFootnoteStyle::Show,
            from_text: None,
            config_file: None,
            config_dir: None,
        }
    }
}

mod files;
mod from_cli;
mod merge;
mod runtime;
mod structured;
pub(crate) fn mdv_no_color_override() -> Option<bool> {
    let raw_value = std::env::var_os(NO_COLOR_ENV)?;
    let value = raw_value.to_string_lossy();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.to_ascii_lowercase();
    match normalized.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => {
            log::warn!(
                "Invalid value '{}' for environment variable {}. Use 'True' or 'False'.",
                trimmed,
                NO_COLOR_ENV
            );
            None
        }
    }
}

#[cfg(test)]
mod tests;
