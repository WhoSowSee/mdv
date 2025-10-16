use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "mdv",
    version = env!("CARGO_PKG_VERSION"),
    about = "Terminal Markdown Viewer - A fast, feature-rich markdown viewer for the terminal",
    long_about = r#"
mdv is a terminal-based markdown viewer that renders markdown files with syntax highlighting, themes, and various formatting options. It supports monitoring files for changes, custom themes, and can output both formatted text and HTML.

Examples:
  mdv README.md                    # View a markdown file
  mdv -t monokai README.md         # Use monokai theme
  mdv -m README.md                 # Monitor file for changes
  mdv -H README.md                 # Output HTML instead of terminal formatting
  cat README.md | mdv              # Read from stdin
"#
)]
pub struct Cli {
    /// Path to markdown file (use '-' for stdin)
    #[arg(value_name = "FILE")]
    pub filename: Option<String>,

    /// Alternative config file path
    #[arg(short = 'F', long = "config-file", value_name = "CONFIG_PATH")]
    pub config_file: Option<PathBuf>,

    /// Skip loading configuration files
    #[arg(short = 'n', long = "no-config")]
    pub no_config: bool,

    /// Strip all ANSI colors
    #[arg(short = 'A', long = "no-colors")]
    pub no_colors: bool,

    /// Hide Markdown comments from the rendered output
    #[arg(short = 'C', long = "hide-comments")]
    pub hide_comments: bool,

    /// Print HTML version instead of terminal formatting
    #[arg(short = 'H', long = "html")]
    pub do_html: bool,

    /// Set theme
    #[arg(short = 't', long = "theme", default_value = "terminal")]
    pub theme: Option<String>,

    /// Theme for code block highlighting
    #[arg(short = 'T', long = "code-theme", default_value = "terminal")]
    pub code_theme: Option<String>,

    /// Show language label above code blocks
    #[arg(short = 'L', long = "show-code-language")]
    pub show_code_language: bool,

    /// Display empty Markdown elements such as blank code blocks and list items
    #[arg(short = 'e', long = "show-empty-elements")]
    pub show_empty_elements: bool,

    /// Disable heuristic language detection for code blocks
    #[arg(short = 'g', long = "no-code-guessing")]
    pub no_code_guessing: bool,

