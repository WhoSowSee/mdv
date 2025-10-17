use crate::cli::{
    Cli, CodeBlockStyle, HeadingLayout, LinkStyle, LinkTruncationStyle, TableWrapMode, TextWrapMode,
};
use crate::error::MdvError;
use anyhow::Result;
use clap::{ArgMatches, parser::ValueSource};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILE_ENV: &str = "MDV_CONFIG_PATH";
const NO_COLOR_ENV: &str = "MDV_NO_COLOR";

fn arg_has_user_value(matches: &ArgMatches, id: &str) -> bool {
    matches
        .value_source(id)
        .map(|source| matches!(source, ValueSource::CommandLine | ValueSource::EnvVariable))
        .unwrap_or(false)
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
    pub heading_layout: HeadingLayout,
    // Smart heading indentation (applies only to HeadingLayout::Level)
    pub smart_indent: bool,
    pub hide_comments: bool,
    pub show_empty_elements: bool,
    pub no_code_language: bool,
    pub code_guessing: bool,
    pub code_block_style: CodeBlockStyle,
    pub reverse: bool,

    // Theme configuration
    pub theme: String,
    pub code_theme: Option<String>,
    pub custom_theme: Option<String>,
    pub custom_code_theme: Option<String>,

    // Link handling
    pub link_style: LinkStyle,
    pub link_truncation: LinkTruncationStyle,

    // Content filtering
    pub from_text: Option<String>,

    // File paths
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
            heading_layout: HeadingLayout::Level,
            smart_indent: false,
            hide_comments: false,
            show_empty_elements: false,
            no_code_language: false,
            code_guessing: true,
            code_block_style: CodeBlockStyle::Pretty,
            reverse: false,
            theme: "terminal".to_string(),
            code_theme: None,
            custom_theme: None,
            custom_code_theme: None,
            link_style: LinkStyle::Clickable,
            link_truncation: LinkTruncationStyle::Wrap,
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

        if let Some(cols) = cli.cols {
            if arg_has_user_value(matches, "cols") {
                config.cols = Some(cols);
                config.cols_from_cli = true;
            }
        }

        if let Some(tab_length) = cli.tab_length {
            if arg_has_user_value(matches, "tab_length") {
                config.tab_length = tab_length;
            }
        }

        if let Some(wrap) = cli.wrap_mode {
            if arg_has_user_value(matches, "wrap_mode") {
                config.wrap = wrap;
            }
        }

        if let Some(table_wrap) = cli.table_wrap_mode {
            if arg_has_user_value(matches, "table_wrap_mode") {
                config.table_wrap = table_wrap;
            }
        }

        if cli.theme_info.is_some() {
            config.theme_info = true;
        }

        if cli.no_code_guessing {
            config.code_guessing = false;
        }

        if let Some(theme) = &cli.theme {
            if arg_has_user_value(matches, "theme") {
                config.theme = theme.clone();
            }
        }

        if let Some(code_theme) = &cli.code_theme {
            if arg_has_user_value(matches, "code_theme") {
                config.code_theme = Some(code_theme.clone());
            }
        }

        if let Some(custom_theme) = &cli.custom_theme {
            if arg_has_user_value(matches, "custom_theme") {
                config.custom_theme = Some(custom_theme.clone());
            }
        }

        if let Some(custom_code_theme) = &cli.custom_code_theme {
            if arg_has_user_value(matches, "custom_code_theme") {
                config.custom_code_theme = Some(custom_code_theme.clone());
            }
        }

        if let Some(link_style) = cli.link_style.clone() {
            if arg_has_user_value(matches, "link_style") {
                config.link_style = link_style;
            }
        }

        if let Some(link_truncation) = cli.link_truncation.clone() {
            if arg_has_user_value(matches, "link_truncation") {
                config.link_truncation = link_truncation;
            }
        }

        if let Some(heading_layout) = cli.heading_layout.clone() {
            if arg_has_user_value(matches, "heading_layout") {
                config.heading_layout = heading_layout;
            }
        }
        if cli.smart_indent {
            config.smart_indent = true;
        }

        if cli.hide_comments {
            config.hide_comments = true;
        }

        if cli.show_empty_elements {
            config.show_empty_elements = true;
        }

        if cli.no_code_language {
            config.no_code_language = true;
        }

        if let Some(style) = cli.style_code_block {
            if arg_has_user_value(matches, "style_code_block") {
                config.code_block_style = style;
            }
        }

        if let Some(from_text) = &cli.from_txt {
            if arg_has_user_value(matches, "from_txt") {
                config.from_text = Some(from_text.clone());
            }
        }

        if cli.reverse {
            config.reverse = true;
        }

        Ok(config)
    }

    fn load_config_files(cli: &Cli, matches: &ArgMatches) -> Result<Self> {
        if cli.no_config {
            return Ok(Self::default());
        }

        let mut config = Self::default();

        let config_paths = Self::get_config_paths(cli, matches);

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

    fn get_config_paths(cli: &Cli, matches: &ArgMatches) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(config_file) = &cli.config_file {
            if arg_has_user_value(matches, "config_file") {
                paths.push(config_file.clone());
            }
        }

        if let Some(env_path) = std::env::var_os(CONFIG_FILE_ENV) {
            if !env_path.is_empty() {
                paths.push(PathBuf::from(env_path));
            }
        }

        if cfg!(target_os = "windows") {
            if let Some(home_dir) = dirs::home_dir() {
                let mdv_dir = home_dir.join(".config").join("mdv");
                paths.push(mdv_dir.join("config.yaml"));
                paths.push(mdv_dir.join("config.yml"));
            }
        } else if let Some(config_dir) = dirs::config_dir() {
            let mdv_dir = config_dir.join("mdv");
            paths.push(mdv_dir.join("config.yaml"));
            paths.push(mdv_dir.join("config.yml"));
        }

        paths
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
        // heading_layout defaults to Level; merge when non-default
        if !matches!(other.heading_layout, HeadingLayout::Level) {
            self.heading_layout = other.heading_layout;
        }
        if other.smart_indent {
            self.smart_indent = true;
        }

        if other.hide_comments {
            self.hide_comments = true;
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
        if !matches!(other.code_block_style, CodeBlockStyle::Pretty) {
            self.code_block_style = other.code_block_style;
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

        if !matches!(other.link_style, LinkStyle::Clickable) {
            self.link_style = other.link_style;
        }

        if !matches!(other.link_truncation, LinkTruncationStyle::Wrap) {
            self.link_truncation = other.link_truncation;
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
        if self.cols_from_cli {
            if let Some(cols) = self.cols {
                return cols;
            }
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
            config_path.clone().into_os_string(),
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
            config_path.clone().into_os_string(),
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
link_style: inline
link_truncation: cut
"#,
        );

        assert!(config.no_colors);
        assert!(matches!(config.wrap, TextWrapMode::Word));
        assert!(matches!(config.table_wrap, TableWrapMode::Wrap));
        assert_eq!(config.tab_length, 2);
        assert!(matches!(config.heading_layout, HeadingLayout::Flat));
        assert!(matches!(config.link_style, LinkStyle::Inline));
        assert!(matches!(config.link_truncation, LinkTruncationStyle::Cut));
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
            config_path.clone().into_os_string(),
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
            config_path.clone().into_os_string(),
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

        let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, config_path.as_os_str());
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
}
