use crate::callout::{CustomCalloutStyle, parse_custom_callouts};
use crate::cli::{
    CalloutStyleConfig, CheckboxShape, Cli, CodeBlockStyleConfig, CodeWrapIndent, FootnoteStyle,
    HeadingLayout, LinkStyle, LinkTruncationStyle, MissingFootnoteStyle, TableWrapMode,
    TextWrapMode,
};
use crate::custom_code_block::{CustomCodeBlock, parse_custom_code_blocks};
use crate::error::MdvError;
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
const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../docs/examples/config.yaml");
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

fn default_config_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        dirs::home_dir().map(|home_dir| home_dir.join(".config").join("mdv"))
    } else {
        dirs::config_dir().map(|config_dir| config_dir.join("mdv"))
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
    pub tab_length: usize,
    pub theme_info: bool,
    pub wrap: TextWrapMode,
    pub table_wrap: TableWrapMode,
    pub reflow: bool,
    pub heading_layout: HeadingLayout,
    // Smart heading indentation (applies only to HeadingLayout::Level)
    pub smart_indent: bool,
    pub table_smart_indent: bool,
    pub hide_comments: bool,
    pub render_html: bool,
    pub show_empty_elements: bool,
    pub no_code_language: bool,
    pub code_guessing: bool,
    pub code_block_style: CodeBlockStyleConfig,
    pub callout_style: CalloutStyleConfig,
    pub pretty_checkbox: Option<CheckboxShape>,
    pub custom_checkbox: Option<String>,
    #[serde(skip)]
    pub(crate) checkbox_overrides: HashMap<char, crate::checkbox_override::CheckboxOverride>,
    pub code_wrap_indent: CodeWrapIndent,
    pub reverse: bool,

    // Theme configuration
    pub theme: String,
    pub code_theme: Option<String>,
    pub custom_theme: Option<String>,
    pub custom_code_theme: Option<String>,
    pub custom_callout: Option<String>,
    #[serde(skip)]
    pub(crate) custom_callouts: HashMap<String, CustomCalloutStyle>,
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            no_colors: false,
            cols: None,
            cols_from_cli: false,
            tab_length: 4,
            theme_info: false,
            wrap: TextWrapMode::Char,
            table_wrap: TableWrapMode::Fit,
            reflow: false,
            heading_layout: HeadingLayout::Level,
            smart_indent: false,
            table_smart_indent: false,
            hide_comments: false,
            render_html: false,
            show_empty_elements: false,
            no_code_language: false,
            code_guessing: true,
            code_block_style: CodeBlockStyleConfig::default(),
            callout_style: CalloutStyleConfig::default(),
            pretty_checkbox: None,
            custom_checkbox: None,
            checkbox_overrides: HashMap::new(),
            code_wrap_indent: CodeWrapIndent::Double,
            reverse: false,
            theme: "terminal".to_string(),
            code_theme: None,
            custom_theme: None,
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
        }
    }
}

