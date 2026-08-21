use crate::block_spacing::BlockSpacingOverrides;
use crate::inline_style::InlineStyleOverrides;
use crate::list_marker::{PrettyListStyle, UniformListMarker};
use clap::builder::PossibleValue;
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;
use std::path::PathBuf;

mod help;
use help::*;

#[derive(Parser, Debug)]
#[command(
    name = "mdv",
    version = env!("CARGO_PKG_VERSION"),
    about = "Terminal Markdown Viewer - A fast, feature-rich markdown viewer for the terminal",
    disable_help_subcommand = true,
    long_about = r#"
mdv is a terminal-based markdown viewer that renders markdown files with syntax highlighting, themes, and various formatting options. It supports monitoring files for changes, custom themes, and can output both formatted text and HTML.

Examples:
  mdv README.md                    # View a markdown file
  mdv help                         # Browse the full help
  mdv -t monokai README.md         # Use monokai theme
  mdv --monitor README.md          # Monitor file for changes
  mdv --html README.md             # Output HTML instead of terminal formatting
  mdv -E README.md                 # Render embedded HTML in terminal output
  cat README.md | mdv              # Read from stdin
"#
)]
#[rustfmt::skip]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Path to markdown file (use '-' for stdin)
    #[arg(value_name = "FILE")]
    pub filename: Option<String>,

    /// Strip all ANSI colors
    #[arg(long = "no-colors", help_heading = "Output and flow", display_order = 8)]
    pub no_colors: bool,

    /// Hide Markdown comments from the rendered output
    #[arg(long = "hide-comments", help_heading = "Output and flow", display_order = 9)]
    pub hide_comments: bool,

    /// Render raw HTML fragments as terminal-formatted content
    #[arg(short = 'E', long = "render-html", help_heading = "Output and flow", display_order = 6)]
    pub render_html: bool,

    /// Show rendered row numbers with optional source and separator modes
    #[arg(short = 'N', long = "line-numbers", num_args = 0..=1, value_name = "MODE", value_enum, hide_possible_values = true, help_heading = "Output and flow", display_order = 7, long_help = LINE_NUMBERS_LONG_HELP,)]
    pub line_numbers: Option<Option<LineNumberOptions>>,

    /// Print HTML version instead of terminal formatting
    #[arg(long = "html", help_heading = "Output and flow", display_order = 5)]
    pub do_html: bool,

    /// Show output in the built-in pager instead of printing everything at once
    #[arg(short = 'p', long = "pager", help_heading = "Output and flow", display_order = 0)]
    pub pager: bool,

    /// Browse and read Markdown documents in an interactive terminal interface
    #[arg(short = 'i', long = "interactive", conflicts_with = "pager", help_heading = "Output and flow", display_order = 1)]
    pub interactive: bool,

    /// Fix columns to this width
    #[arg(short = 'c', long = "cols", help_heading = "Layout and wrapping", display_order = 11)]
    pub cols: Option<usize>,

    /// Set theme
    #[arg(short = 't', long = "theme", default_value = "terminal", help_heading = "Themes and code", display_order = 23)]
    pub theme: Option<String>,

    /// Theme for code block highlighting
    #[arg(short = 'T', long = "code-theme", default_value = "terminal", help_heading = "Themes and code", display_order = 24)]
    pub code_theme: Option<String>,

    /// Display empty Markdown elements such as blank code blocks and list items
    #[arg(long = "show-empty-elements", help_heading = "Output and flow", display_order = 10)]
    pub show_empty_elements: bool,

    /// Disable heuristic language detection for code blocks
    #[arg(long = "no-code-guessing", help_heading = "Themes and code", display_order = 34)]
    pub no_code_guessing: bool,

    /// Directory containing custom .sublime-syntax files
    #[arg(long = "syntaxes-dir", value_name = "DIR", help_heading = "Themes and code", display_order = 33, long_help = SYNTAXES_DIR_LONG_HELP,)]
    pub syntaxes_dir: Option<PathBuf>,

    /// Configure visual style for code blocks
    #[arg(short = 'b', long = "code-block-style", value_name = "CODE_STYLE", default_value = "basic", value_parser = parse_code_block_style_config, help_heading = "Themes and code", display_order = 29, long_help = CODE_BLOCK_STYLE_LONG_HELP,)]
    pub code_block_style: Option<CodeBlockStyleConfig>,

    /// Show row numbers inside code blocks with optional source and separator modes
    #[arg(short = 'K', long = "code-line-numbers", num_args = 0..=1, value_name = "MODE", value_enum, hide_possible_values = true, help_heading = "Themes and code", display_order = 30, long_help = CODE_LINE_NUMBERS_LONG_HELP,)]
    pub code_line_numbers: Option<Option<LineNumberOptions>>,

    /// Override code block icon/label/aliases.
    #[arg(long = "custom-code-block", value_name = "BLOCKS", help_heading = "Themes and code", display_order = 31, long_help = CUSTOM_CODE_BLOCK_LONG_HELP,)]
    pub custom_code_block: Option<String>,

    #[arg(short = 'C', long = "callout-style", value_name = "CALLOUT_STYLE", default_value = "pretty", value_parser = parse_callout_style_config, help_heading = "Callouts and lists", display_order = 35, long_help = STYLE_CALLOUT_LONG_HELP,)]
    pub style_callout: Option<CalloutStyleConfig>,

    /// Render task-list checkboxes as Nerd Font icons (requires a Nerd Font terminal)
    #[arg(short = 'x', long = "pretty-checkbox", value_enum, value_name = "SHAPE", help_heading = "Callouts and lists", display_order = 37, long_help = PRETTY_CHECKBOX_LONG_HELP,)]
    pub pretty_checkbox: Option<CheckboxShape>,

    /// Override or add checkbox icons with optional color (e.g. ` :󰀦:yellow`). Requires --pretty-checkbox
    #[arg(long = "custom-checkbox", value_name = "PAIRS", help_heading = "Callouts and lists", display_order = 38, long_help = CUSTOM_CHECKBOX_LONG_HELP,)]
    pub custom_checkbox: Option<String>,

    /// Render unordered list markers with Nerd Font or Unicode icons
    #[arg(short = 'L', long = "pretty-list", value_name = "LIST_STYLE", value_parser = PrettyListStyle::parse, help_heading = "Callouts and lists", display_order = 39, long_help = PRETTY_LIST_LONG_HELP,)]
    pub pretty_list: Option<PrettyListStyle>,

    /// Render definition descriptions with a Unicode or Nerd Font marker
    #[arg(short = 'D', long = "pretty-definition", value_enum, value_name = "STYLE", help_heading = "Callouts and lists", display_order = 42, long_help = PRETTY_DEFINITION_LONG_HELP,)]
    pub pretty_definition: Option<PrettyDefinitionStyle>,

    /// Use one list marker for every nesting level. Requires --pretty-list
    #[arg(long = "uniform-list-marker", value_name = "MARKER", value_parser = UniformListMarker::parse, help_heading = "Callouts and lists", display_order = 40, long_help = UNIFORM_LIST_MARKER_LONG_HELP,)]
    pub uniform_list_marker: Option<UniformListMarker>,

    /// Override list marker icon and/or color per nesting level. Requires --pretty-list
    #[arg(long = "custom-list", value_name = "PAIRS", help_heading = "Callouts and lists", display_order = 41, long_help = CUSTOM_LIST_LONG_HELP,)]
    pub custom_list: Option<String>,
    /// Set hanging indent style for wrapped code block lines
    #[arg(long = "code-wrap-indent", value_enum, value_name = "MODE", default_value = "double", help_heading = "Themes and code", display_order = 32)]
    pub code_wrap_indent: Option<CodeWrapIndent>,

    /// Show current theme and optionally display the contents of FILE when provided
    #[arg(long = "theme-info", value_name = "FILE", num_args = 0..=1, value_hint = clap::ValueHint::FilePath, help_heading = "Themes and code", display_order = 25)]
    pub theme_info: Option<Option<PathBuf>>,

    /// Set tab length
    #[arg(long = "tab-length", default_value = "4", help_heading = "Layout and wrapping", display_order = 13)]
    pub tab_length: Option<usize>,

    /// Set left and right terminal margins
    #[arg(short = 'm', long = "margin", value_name = "MARGINS", value_parser = parse_horizontal_margins, help_heading = "Layout and wrapping", display_order = 12, long_help = MARGIN_LONG_HELP,)]
    pub margin: Option<HorizontalMargins>,

    /// Configure text and table-cell wrapping mode
    #[arg(short = 'w', long = "wrap", value_enum, value_name = "MODE", default_value = "char", help_heading = "Layout and wrapping", display_order = 14)]
    pub wrap_mode: Option<TextWrapMode>,

    /// Reflow paragraphs by collapsing source newlines and refilling to width
    #[arg(long = "reflow", help_heading = "Layout and wrapping", display_order = 15)]
    pub reflow: bool,

    /// Configure table geometry and overflow behavior
    #[arg(short = 'W', long = "table-wrap", value_enum, value_name = "MODE", default_value = "fit", help_heading = "Layout and wrapping", display_order = 19)]
    pub table_wrap_mode: Option<TableWrapMode>,

    /// Render tables with full rounded borders
    #[arg(short = 'B', long = "pretty-table", help_heading = "Layout and wrapping", display_order = 20)]
    pub pretty_table: bool,

    /// Display from given substring of the file
    #[arg(long = "from", value_name = "TEXT", help_heading = "Output and flow", display_order = 3)]
    pub from_txt: Option<String>,

    /// Render document starting from the end while preserving layout
    #[arg(short = 'r', long = "reverse", help_heading = "Output and flow", display_order = 4)]
    pub reverse: bool,

    /// Monitor file for changes and redisplay
    #[arg(long = "monitor", help_heading = "Output and flow", display_order = 2)]
    pub monitor_file: bool,

    /// Override colors of the selected theme (e.g. `text=#ffffff;h1=187,154,247`)
    #[arg(long = "custom-theme", value_name = "PAIRS", help_heading = "Themes and code", display_order = 26)]
    pub custom_theme: Option<String>,

    /// Override inline Markdown element decorations
    #[arg(long = "inline-style", value_name = "STYLES", help_heading = "Themes and code", display_order = 28, long_help = INLINE_STYLE_LONG_HELP,)]
    pub inline_style: Option<InlineStyleOverrides>,

    /// Override syntax highlighting colors (e.g. `keyword=#ffffff;string=128,0,128`)
    #[arg(long = "custom-code-theme", value_name = "PAIRS", help_heading = "Themes and code", display_order = 27)]
    pub custom_code_theme: Option<String>,

    /// Override or create callout styles (e.g. tip:icon=*,color=red;custom:color=#ffffff)
    #[arg(long = "custom-callout", value_name = "CALLOUTS", help_heading = "Callouts and lists", display_order = 36)]
    pub custom_callout: Option<String>,

    /// Set link style
    #[arg(short = 'u', long = "link-style", value_enum, default_value = "clickable", help_heading = "Links and footnotes", display_order = 43)]
    pub link_style: Option<LinkStyle>,

    /// Set link truncation style
    #[arg(short = 'l', long = "link-truncation", value_enum, default_value = "wrap", help_heading = "Links and footnotes", display_order = 44)]
    pub link_truncation: Option<LinkTruncationStyle>,

    /// Configure footnote rendering style
    #[arg(long = "footnote-style", value_enum, value_name = "STYLE", default_value = "endnotes", help_heading = "Links and footnotes", display_order = 45)]
    pub footnote_style: Option<FootnoteStyle>,

    /// Configure handling of missing footnote definitions
    #[arg(long = "missing-footnote-style", value_enum, value_name = "STYLE", default_value = "show", help_heading = "Links and footnotes", display_order = 46)]
    pub missing_footnote_style: Option<MissingFootnoteStyle>,

    /// Directory containing the configuration file.
    #[arg(short = 'F', long = "config-file", value_name = "CONFIG_DIR", help_heading = "Configuration", display_order = 47, long_help = CONFIG_FILE_LONG_HELP,)]
    pub config_file: Option<PathBuf>,

    /// Skip loading configuration files
    #[arg(short = 'n', long = "no-config", help_heading = "Configuration", display_order = 48)]
    pub no_config: bool,

    /// Apply a named built-in or user preset
    #[arg(short = 'P', long = "preset", value_name = "NAME", help_heading = "Configuration", display_order = 49)]
    pub preset: Option<String>,

    /// List presets, or show the active preset while rendering a file
    #[arg(long = "preset-info", help_heading = "Configuration", display_order = 50)]
    pub preset_info: bool,

    /// Create the default configuration file
    #[arg(long = "init-config", num_args = 0..=1, value_name = "CONFIG_DIR", help_heading = "Configuration", display_order = 51)]
    pub init_config: Option<Option<PathBuf>>,

    /// Set heading layout
    #[arg(short = 'H', long = "heading-layout", value_enum, default_value = "level", help_heading = "Layout and wrapping", display_order = 16)]
    pub heading_layout: Option<HeadingLayout>,

    /// Show Markdown-style markers before headings
    #[arg(long = "show-heading-markers", help_heading = "Layout and wrapping", display_order = 17)]
    pub show_heading_markers: bool,

    #[arg(short = 'I', long = "smart-indent", help_heading = "Layout and wrapping", display_order = 18, long_help = SMART_INDENT_LONG_HELP,)]
    pub smart_indent: bool,

    #[arg(short = 'S', long = "table-smart-indent", help = "Automatically adjusts table indentation based on available width", help_heading = "Layout and wrapping", display_order = 21, long_help = TABLE_SMART_INDENT_LONG_HELP,)]
    pub table_smart_indent: bool,

    /// Configure blank lines around block elements
    #[arg(long = "block-spacing", value_name = "SPACING", help_heading = "Layout and wrapping", display_order = 22, long_help = BLOCK_SPACING_LONG_HELP,)]
    pub block_spacing: Option<BlockSpacingOverrides>,
}

mod callouts;
mod code_blocks;
mod commands;
mod layout;
mod line_numbers;
mod links;
mod margins;

pub use callouts::{CalloutStyle, CalloutStyleConfig, CheckboxShape, PrettyDefinitionStyle};
pub use code_blocks::{CodeBlockStyle, CodeBlockStyleConfig, CodeWrapIndent};
pub use commands::CliCommand;
pub use layout::{HeadingLayout, TableWrapMode, TextWrapMode};
pub use line_numbers::{LineNumberOptions, LineNumberTarget};
pub use links::{FootnoteStyle, LinkStyle, LinkTruncationStyle, MissingFootnoteStyle};
pub use margins::HorizontalMargins;

use callouts::parse_callout_style_config;
use code_blocks::parse_code_block_style_config;
use margins::parse_horizontal_margins;

#[cfg(test)]
mod tests;
