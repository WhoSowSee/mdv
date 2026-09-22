use crate::block_spacing::BlockSpacingOverrides;
use crate::inline_style::InlineStyleOverrides;
use crate::list_marker::{PrettyListStyle, UniformListMarker};
use clap::builder::PossibleValue;
use clap::{ColorChoice, Parser, Subcommand, ValueEnum};
use std::fmt;
use std::path::PathBuf;

mod color;
mod color_depth;
mod help;
pub use color::{ColorMode, OutputStyle};
pub use color_depth::ColorDepth;
use help::*;

#[derive(Parser, Debug)]
#[command(
    name = "mdv",
    version = env!("CARGO_PKG_VERSION"),
    about = "Render Markdown in the terminal",
    disable_help_subcommand = true,
    color = ColorChoice::Never,
    long_about = r#"
Render Markdown in the terminal with syntax highlighting and configurable themes.
Read files or standard input, browse documents interactively, or export HTML.

Examples:
  mdv README.md                   # View a Markdown file
  mdv help                        # Browse the full help
  mdv --theme monokai README.md   # Use the monokai theme
  mdv --monitor README.md         # Reload when the file changes
  mdv --html README.md            # Export HTML
  mdv --render-html README.md     # Render embedded HTML in the terminal
  cat README.md | mdv             # Read from standard input