impl Config {
    pub fn from_cli(cli: &Cli, matches: &ArgMatches) -> Result<Self> {
        let mut config = Self::load_config_files(cli, matches)?;

        if let Some(no_colors) = mdv_no_color_override() {
            config.no_colors = no_colors;
        }

        if cli.no_colors {
            config.no_colors = true;
        }

        if let Some(cols) = cli.cols
            && arg_has_user_value(matches, "cols")
        {
            config.cols = Some(cols);
            config.cols_from_cli = true;
        }

        if let Some(tab_length) = cli.tab_length
            && arg_has_user_value(matches, "tab_length")
        {
            config.tab_length = tab_length;
        }

        if let Some(wrap) = cli.wrap_mode
            && arg_has_user_value(matches, "wrap_mode")
        {
            config.wrap = wrap;
        }

        if let Some(table_wrap) = cli.table_wrap_mode
            && arg_has_user_value(matches, "table_wrap_mode")
        {
            config.table_wrap = table_wrap;
        }
        if cli.reflow {
            config.reflow = true;
        }

        if cli.theme_info.is_some() {
            config.theme_info = true;
        }

        if cli.no_code_guessing {
            config.code_guessing = false;
        }

        if let Some(theme) = &cli.theme
            && arg_has_user_value(matches, "theme")
        {
            config.theme = theme.clone();
        }

        if let Some(code_theme) = &cli.code_theme
            && arg_has_user_value(matches, "code_theme")
        {
            config.code_theme = Some(code_theme.clone());
        }

        if let Some(custom_theme) = &cli.custom_theme
            && arg_has_user_value(matches, "custom_theme")
        {
            config.custom_theme = Some(custom_theme.clone());
        }

        if let Some(custom_code_theme) = &cli.custom_code_theme
            && arg_has_user_value(matches, "custom_code_theme")
        {
            config.custom_code_theme = Some(custom_code_theme.clone());
        }

        if let Some(custom_callout) = &cli.custom_callout
            && arg_has_user_value(matches, "custom_callout")
        {
            config.custom_callout = Some(custom_callout.clone());
        }

        if let Some(custom_code_block) = &cli.custom_code_block
            && arg_has_user_value(matches, "custom_code_block")
        {
            config.custom_code_block = Some(custom_code_block.clone());
        }

        if let Some(link_style) = cli.link_style.clone()
            && arg_has_user_value(matches, "link_style")
        {
            config.link_style = link_style;
        }

        if let Some(link_truncation) = cli.link_truncation.clone()
            && arg_has_user_value(matches, "link_truncation")
        {
            config.link_truncation = link_truncation;
        }

        if let Some(footnote_style) = cli.footnote_style
            && arg_has_user_value(matches, "footnote_style")
        {
            config.footnote_style = footnote_style;
        }

        if let Some(missing_style) = cli.missing_footnote_style
            && arg_has_user_value(matches, "missing_footnote_style")
        {
            config.missing_footnote_style = missing_style;
        }

        if let Some(heading_layout) = cli.heading_layout.clone()
            && arg_has_user_value(matches, "heading_layout")
        {
            config.heading_layout = heading_layout;
        }
        if cli.smart_indent {
            config.smart_indent = true;
        }
        if cli.table_smart_indent {
            config.table_smart_indent = true;
        }

        if cli.hide_comments {
            config.hide_comments = true;
        }

        if cli.render_html {
            config.render_html = true;
        }

        if cli.show_empty_elements {
            config.show_empty_elements = true;
        }

        if cli.no_code_language {
            config.no_code_language = true;
        }

        if let Some(style) = cli.code_block_style
            && arg_has_user_value(matches, "code_block_style")
        {
            config.code_block_style = style;
        }
        if let Some(style) = cli.style_callout
            && arg_has_user_value(matches, "style_callout")
        {
            config.callout_style = style;
        }
        if let Some(shape) = cli.pretty_checkbox
            && arg_has_user_value(matches, "pretty_checkbox")
        {
            config.pretty_checkbox = Some(shape);
        }

        if let Some(raw) = &cli.custom_checkbox
            && arg_has_user_value(matches, "custom_checkbox")
        {
            config.custom_checkbox = Some(raw.clone());
        }

        if let Some(indent) = cli.code_wrap_indent
            && arg_has_user_value(matches, "code_wrap_indent")
        {
            config.code_wrap_indent = indent;
        }

        if let Some(from_text) = &cli.from_txt
            && arg_has_user_value(matches, "from_txt")
        {
            config.from_text = Some(from_text.clone());
        }

        if cli.reverse {
            config.reverse = true;
        }

        config.normalize_theme_settings();
        config.apply_custom_callouts()?;
        config.apply_custom_code_blocks()?;
        config.apply_checkbox_overrides()?;

        Ok(config)
    }