    /// Configure visual style for code blocks
    #[arg(
        short = 's',
        long = "style-code-block",
        value_enum,
        default_value = "simple"
    )]
    pub style_code_block: Option<CodeBlockStyle>,

    /// Show current theme and optionally display the contents of FILE when provided
    #[arg(short = 'i', long = "theme-info", value_name = "FILE", num_args = 0..=1, value_hint = clap::ValueHint::FilePath)]
    pub theme_info: Option<Option<PathBuf>>,

    /// Set tab length
    #[arg(short = 'b', long = "tab-length", default_value = "4")]
    pub tab_length: Option<usize>,

    /// Fix columns to this width
    #[arg(short = 'c', long = "cols")]
    pub cols: Option<usize>,

    /// Configure text wrapping mode
    #[arg(
        short = 'W',
        long = "wrap",
        value_enum,
        value_name = "MODE",
        default_value = "char"
    )]
    pub wrap_mode: Option<TextWrapMode>,

    /// Configure table wrapping behavior
    #[arg(
        short = 'w',
        long = "table-wrap",
        value_enum,
        value_name = "MODE",
        default_value = "fit"
    )]
    pub table_wrap_mode: Option<TableWrapMode>,

    /// Display from given substring of the file
    #[arg(short = 'f', long = "from", value_name = "TEXT")]
    pub from_txt: Option<String>,

    /// Monitor file for changes and redisplay
    #[arg(short = 'm', long = "monitor")]
    pub monitor_file: bool,

    /// Override colors of the selected theme (e.g. `text=#ffffff;h1=187,154,247`)
    #[arg(short = 'y', long = "custom-theme", value_name = "PAIRS")]
    pub custom_theme: Option<String>,

    /// Override syntax highlighting colors (e.g. `keyword=#ffffff;string=128,0,128`)
    #[arg(short = 'Y', long = "custom-code-theme", value_name = "PAIRS")]
    pub custom_code_theme: Option<String>,

    /// Set link style
    #[arg(
        short = 'u',
        long = "link-style",
        value_enum,
        default_value = "clickable"
    )]
    pub link_style: Option<LinkStyle>,

    /// Set link truncation style
    #[arg(
        short = 'l',
        long = "link-truncation",
        value_enum,
        default_value = "wrap"
    )]
    pub link_truncation: Option<LinkTruncationStyle>,

    /// Set heading layout
    #[arg(
        short = 'd',
        long = "heading-layout",
        value_enum,
        default_value = "level"
    )]
    pub heading_layout: Option<HeadingLayout>,

    /// Smart indentation for headings when using `--heading-layout level`
    /// compresses large jumps between heading levels so consecutive headings
    /// change indentation gradually (e.g. H1 → H4 indents like H2).
    #[arg(short = 'I', long = "smart-indent")]
    pub smart_indent: bool,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkStyle {
    /// [alias:  c] Link text becomes clickable without showing URL
    #[value(name = "clickable", alias = "c")]
    #[serde(alias = "clickable", alias = "c")]
    Clickable,
    /// [alias: fc] Clickable links with forced underline
    #[value(name = "fclickable", alias = "fc")]
    #[serde(alias = "fclickable", alias = "fc")]
    ClickableForced,
    /// [alias:  i] Link URL after link name
    #[value(name = "inline", alias = "i")]
    #[serde(alias = "inline", alias = "i")]
    Inline,
    /// [alias: it] Index after link name and link URL table after text
    #[value(name = "inlinetable", alias = "it")]
    #[serde(alias = "inlinetable", alias = "it")]
    InlineTable,
    /// [alias:  h] Hide link URLs
    #[value(name = "hide", alias = "h")]
    #[serde(alias = "hide", alias = "h")]
    Hide,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkTruncationStyle {
    /// Wrap links when they don't fit
    #[value(name = "wrap")]
    #[serde(alias = "wrap")]
    Wrap,
    /// Cut links and replace with "..." when they don't fit
    #[value(name = "cut")]
    #[serde(alias = "cut")]
    Cut,
    /// No truncation - links overflow horizontally
    #[value(name = "none")]
    #[serde(alias = "none")]
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextWrapMode {
    #[value(help = "Character-level wrapping")]
    Char,
    #[value(help = "Wrap at word boundaries")]
    Word,
    #[value(help = "Disable wrapping")]
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableWrapMode {
    #[value(help = "Wrap text within table cells, fit to terminal width")]
    Fit,
    #[value(help = "Column wrapping: split table into blocks when too wide")]
    Wrap,
    #[value(help = "No wrapping: tables overflow horizontally")]
    None,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeadingLayout {
    #[value(help = "Level header indent, content indent = 1")]
    Level,
    #[value(help = "Center all headings, no content indentation")]
    Center,
    #[value(help = "No header indentation, content indent = 1")]
    Flat,
    #[value(help = "No indentation for headers and content")]
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeBlockStyle {
    #[value(help = "Classic terminal gutter with single left border")]
    Simple,
    #[value(help = "Box-drawn frame around code blocks")]
    Pretty,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_link_style(value: &str) -> LinkStyle {
        Cli::parse_from(["mdv", "-u", value])
            .link_style
            .expect("link style parsed")
    }

    #[test]
    fn short_flag_accepts_long_link_style_names() {
        assert!(matches!(parse_link_style("inline"), LinkStyle::Inline));
        assert!(matches!(
            parse_link_style("inlinetable"),
            LinkStyle::InlineTable
        ));
        assert!(matches!(
            parse_link_style("clickable"),
            LinkStyle::Clickable
        ));
        assert!(matches!(
            parse_link_style("fclickable"),
            LinkStyle::ClickableForced
        ));
        assert!(matches!(parse_link_style("fc"), LinkStyle::ClickableForced));
        assert!(matches!(parse_link_style("hide"), LinkStyle::Hide));
    }
}