"#
)]
#[rustfmt::skip]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Read a Markdown file (use '-' for standard input)
    #[arg(value_name = "FILE")]
    pub filename: Option<String>,

    /// Set when to use terminal styling and hyperlinks
    #[arg(long = "color", value_enum, value_name = "WHEN", default_value = "auto", help_heading = "Output and flow", display_order = 8)]
    pub color: Option<ColorMode>,

    /// Detect terminal color depth or limit colors explicitly
    #[arg(long = "color-depth", value_enum, value_name = "DEPTH", default_value = "auto", help_heading = "Output and flow", display_order = 8)]
    pub color_depth: Option<ColorDepth>,

    /// Hide Markdown comments from the rendered output
    #[arg(long = "hide-comments", help_heading = "Output and flow", display_order = 9)]
    pub hide_comments: bool,

    /// Set how to display YAML front matter at the beginning of a document
    #[arg(long = "front-matter", value_enum, value_name = "MODE", default_value = "hidden", help_heading = "Output and flow", display_order = 10)]
    pub front_matter: Option<FrontMatterMode>,

    /// Render raw HTML fragments as terminal-formatted content
    #[arg(short = 'E', long = "render-html", help_heading = "Output and flow", display_order = 6)]
    pub render_html: bool,

    /// Show row numbers in terminal and pager output
    #[arg(short = 'N', long = "line-numbers", num_args = 0..=1, value_name = "MODE", value_enum, hide_possible_values = true, help_heading = "Output and flow", display_order = 7, long_help = LINE_NUMBERS_LONG_HELP,)]
    pub line_numbers: Option<Option<LineNumberOptions>>,

    /// Output HTML instead of terminal-formatted text
    #[arg(long = "html", help_heading = "Output and flow", display_order = 5)]
    pub do_html: bool,

    /// Show output in a pager instead of printing everything at once
    #[arg(short = 'p', long = "pager", num_args = 0..=1, require_equals = true, value_name = "COMMAND", help_heading = "Output and flow", display_order = 0, long_help = PAGER_LONG_HELP)]
    pub pager: Option<Option<String>>,

    /// Browse and read Markdown documents in an interactive terminal interface
    #[arg(short = 'i', long = "interactive", conflicts_with = "pager", help_heading = "Output and flow", display_order = 1)]
    pub interactive: bool,

    /// Set the output width in terminal columns
    #[arg(short = 'c', long = "cols", help_heading = "Layout and wrapping", display_order = 11)]
    pub cols: Option<usize>,

    /// Set the document theme
    #[arg(short = 't', long = "theme", default_value = "terminal", help_heading = "Themes and code", display_order = 23)]
    pub theme: Option<String>,

    /// Set the syntax highlighting theme for code blocks
    #[arg(short = 'T', long = "code-theme", default_value = "terminal", help_heading = "Themes and code", display_order = 24)]
    pub code_theme: Option<String>,

    /// Show empty Markdown elements, including code blocks and list items
    #[arg(long = "show-empty-elements", help_heading = "Output and flow", display_order = 10)]
    pub show_empty_elements: bool,

    /// Disable heuristic language detection for code blocks
    #[arg(long = "no-code-guessing", help_heading = "Themes and code", display_order = 35)]
    pub no_code_guessing: bool,

    /// Load custom .sublime-syntax files from a directory
    #[arg(long = "syntaxes-dir", value_name = "DIR", help_heading = "Themes and code", display_order = 34, long_help = SYNTAXES_DIR_LONG_HELP,)]
    pub syntaxes_dir: Option<PathBuf>,

    /// Set the visual style for code blocks
    #[arg(short = 'b', long = "code-block-style", value_name = "CODE_STYLE", default_value = "basic", value_parser = parse_code_block_style_config, help_heading = "Themes and code", display_order = 29, long_help = CODE_BLOCK_STYLE_LONG_HELP,)]
    pub code_block_style: Option<CodeBlockStyleConfig>,

    /// Set the visual style for display math and fenced math blocks
    #[arg(long = "math-block-style", value_enum, value_name = "MATH_STYLE", default_value = "basic", help_heading = "Themes and code", display_order = 30, long_help = MATH_BLOCK_STYLE_LONG_HELP,)]
    pub math_block_style: Option<MathBlockStyle>,

    /// Show row numbers inside code blocks
    #[arg(short = 'K', long = "code-line-numbers", num_args = 0..=1, value_name = "MODE", value_enum, hide_possible_values = true, help_heading = "Themes and code", display_order = 31, long_help = CODE_LINE_NUMBERS_LONG_HELP,)]
    pub code_line_numbers: Option<Option<LineNumberOptions>>,

    /// Override code block icons, labels, and language aliases
    #[arg(long = "custom-code-block", value_name = "BLOCKS", help_heading = "Themes and code", display_order = 32, long_help = CUSTOM_CODE_BLOCK_LONG_HELP,)]
    pub custom_code_block: Option<String>,

    /// Set the visual style for callouts
    #[arg(short = 'C', long = "callout-style", value_name = "CALLOUT_STYLE", default_value = "pretty", value_parser = parse_callout_style_config, help_heading = "Callouts and lists", display_order = 35, long_help = STYLE_CALLOUT_LONG_HELP,)]
    pub style_callout: Option<CalloutStyleConfig>,

    /// Render task-list checkboxes as Nerd Font icons
    #[arg(short = 'x', long = "checkbox-style", value_enum, value_name = "SHAPE", help_heading = "Callouts and lists", display_order = 37, long_help = PRETTY_CHECKBOX_LONG_HELP,)]
    pub checkbox_style: Option<CheckboxShape>,

    /// Override checkbox icons and colors or add checkbox states
    #[arg(long = "custom-checkbox", value_name = "PAIRS", help_heading = "Callouts and lists", display_order = 38, long_help = CUSTOM_CHECKBOX_LONG_HELP,)]
    pub custom_checkbox: Option<String>,

    /// Render unordered list markers with Nerd Font or Unicode icons
    #[arg(short = 'L', long = "list-style", value_name = "LIST_STYLE", value_parser = PrettyListStyle::parse, help_heading = "Callouts and lists", display_order = 39, long_help = PRETTY_LIST_LONG_HELP,)]
    pub list_style: Option<PrettyListStyle>,

    /// Render definition descriptions with a Unicode or Nerd Font marker
    #[arg(short = 'D', long = "definition-marker-style", value_enum, value_name = "STYLE", help_heading = "Callouts and lists", display_order = 42, long_help = PRETTY_DEFINITION_LONG_HELP,)]
    pub definition_marker_style: Option<PrettyDefinitionStyle>,

    /// Use one list marker for every nesting level
    #[arg(long = "uniform-list-marker", value_name = "MARKER", value_parser = UniformListMarker::parse, help_heading = "Callouts and lists", display_order = 40, long_help = UNIFORM_LIST_MARKER_LONG_HELP,)]
    pub uniform_list_marker: Option<UniformListMarker>,

    /// Override list marker icons and colors per nesting level
    #[arg(long = "custom-list", value_name = "PAIRS", help_heading = "Callouts and lists", display_order = 41, long_help = CUSTOM_LIST_LONG_HELP,)]
    pub custom_list: Option<String>,
    /// Set hanging indent style for wrapped code block lines
    #[arg(long = "code-wrap-indent", value_enum, value_name = "MODE", default_value = "double", help_heading = "Themes and code", display_order = 33)]
    pub code_wrap_indent: Option<CodeWrapIndent>,

    /// Show the current theme and optionally render a file
    #[arg(long = "theme-info", value_name = "FILE", num_args = 0..=1, value_hint = clap::ValueHint::FilePath, help_heading = "Themes and code", display_order = 25)]
    pub theme_info: Option<Option<PathBuf>>,

    /// Set the number of spaces per tab
    #[arg(long = "tab-length", default_value = "4", help_heading = "Layout and wrapping", display_order = 13)]
    pub tab_length: Option<usize>,

    /// Set left and right terminal margins
    #[arg(short = 'm', long = "margin", value_name = "MARGINS", value_parser = parse_horizontal_margins, help_heading = "Layout and wrapping", display_order = 12, long_help = MARGIN_LONG_HELP,)]
    pub margin: Option<HorizontalMargins>,

    /// Set the text and table-cell wrapping mode
    #[arg(short = 'w', long = "wrap", value_enum, value_name = "MODE", default_value = "char", help_heading = "Layout and wrapping", display_order = 14)]
    pub wrap_mode: Option<TextWrapMode>,

    /// Reflow paragraphs by collapsing source newlines and refilling to width
    #[arg(long = "reflow", help_heading = "Layout and wrapping", display_order = 15)]
    pub reflow: bool,

    /// Set table fitting and overflow behavior
    #[arg(short = 'W', long = "table-wrap", value_enum, value_name = "MODE", default_value = "fit", help_heading = "Layout and wrapping", display_order = 19)]
    pub table_wrap_mode: Option<TableWrapMode>,

    /// Render tables with full rounded borders
    #[arg(short = 'B', long = "table-borders", help_heading = "Layout and wrapping", display_order = 20)]
    pub table_borders: bool,

    /// Render from the first occurrence of the given text
    #[arg(long = "from", value_name = "TEXT", help_heading = "Output and flow", display_order = 3)]
    pub from_txt: Option<String>,

    /// Render document starting from the end while preserving layout
    #[arg(short = 'r', long = "reverse", help_heading = "Output and flow", display_order = 4)]
    pub reverse: bool,

    /// Watch the file and reload it when it changes
    #[arg(long = "monitor", help_heading = "Output and flow", display_order = 2)]
    pub monitor_file: bool,

    /// Override colors of the selected theme
    #[arg(long = "custom-theme", value_name = "PAIRS", help_heading = "Themes and code", display_order = 26, long_help = CUSTOM_THEME_LONG_HELP)]
    pub custom_theme: Option<String>,

    /// Override inline Markdown element decorations
    #[arg(long = "inline-style", value_name = "STYLES", help_heading = "Themes and code", display_order = 28, long_help = INLINE_STYLE_LONG_HELP,)]
    pub inline_style: Option<InlineStyleOverrides>,

    /// Override syntax highlighting colors
    #[arg(long = "custom-code-theme", value_name = "PAIRS", help_heading = "Themes and code", display_order = 27, long_help = CUSTOM_CODE_THEME_LONG_HELP)]
    pub custom_code_theme: Option<String>,

    /// Override existing callout styles or add new ones
    #[arg(long = "custom-callout", value_name = "CALLOUTS", help_heading = "Callouts and lists", display_order = 36, long_help = CUSTOM_CALLOUT_LONG_HELP)]
    pub custom_callout: Option<String>,

    /// Set link style
    #[arg(short = 'u', long = "link-style", value_enum, default_value = "clickable", help_heading = "Links and footnotes", display_order = 43)]
    pub link_style: Option<LinkStyle>,

    /// Set link overflow behavior
    #[arg(short = 'l', long = "link-overflow", value_enum, value_name = "MODE", default_value = "wrap", help_heading = "Links and footnotes", display_order = 44)]
    pub link_overflow: Option<LinkTruncationStyle>,

    /// Set where to display footnotes
    #[arg(long = "footnote-style", value_enum, value_name = "STYLE", default_value = "endnotes", help_heading = "Links and footnotes", display_order = 45)]
    pub footnote_style: Option<FootnoteStyle>,

    /// Set how to handle missing footnote definitions
    #[arg(long = "missing-footnote-style", value_enum, value_name = "STYLE", default_value = "show", help_heading = "Links and footnotes", display_order = 46)]
    pub missing_footnote_style: Option<MissingFootnoteStyle>,

    /// Set the directory containing the configuration file
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

    /// Reduce indentation jumps between heading levels
    #[arg(short = 'I', long = "smart-indent", help_heading = "Layout and wrapping", display_order = 18, long_help = SMART_INDENT_LONG_HELP,)]
    pub smart_indent: bool,

    #[arg(short = 'S', long = "table-smart-indent", help = "Adjust table indentation to the available width", help_heading = "Layout and wrapping", display_order = 21, long_help = TABLE_SMART_INDENT_LONG_HELP,)]
    pub table_smart_indent: bool,

    /// Set blank lines above and below block elements
    #[arg(long = "block-spacing", value_name = "SPACING", help_heading = "Layout and wrapping", display_order = 22, long_help = BLOCK_SPACING_LONG_HELP,)]
    pub block_spacing: Option<BlockSpacingOverrides>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrontMatterMode {
    #[value(help = "Hide the front matter")]
    Hidden,
    #[value(help = "Render properties in a callout panel")]
    Panel,
    #[value(help = "Render properties in a two-column table")]
    Table,
    #[value(help = "Render one key/value pair per line")]
    Plain,
    #[value(help = "Render properties in one wrapping paragraph")]
    Inline,
    #[value(help = "Render properties as a definition list")]
    Blocks,
    #[value(help = "Render properties as a YAML code block")]
    Code,
    #[value(help = "Parse the complete source as Markdown without extracting front matter")]
    Source,
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
pub use layout::{HeadingLayout, MathBlockStyle, TableWrapMode, TextWrapMode};
pub use line_numbers::{LineNumberOptions, LineNumberTarget};
pub use links::{FootnoteStyle, LinkStyle, LinkTruncationStyle, MissingFootnoteStyle};
pub use margins::HorizontalMargins;

use callouts::parse_callout_style_config;
use code_blocks::parse_code_block_style_config;
use margins::parse_horizontal_margins;

#[cfg(test)]
mod tests;