    pub(crate) fn write_default_config(cli: &Cli, matches: &ArgMatches) -> Result<PathBuf> {
        let dir = if let Some(Some(ref path)) = cli.init_config {
            resolve_config_dir(path)?
        } else if let Some(ref config_file) = cli.config_file
            && arg_has_user_value(matches, "config_file")
        {
            resolve_config_dir(config_file)?
        } else if let Some(env_path) = std::env::var_os(CONFIG_FILE_ENV)
            && !env_path.is_empty()
        {
            resolve_config_dir(Path::new(&env_path))?
        } else {
            default_config_dir()
                .ok_or_else(|| anyhow::anyhow!("Unable to determine user config directory"))?
        };

        let path = dir.join(DEFAULT_CONFIG_FILE_NAME);

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                anyhow::bail!("Config file already exists: {}", path.display());
            }
            Err(error) => return Err(error.into()),
        };
        file.write_all(DEFAULT_CONFIG_TEMPLATE.as_bytes())?;

        Ok(path)
    }

    fn load_config_files(cli: &Cli, matches: &ArgMatches) -> Result<Self> {
        if cli.no_config {
            return Ok(Self::default());
        }

        let mut config = Self::default();

        let config_paths = Self::get_config_paths(cli, matches)?;

        for path in config_paths {
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(file_config) => {
                        config.merge_with(file_config);
                        config.config_file = Some(path.clone());
                        break;
                    }
                    Err(e) => {
                        log::warn!("Failed to load config from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(config)
    }

    fn get_config_paths(cli: &Cli, matches: &ArgMatches) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        if let Some(config_file) = &cli.config_file
            && arg_has_user_value(matches, "config_file")
        {
            let dir = resolve_config_dir(config_file)?;
            paths.push(dir.join("config.yaml"));
            paths.push(dir.join("config.yml"));
        }

        if let Some(env_path) = std::env::var_os(CONFIG_FILE_ENV)
            && !env_path.is_empty()
        {
            let dir = resolve_config_dir(Path::new(&env_path))?;
            paths.push(dir.join("config.yaml"));
            paths.push(dir.join("config.yml"));
        }

        if let Some(mdv_dir) = default_config_dir() {
            paths.push(mdv_dir.join("config.yaml"));
            paths.push(mdv_dir.join("config.yml"));
        }

        Ok(paths)
    }

    fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        serde_yaml::from_str::<Self>(&content).map_err(|_| {
            anyhow::Error::from(MdvError::ConfigParseError(format!(
                "Failed to parse YAML config file: {}",
                path.display()
            )))
        })
    }

    fn merge_with(&mut self, other: Self) {
        if other.no_colors {
            self.no_colors = other.no_colors;
        }

        if other.cols.is_some() {
            self.cols = other.cols;
        }

        if other.cols_from_cli {
            self.cols_from_cli = true;
        }

        if other.tab_length != 4 {
            self.tab_length = other.tab_length;
        }

        if other.theme_info {
            self.theme_info = other.theme_info;
        }

        if !matches!(other.wrap, TextWrapMode::Char) {
            self.wrap = other.wrap;
        }

        if !matches!(other.table_wrap, TableWrapMode::Fit) {
            self.table_wrap = other.table_wrap;
        }
        if other.reflow {
            self.reflow = true;
        }
        // heading_layout defaults to Level; merge when non-default
        if !matches!(other.heading_layout, HeadingLayout::Level) {
            self.heading_layout = other.heading_layout;
        }
        if other.smart_indent {
            self.smart_indent = true;
        }
        if other.table_smart_indent {
            self.table_smart_indent = true;
        }

        if other.hide_comments {
            self.hide_comments = true;
        }
        if other.render_html {
            self.render_html = true;
        }
        if other.show_empty_elements {
            self.show_empty_elements = true;
        }
        if other.no_code_language {
            self.no_code_language = true;
        }
        if !other.code_guessing {
            self.code_guessing = false;
        }
        if other.code_block_style != CodeBlockStyleConfig::default() {
            self.code_block_style = other.code_block_style;
        }
        if other.callout_style != CalloutStyleConfig::default() {
            self.callout_style = other.callout_style;
        }
        if other.pretty_checkbox.is_some() {
            self.pretty_checkbox = other.pretty_checkbox;
        }
        if other.custom_checkbox.is_some() {
            self.custom_checkbox = other.custom_checkbox.clone();
        }
        if !matches!(other.code_wrap_indent, CodeWrapIndent::Double) {
            self.code_wrap_indent = other.code_wrap_indent;
        }

        if other.theme != "terminal" {
            self.theme = other.theme;
        }

        if other.code_theme.is_some() {
            self.code_theme = other.code_theme;
        }

        if other.custom_theme.is_some() {
            self.custom_theme = other.custom_theme;
        }

        if other.custom_code_theme.is_some() {
            self.custom_code_theme = other.custom_code_theme;
        }
        if other.custom_callout.is_some() {
            self.custom_callout = other.custom_callout;
        }

        if other.custom_code_block.is_some() {
            self.custom_code_block = other.custom_code_block;
        }

        if other.custom_code_default_icon.is_some() {
            self.custom_code_default_icon = other.custom_code_default_icon;
        }

        if !matches!(other.link_style, LinkStyle::Clickable) {
            self.link_style = other.link_style;
        }

        if !matches!(other.link_truncation, LinkTruncationStyle::Wrap) {
            self.link_truncation = other.link_truncation;
        }

        if !matches!(other.footnote_style, FootnoteStyle::Endnotes) {
            self.footnote_style = other.footnote_style;
        }

        if !matches!(other.missing_footnote_style, MissingFootnoteStyle::Show) {
            self.missing_footnote_style = other.missing_footnote_style;
        }

        if other.from_text.is_some() {
            self.from_text = other.from_text;
        }

        if other.reverse {
            self.reverse = true;
        }
    }

    pub fn text_wrap_mode(&self) -> crate::utils::WrapMode {
        match self.wrap {
            TextWrapMode::Char => crate::utils::WrapMode::Character,
            TextWrapMode::Word => crate::utils::WrapMode::Word,
            TextWrapMode::None => crate::utils::WrapMode::None,
        }
    }

    pub fn is_text_wrapping_enabled(&self) -> bool {
        !matches!(self.wrap, TextWrapMode::None)
    }

    pub fn get_terminal_width(&self) -> usize {
        if self.cols_from_cli
            && let Some(cols) = self.cols
        {
            return cols;
        }

        if let Ok((width, _)) = crossterm::terminal::size() {
            let width = width as usize;
            if width >= 20 {
                return width;
            }
        }

        if let Some(cols) = self.cols {
            return cols;
        }

        80 // Default fallback
    }

    fn normalize_theme_settings(&mut self) {
        if self.theme.trim().is_empty() {
            self.theme = "terminal".to_string();
        }

        if let Some(code_theme) = self.code_theme.as_ref()
            && code_theme.trim().is_empty()
        {
            self.code_theme = None;
        }
    }

    fn apply_custom_callouts(&mut self) -> Result<()> {
        self.custom_callouts.clear();
        if let Some(raw) = &self.custom_callout {
            self.custom_callouts = parse_custom_callouts(raw)?;
        }
        Ok(())
    }

    fn apply_checkbox_overrides(&mut self) -> Result<()> {
        self.checkbox_overrides.clear();
        if self.pretty_checkbox.is_none() {
            return Ok(());
        }
        let Some(raw) = &self.custom_checkbox else {
            return Ok(());
        };
        for entry in raw.split(';') {
            match crate::checkbox_override::CheckboxOverride::parse_entry(entry) {
                Ok(Some((ch, ov))) => {
                    self.checkbox_overrides.insert(ch, ov);
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn apply_custom_code_blocks(&mut self) -> Result<()> {
        self.custom_code_blocks.clear();
        self.custom_code_default_icon = None;
        if let Some(raw) = &self.custom_code_block {
            let mut parsed = parse_custom_code_blocks(raw)?;
            if let Some(block) = parsed.remove("default") {
                self.custom_code_default_icon = block.icon;
            }
            self.custom_code_blocks = parsed;
        }
        Ok(())
    }
}

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
mod tests {
    use super::*;
    use crate::cli::Cli;

    use clap::{Arg, Command, CommandFactory, FromArgMatches};
    use std::ffi::{OsStr, OsString};
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    #[test]
    fn cli_cols_override_terminal_width() {
        let _env_lock = env_lock();
        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("-c"),
            OsString::from("42"),
        ]);

        let config = Config::from_cli(&cli, &matches).expect("load config");
        assert_eq!(config.cols, Some(42));
        assert!(config.cols_from_cli);
        assert_eq!(config.get_terminal_width(), 42);
    }

    struct EnvVarGuard {
        key: &'static str,
        original: Option<OsString>,
    }

    fn set_env_var<K, V>(key: K, value: V)
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn remove_env_var<K>(key: K)
    where
        K: AsRef<OsStr>,
    {
        unsafe {
            std::env::remove_var(key);
        }
    }

    impl EnvVarGuard {
        fn set_temp<K>(key: &'static str, value: K) -> Self
        where
            K: AsRef<OsStr>,
        {
            let original = std::env::var_os(key);
            set_env_var(key, value);
            Self { key, original }
        }
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock env mutex")
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(ref value) = self.original {
                set_env_var(self.key, value);
            } else {
                remove_env_var(self.key);
            }
        }
    }

    fn parse_cli_from(args: Vec<OsString>) -> (Cli, clap::ArgMatches) {
        let matches = Cli::command().get_matches_from(args);
        let cli = Cli::from_arg_matches(&matches).expect("parse cli from matches");
        (cli, matches)
    }

    fn parse_with_config(config_contents: &str) -> Config {
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, config_contents).expect("write config file");

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--config-file"),
            temp_dir.path().as_os_str().to_owned(),
        ]);

        Config::from_cli(&cli, &matches).expect("load config")
    }

    #[test]
    fn no_config_flag_skips_loading_files() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "no_colors: true\n").expect("write config file");

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--config-file"),
            temp_dir.path().as_os_str().to_owned(),
            OsString::from("--no-config"),
        ]);

        let config = Config::from_cli(&cli, &matches).expect("load config");
        assert!(
            !config.no_colors,
            "config file should be ignored when --no-config is set"
        );
    }

    #[test]
    fn config_file_settings_survive_cli_defaults() {
        let _env_lock = env_lock();
        let config = parse_with_config(
            r#"
no_colors: true
wrap: word
table_wrap: wrap
tab_length: 2
heading_layout: flat
table_smart_indent: true
link_style: inline
link_truncation: cut
"#,
        );

        assert!(config.no_colors);
        assert!(matches!(config.wrap, TextWrapMode::Word));
        assert!(matches!(config.table_wrap, TableWrapMode::Wrap));
        assert_eq!(config.tab_length, 2);
        assert!(matches!(config.heading_layout, HeadingLayout::Flat));
        assert!(config.table_smart_indent);
        assert!(matches!(config.link_style, LinkStyle::Inline));
        assert!(matches!(config.link_truncation, LinkTruncationStyle::Cut));
    }

    #[test]
    fn config_file_parses_tablecut_link_truncation() {
        let _env_lock = env_lock();
        let config = parse_with_config(
            r#"
link_style: inline
link_truncation: tablecut
"#,
        );

        assert!(matches!(config.link_style, LinkStyle::Inline));
        assert!(matches!(
            config.link_truncation,
            LinkTruncationStyle::TableCut
        ));
    }

    #[test]
    fn config_cols_from_file_does_not_mark_cli_override() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "cols: 70\n").expect("write config file");

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--config-file"),
            temp_dir.path().as_os_str().to_owned(),
        ]);

        let config = Config::from_cli(&cli, &matches).expect("load config");
        assert_eq!(config.cols, Some(70));
        assert!(!config.cols_from_cli);
    }

    #[test]
    fn cli_arguments_override_config_when_provided() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "wrap: word\nlink_style: inline\n")
            .expect("write config file");

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--config-file"),
            temp_dir.path().as_os_str().to_owned(),
            OsString::from("--wrap"),
            OsString::from("none"),
            OsString::from("--link-style"),
            OsString::from("hide"),
        ]);

        let config = Config::from_cli(&cli, &matches).expect("load config with overrides");
        assert!(matches!(config.wrap, TextWrapMode::None));
        assert!(matches!(config.link_style, LinkStyle::Hide));
    }

    #[test]
    fn empty_theme_from_cli_falls_back_to_default() {
        let _env_lock = env_lock();
        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--theme"),
            OsString::from(""),
        ]);

        let config = Config::from_cli(&cli, &matches).expect("load config with empty theme");
        assert_eq!(config.theme, "terminal");
    }

    #[test]
    fn empty_theme_in_config_file_falls_back_to_default() {
        let _env_lock = env_lock();
        let config = parse_with_config("theme: \"\"\n");

        assert_eq!(config.theme, "terminal");
    }

    #[test]
    fn empty_code_theme_input_clears_override() {
        let _env_lock = env_lock();
        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--code-theme"),
            OsString::from(""),
        ]);

        let config = Config::from_cli(&cli, &matches).expect("load config with empty code theme");
        assert!(config.code_theme.is_none());
    }

    #[test]
    fn empty_code_theme_in_config_file_is_ignored() {
        let _env_lock = env_lock();
        let config = parse_with_config("code_theme: \"\"\n");

        assert!(config.code_theme.is_none());
    }

    #[test]
    fn environment_no_color_true_sets_flag() {
        let _env_lock = env_lock();
        let _guard = EnvVarGuard::set_temp(NO_COLOR_ENV, "True");
        assert_eq!(mdv_no_color_override(), Some(true));
        let (cli, matches) = parse_cli_from(vec![OsString::from("mdv")]);

        let config = Config::from_cli(&cli, &matches).expect("load config from env");
        assert!(config.no_colors, "True must disable colors");
    }

    #[test]
    fn environment_no_color_false_overrides_config() {
        let _env_lock = env_lock();
        let _guard = EnvVarGuard::set_temp(NO_COLOR_ENV, "False");
        assert_eq!(mdv_no_color_override(), Some(false));
        let config = parse_with_config("no_colors: true\n");

        assert!(!config.no_colors, "False must allow colors");
    }

    #[test]
    fn environment_config_path_is_used() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "no_colors: true\n").expect("write config file");

        let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, temp_dir.path().as_os_str());
        let (cli, matches) = parse_cli_from(vec![OsString::from("mdv")]);

        let config = Config::from_cli(&cli, &matches).expect("load config from env");
        assert!(config.no_colors, "environment config should be applied");
        assert_eq!(
            config.config_file.as_deref(),
            Some(config_path.as_path()),
            "config should record loaded path"
        );
    }

    #[test]
    fn arg_has_user_value_detects_command_line_sources() {
        let matches = Cli::command().get_matches_from(vec![
            OsString::from("mdv"),
            OsString::from("--wrap"),
            OsString::from("none"),
        ]);

        assert!(arg_has_user_value(&matches, "wrap_mode"));
    }

    #[test]
    fn arg_has_user_value_ignores_default_values() {
        let matches = Command::new("mdv-test")
            .arg(Arg::new("opt").default_value("foo"))
            .get_matches_from(vec!["mdv-test"]);

        assert!(!arg_has_user_value(&matches, "opt"));
    }

    #[test]
    fn config_file_rejects_existing_file_path() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "no_colors: true\n").expect("write config file");

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--config-file"),
            config_path.as_os_str().to_owned(),
        ]);

        let error = Config::from_cli(&cli, &matches).expect_err("file path must fail");
        assert!(
            error.to_string().contains("must be a directory"),
            "error: {error}"
        );
    }

    #[test]
    fn expand_tilde_replaces_leading_tilde_with_home_dir() {
        let home = dirs::home_dir().expect("home directory");
        assert_eq!(expand_tilde(Path::new("~")), home);
        assert_eq!(
            expand_tilde(Path::new("~/.config/mdv")),
            home.join(".config").join("mdv")
        );
        assert_eq!(
            expand_tilde(Path::new("not/tilde/path")),
            Path::new("not/tilde/path")
        );
    }

    #[test]
    fn default_config_template_matches_default_settings() {
        let config: Config =
            serde_yaml::from_str(DEFAULT_CONFIG_TEMPLATE).expect("default config template parses");

        assert_eq!(config.theme, "terminal");
        assert!(config.code_theme.is_none());
        assert!(config.cols.is_none());
        assert!(!config.smart_indent);
        assert!(!config.render_html);
        assert!(matches!(config.link_style, LinkStyle::Clickable));
    }

    #[test]
    fn write_default_config_uses_init_config_path() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--init-config"),
            temp_dir.path().as_os_str().to_owned(),
        ]);

        let written_path =
            Config::write_default_config(&cli, &matches).expect("write default config");
        let expected_path = temp_dir.path().join("config.yaml");

        assert_eq!(written_path, expected_path);
        assert!(expected_path.exists());
        assert_eq!(
            std::fs::read_to_string(&expected_path).expect("read generated config"),
            DEFAULT_CONFIG_TEMPLATE
        );
    }

    #[test]
    fn write_default_config_uses_config_file_path() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--init-config"),
            OsString::from("--config-file"),
            temp_dir.path().join("nested").as_os_str().to_owned(),
        ]);

        let written_path =
            Config::write_default_config(&cli, &matches).expect("write default config");
        let expected_path = temp_dir.path().join("nested").join("config.yaml");

        assert_eq!(written_path, expected_path);
        assert!(expected_path.exists());
    }

    #[test]
    fn write_default_config_uses_environment_config_path() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, temp_dir.path().as_os_str());
        let (cli, matches) =
            parse_cli_from(vec![OsString::from("mdv"), OsString::from("--init-config")]);

        let written_path =
            Config::write_default_config(&cli, &matches).expect("write default config from env");
        let expected_path = temp_dir.path().join("config.yaml");

        assert_eq!(written_path, expected_path);
        assert!(expected_path.exists());
    }

    #[test]
    fn write_default_config_prefers_init_config_path_over_config_file_and_environment() {
        let _env_lock = env_lock();
        let temp_dir = TempDir::new().expect("create temp dir");
        let env_path = temp_dir.path().join("env");
        let config_file_path = temp_dir.path().join("config-file");
        let init_path = temp_dir.path().join("init");
        let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, env_path.as_os_str());

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--init-config"),
            init_path.clone().into_os_string(),
            OsString::from("--config-file"),
            config_file_path.clone().into_os_string(),
        ]);

        let written_path =
            Config::write_default_config(&cli, &matches).expect("write default config");
        let expected_path = init_path.join("config.yaml");

        assert_eq!(written_path, expected_path);
        assert!(expected_path.exists());
        assert!(!config_file_path.join("config.yaml").exists());
        assert!(!env_path.join("config.yaml").exists());
    }

    #[test]
    fn write_default_config_refuses_existing_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "theme: \"monokai\"\n").expect("write config file");

        let (cli, matches) = parse_cli_from(vec![
            OsString::from("mdv"),
            OsString::from("--init-config"),
            temp_dir.path().as_os_str().to_owned(),
        ]);

        let error =
            Config::write_default_config(&cli, &matches).expect_err("existing config must fail");
        assert!(error.to_string().contains("Config file already exists"));
        assert_eq!(
            std::fs::read_to_string(&config_path).expect("read existing config"),
            "theme: \"monokai\"\n"
        );
    }
}
