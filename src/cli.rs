use crate::block_spacing::BlockSpacingOverrides;
use crate::list_marker::{PrettyListStyle, UniformListMarker};
use clap::builder::PossibleValue;
use clap::{Parser, ValueEnum};
use std::fmt;
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
  mdv -E README.md                 # Render embedded HTML in terminal output
  cat README.md | mdv              # Read from stdin
"#
)]
pub struct Cli {
    /// Path to markdown file (use '-' for stdin)
    #[arg(value_name = "FILE")]
    pub filename: Option<String>,

    /// Directory containing the configuration file.
    #[arg(
        short = 'F',
        long = "config-file",
        value_name = "CONFIG_DIR",
        long_help = "Directory containing the configuration file.\nmdv looks for config.yaml or config.yml inside it"
    )]
    pub config_file: Option<PathBuf>,

    /// Skip loading configuration files
    #[arg(short = 'n', long = "no-config")]
    pub no_config: bool,

    /// Apply a named built-in or user preset
    #[arg(short = 'x', long = "preset", value_name = "NAME")]
    pub preset: Option<String>,

    /// List presets, or show the active preset while rendering a file
    #[arg(short = 'X', long = "preset-info")]
    pub preset_info: bool,

    /// Create the default configuration file
    #[arg(short = 'G', long = "init-config", num_args = 0..=1, value_name = "CONFIG_DIR")]
    pub init_config: Option<Option<PathBuf>>,

    /// Strip all ANSI colors
    #[arg(short = 'A', long = "no-colors")]
    pub no_colors: bool,

    /// Hide Markdown comments from the rendered output
    #[arg(short = 'C', long = "hide-comments")]
    pub hide_comments: bool,

    /// Render raw HTML fragments as terminal-formatted content
    #[arg(short = 'E', long = "render-html")]
    pub render_html: bool,

    /// Show rendered row numbers with optional source and separator modes
    #[arg(
        short = 'j',
        long = "line-numbers",
        num_args = 0..=1,
        value_name = "MODE",
        value_enum,
        hide_possible_values = true,
        long_help = "Show row numbers in terminal and pager output\nWithout a value, number every rendered row without a separator\n\nPossible values:\n- source:    Number physical Markdown source lines instead of rendered rows\n- separator: Display a separator after each rendered row number\n\nExamples:\n  --line-numbers separator\n  --line-numbers source\n  --line-numbers \"source;separator\""
    )]
    pub line_numbers: Option<Option<LineNumberOptions>>,

    /// Print HTML version instead of terminal formatting
    #[arg(short = 'H', long = "html")]
    pub do_html: bool,

    /// Show output in the built-in pager instead of printing everything at once
    #[arg(short = 'p', long = "pager")]
    pub pager: bool,

    /// Set theme
    #[arg(short = 't', long = "theme", default_value = "terminal")]
    pub theme: Option<String>,

    /// Theme for code block highlighting
    #[arg(short = 'T', long = "code-theme", default_value = "terminal")]
    pub code_theme: Option<String>,

    /// Display empty Markdown elements such as blank code blocks and list items
    #[arg(short = 'e', long = "show-empty-elements")]
    pub show_empty_elements: bool,

    /// Disable heuristic language detection for code blocks
    #[arg(short = 'g', long = "no-code-guessing")]
    pub no_code_guessing: bool,

    /// Directory containing custom .sublime-syntax files
    #[arg(
        short = 'z',
        long = "syntaxes-dir",
        value_name = "DIR",
        long_help = "Directory containing custom .sublime-syntax files\nFiles are loaded recursively on top of the embedded syntax set\nCustom entries take precedence"
    )]
    pub syntaxes_dir: Option<PathBuf>,

    /// Configure visual style for code blocks
    #[arg(
        short = 's',
        long = "code-block-style",
        value_name = "CODE_STYLE",
        default_value = "basic",
        value_parser = parse_code_block_style_config,
        long_help = "Configure visual style for code blocks\nStyles: basic, simple, pretty\nOptions: show-name, show-icon\nCombine options with ';', for example pretty:show-name;show-icon\nIcons require a Nerd Font in the terminal to display correctly"
    )]
    pub code_block_style: Option<CodeBlockStyleConfig>,

    /// Override code block icon/label/aliases.
    #[arg(
        short = 'J',
        long = "custom-code-block",
        value_name = "BLOCKS",
        long_help = "Override code block icon/label/aliases.\nEntries are separated by ';', options by ',', aliases by '|'.\nAt least one of 'icon' or 'label' is required; 'aliases' is optional.\n\nExample: rust:icon=*,label=russst;py:icon=?,aliases=py|py3"
    )]
    pub custom_code_block: Option<String>,

    #[arg(
        short = 'O',
        long = "callout-style",
        value_name = "CALLOUT_STYLE",
        default_value = "pretty",
        value_parser = parse_callout_style_config,
        long_help = "Configure visual style for callouts\n(pretty:show-icons;label-inside;uppercase;fold-icons\nsimple:show-icons;uppercase;fold-icons)\nOption fold-icons requires show-icons\nIcons require a Nerd Font in the terminal to display correctly"
    )]
    pub style_callout: Option<CalloutStyleConfig>,

    /// Render task-list checkboxes as Nerd Font icons (requires a Nerd Font terminal)
    #[arg(
        short = 'P',
        long = "pretty-checkbox",
        value_enum,
        value_name = "SHAPE",
        long_help = "Render task-list checkboxes as Nerd Font icons\nChoose 'square' or 'circle' icon set\nDisabled by default; requires a Nerd Font to display correctly"
    )]
    pub pretty_checkbox: Option<CheckboxShape>,

    /// Override or add checkbox icons with optional color (e.g. ` :icon:yellow`). Requires --pretty-checkbox
    #[arg(
        short = 'B',
        long = "custom-checkbox",
        value_name = "PAIRS",
        long_help = "Override built-in checkbox icons or add new checkbox states (only with --pretty-checkbox)\n\nFormat: '<char>:<icon>[:<color>];<char>:<icon>[:<color>]'\nIcon is optional: '<char>:<color>' keeps the default icon, just changes the color\n\nOverride:  -B ' :icon'        replaces the unchecked icon\nAdd:       -B '*:icon'        adds a new '[*]' checkbox state\nColor:     -B ' :icon:yellow' or '#ffffff' or '128,1,1' or 'ansi(200)'\nIconless:  -B '?:red'         keeps default [?] icon, applies red color\n           -B '*:yellow'      new '[*]' uses default unchecked icon + yellow"
    )]
    pub custom_checkbox: Option<String>,

    /// Render unordered list markers with Nerd Font or Unicode icons
    #[arg(
        short = 'D',
        long = "pretty-list",
        value_name = "LIST_STYLE",
        value_parser = PrettyListStyle::parse,
        long_help = "Render unordered list markers with a built-in icon set per nesting level\n\nFormat: 'type:<nerd-font|unicode>;size:<large|small>'\n\nThe size option only changes Nerd Font icons.\nIt is accepted for Unicode, but both sizes use the same glyphs.\nUnicode glyph spacing may vary by font.\nRendering was verified with Nerd Font families, especially JetBrainsMono Nerd Font.\n\nExamples:\n  --pretty-list 'type:nerd-font;size:large'\n  --pretty-list 'type:nerd-font;size:small'\n  --pretty-list 'type:unicode;size:large'\n  --pretty-list 'size:large'\n  --pretty-list 'type:unicode'"
    )]
    pub pretty_list: Option<PrettyListStyle>,

    /// Render definition descriptions with a Unicode or Nerd Font marker
    #[arg(
        short = 'v',
        long = "pretty-definition",
        value_enum,
        value_name = "STYLE",
        long_help = "Render definition descriptions with a built-in marker\n\nUnicode definition marker spacing may vary by font.\nNerd Font definition marker requires a Nerd Font terminal.\nRendering was verified with Nerd Font families, especially JetBrainsMono Nerd Font."
    )]
    pub pretty_definition: Option<PrettyDefinitionStyle>,

    /// Use one list marker for every nesting level. Requires --pretty-list
    #[arg(
        short = 'N',
        long = "uniform-list-marker",
        value_name = "MARKER",
        value_parser = UniformListMarker::parse,
        long_help = "Use one marker for every unordered-list nesting level (only with --pretty-list)\n\nChoose exactly one form:\n  level:<1-4>  reuse that level's icon from the selected --pretty-list set\n  icon:<glyph> use a custom glyph or string\n\nExamples:\n  --uniform-list-marker 'level:2'\n  --uniform-list-marker 'icon:*'"
    )]
    pub uniform_list_marker: Option<UniformListMarker>,

    /// Override list marker icon and/or color per nesting level. Requires --pretty-list
    #[arg(
        short = 'Q',
        long = "custom-list",
        value_name = "PAIRS",
        long_help = "Override list marker icon and/or color per nesting level (only with --pretty-list)\n\nFormat: '<level>:<icon>[:<color>];<level>:<color>'\nLevel is 1-based nesting depth; icon is the marker glyph\n\nIcon + color:  --custom-list '1:*:yellow'   marker '*' in yellow\nIcon only:     --custom-list '1:>'          marker '>' in theme color\nColor only:    --custom-list '1:red'        keep built-in icon, red color\n\nColors accept: named ('red', 'blue'), hex ('#ff0000'), rgb ('255,0,0'), ansi ('ansi(200)')"
    )]
    pub custom_list: Option<String>,
    /// Set hanging indent style for wrapped code block lines
    #[arg(
        short = 'K',
        long = "code-wrap-indent",
        value_enum,
        value_name = "MODE",
        default_value = "double"
    )]
    pub code_wrap_indent: Option<CodeWrapIndent>,

    /// Show current theme and optionally display the contents of FILE when provided
    #[arg(short = 'i', long = "theme-info", value_name = "FILE", num_args = 0..=1, value_hint = clap::ValueHint::FilePath)]
    pub theme_info: Option<Option<PathBuf>>,

    /// Set tab length
    #[arg(short = 'b', long = "tab-length", default_value = "4")]
    pub tab_length: Option<usize>,

    /// Fix columns to this width
    #[arg(short = 'c', long = "cols")]
    pub cols: Option<usize>,

    /// Set left and right terminal margins
    #[arg(
        short = 'a',
        long = "margin",
        value_name = "MARGINS",
        value_parser = parse_horizontal_margins,
        long_help = "Set horizontal margins around terminal output\nFormat: 'left:<columns>;right:<columns>'\nSpecify either side or both; an omitted side defaults to 0\n\nExamples:\n  --margin 'left:4'\n  --margin 'right:5'\n  --margin 'left:4;right:5'"
    )]
    pub margin: Option<HorizontalMargins>,

    /// Configure text wrapping mode
    #[arg(
        short = 'W',
        long = "wrap",
        value_enum,
        value_name = "MODE",
        default_value = "char"
    )]
    pub wrap_mode: Option<TextWrapMode>,

    /// Reflow paragraphs by collapsing source newlines and refilling to width
    #[arg(short = 'R', long = "reflow")]
    pub reflow: bool,

    /// Configure table wrapping behavior
    #[arg(
        short = 'w',
        long = "table-wrap",
        value_enum,
        value_name = "MODE",
        default_value = "fit"
    )]
    pub table_wrap_mode: Option<TableWrapMode>,

    /// Render tables with full rounded borders
    #[arg(short = 'q', long = "pretty-table")]
    pub pretty_table: bool,

    /// Display from given substring of the file
    #[arg(short = 'f', long = "from", value_name = "TEXT")]
    pub from_txt: Option<String>,

    /// Render document starting from the end while preserving layout
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    /// Monitor file for changes and redisplay
    #[arg(short = 'm', long = "monitor")]
    pub monitor_file: bool,

    /// Override colors of the selected theme (e.g. `text=#ffffff;h1=187,154,247`)
    #[arg(short = 'y', long = "custom-theme", value_name = "PAIRS")]
    pub custom_theme: Option<String>,

    /// Override syntax highlighting colors (e.g. `keyword=#ffffff;string=128,0,128`)
    #[arg(short = 'Y', long = "custom-code-theme", value_name = "PAIRS")]
    pub custom_code_theme: Option<String>,

    /// Override or create callout styles (e.g. tip:icon=*,color=red;custom:color=#ffffff)
    #[arg(short = 'U', long = "custom-callout", value_name = "CALLOUTS")]
    pub custom_callout: Option<String>,

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

    /// Configure footnote rendering style
    #[arg(
        short = 'o',
        long = "footnote-style",
        value_enum,
        value_name = "STYLE",
        default_value = "endnotes"
    )]
    pub footnote_style: Option<FootnoteStyle>,

    /// Configure handling of missing footnote definitions
    #[arg(
        short = 'M',
        long = "missing-footnote-style",
        value_enum,
        value_name = "STYLE",
        default_value = "show"
    )]
    pub missing_footnote_style: Option<MissingFootnoteStyle>,

    /// Set heading layout
    #[arg(
        short = 'd',
        long = "heading-layout",
        value_enum,
        default_value = "level"
    )]
    pub heading_layout: Option<HeadingLayout>,

    /// Show Markdown-style markers before headings
    #[arg(short = 'k', long = "show-heading-markers")]
    pub show_heading_markers: bool,

    #[arg(
        short = 'I',
        long = "smart-indent",
        long_help = "Smart indentation for headings when using `--heading-layout level`\ncompress large jumps between heading levels so consecutive headings \nchange indentation gradually (e.g. H1 → H4 indents like H2)"
    )]
    pub smart_indent: bool,

    #[arg(
        short = 'S',
        long = "table-smart-indent",
        help = "Automatically adjusts table indentation based on available width",
        long_help = "Automatically adjusts table indentation based on available width\nUses heading content indentation when space allows and reduces it when width is tight"
    )]
    pub table_smart_indent: bool,

    /// Configure blank lines around block elements
    #[arg(
        long = "block-spacing",
        value_name = "SPACING",
        long_help = "Configure blank lines above and below individual block elements\nEntries are separated by ';', sides by ','\nOmitted elements and sides keep their default spacing\nElements: paragraph, h1..h6, code-block, display-math, table, horizontal-rule\nunordered-list, ordered-list, task-list, blockquote, callout, definition-list\ninline-references, end-references, attached-footnotes, endnotes\n\nExample: --block-spacing 'paragraph:top=0,bottom=1;callout:top=1'"
    )]
    pub block_spacing: Option<BlockSpacingOverrides>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HorizontalMargins {
    pub left: usize,
    pub right: usize,
}

impl HorizontalMargins {
    fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Horizontal margins cannot be empty.".to_string());
        }

        let mut margins = Self::default();
        let mut has_left = false;
        let mut has_right = false;

        for pair in input.split(';') {
            let pair = pair.trim();
            let Some((side, columns)) = pair.split_once(':') else {
                return Err(format!(
                    "Invalid horizontal margin '{}'. Expected '<side>:<columns>'.",
                    pair
                ));
            };
            let side = side.trim().to_ascii_lowercase();
            let columns = columns.trim().parse::<usize>().map_err(|_| {
                format!(
                    "Invalid horizontal margin value '{}'. Expected a non-negative integer.",
                    columns.trim()
                )
            })?;

            match side.as_str() {
                "left" if !has_left => {
                    margins.left = columns;
                    has_left = true;
                }
                "right" if !has_right => {
                    margins.right = columns;
                    has_right = true;
                }
                "left" | "right" => {
                    return Err(format!(
                        "Horizontal margin '{}' is defined more than once.",
                        side
                    ));
                }
                _ => {
                    return Err(format!(
                        "Unknown horizontal margin '{}'. Expected 'left' or 'right'.",
                        side
                    ));
                }
            }
        }

        Ok(margins)
    }

    pub fn total(self) -> usize {
        self.left.saturating_add(self.right)
    }
}

impl fmt::Display for HorizontalMargins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "left:{};right:{}", self.left, self.right)
    }
}

impl serde::Serialize for HorizontalMargins {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if *self == Self::default() {
            serializer.serialize_none()
        } else {
            serializer.serialize_str(&self.to_string())
        }
    }
}

impl<'de> serde::Deserialize<'de> for HorizontalMargins {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <Option<String> as serde::Deserialize>::deserialize(deserializer)?;
        match value {
            Some(value) => Self::parse(&value).map_err(serde::de::Error::custom),
            None => Ok(Self::default()),
        }
    }
}

impl TryFrom<String> for HorizontalMargins {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<HorizontalMargins> for String {
    fn from(value: HorizontalMargins) -> Self {
        value.to_string()
    }
}

fn parse_horizontal_margins(value: &str) -> Result<HorizontalMargins, String> {
    HorizontalMargins::parse(value)
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkStyle {
    /// [alias:  c] Link text becomes clickable without showing URL
    #[value(alias = "c")]
    #[serde(alias = "c")]
    Clickable,
    /// [alias: fc] Clickable links with forced underline
    #[value(name = "fclickable", alias = "fc")]
    #[serde(alias = "fclickable", alias = "fc")]
    ClickableForced,
    /// [alias:  i] Link URL after link name
    #[value(alias = "i")]
    #[serde(alias = "i")]
    Inline,
    /// [alias: it] Index after link name and link URL table after text
    #[value(name = "inlinetable", alias = "it")]
    #[serde(alias = "inlinetable", alias = "it")]
    InlineTable,
    /// [alias: et] Index after link name and link URL table at document end
    #[value(name = "endtable", alias = "et")]
    #[serde(alias = "endtable", alias = "et")]
    EndTable,
    /// [alias:  h] Hide link URLs
    #[value(alias = "h")]
    #[serde(alias = "h")]
    Hide,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkTruncationStyle {
    /// Wrap links when they don't fit
    Wrap,
    /// Cut links and replace with "..." when they don't fit
    Cut,
    /// Cut links in normal flow and inside table cells
    #[value(name = "tablecut")]
    #[serde(rename = "tablecut")]
    TableCut,
    /// No truncation - links overflow horizontally
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FootnoteStyle {
    #[value(help = "Collect all footnotes at the end of the document")]
    Endnotes,
    #[value(help = "Render footnotes immediately after the block that references them")]
    Attached,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingFootnoteStyle {
    #[value(help = "Render missing footnotes with a placeholder entry")]
    Show,
    #[value(help = "Omit missing footnotes from the footnote block")]
    Hide,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineNumberTarget {
    #[default]
    Rendered,
    Source,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LineNumberOptions {
    pub target: LineNumberTarget,
    pub separator: bool,
}

impl LineNumberOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        Self::from_str(value, false)
    }
}

impl ValueEnum for LineNumberOptions {
    fn value_variants<'a>() -> &'a [Self] {
        const VARIANTS: &[LineNumberOptions] = &[
            LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: false,
            },
            LineNumberOptions {
                target: LineNumberTarget::Rendered,
                separator: true,
            },
            LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: true,
            },
        ];
        VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match (self.target, self.separator) {
            (LineNumberTarget::Rendered, false) => None,
            (LineNumberTarget::Source, false) => Some(
                PossibleValue::new("source")
                    .help("Number physical Markdown source lines instead of rendered rows"),
            ),
            (LineNumberTarget::Rendered, true) => Some(
                PossibleValue::new("separator")
                    .help("Display a separator after each rendered row number"),
            ),
            (LineNumberTarget::Source, true) => Some(
                PossibleValue::new("source;separator")
                    .alias("separator;source")
                    .help("Number physical Markdown source lines and add the ` │ ` separator"),
            ),
        }
    }
}

impl fmt::Display for LineNumberOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.target, self.separator) {
            (LineNumberTarget::Rendered, false) => Ok(()),
            (LineNumberTarget::Rendered, true) => f.write_str("separator"),
            (LineNumberTarget::Source, false) => f.write_str("source"),
            (LineNumberTarget::Source, true) => f.write_str("source;separator"),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeBlockStyle {
    #[value(help = "Indented code block without a border")]
    Basic,
    #[value(help = "Classic terminal gutter with single left border")]
    Simple,
    #[value(help = "Box-drawn frame around code blocks")]
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CodeBlockStyleConfig {
    pub style: CodeBlockStyle,
    pub show_name: bool,
    pub show_icon: bool,
}

impl Default for CodeBlockStyleConfig {
    fn default() -> Self {
        Self {
            style: CodeBlockStyle::Basic,
            show_name: false,
            show_icon: false,
        }
    }
}

impl CodeBlockStyleConfig {
    fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Code block style cannot be empty.".to_string());
        }

        let (style_raw, options_raw) = match input.split_once(':') {
            Some((style, options)) => (style.trim(), Some(options.trim())),
            None => (input, None),
        };

        let style = match style_raw.to_ascii_lowercase().as_str() {
            "basic" => CodeBlockStyle::Basic,
            "simple" => CodeBlockStyle::Simple,
            "pretty" => CodeBlockStyle::Pretty,
            _ => {
                return Err(format!(
                    "Unknown code block style '{}'. Expected 'basic', 'simple', or 'pretty'.",
                    style_raw
                ));
            }
        };

        let mut config = Self {
            style,
            ..Self::default()
        };

        if let Some(options_raw) = options_raw {
            if options_raw.is_empty() {
                return Err("Code block style options cannot be empty.".to_string());
            }

            for option in options_raw.split(';') {
                let option = option.trim();
                if option.is_empty() {
                    return Err("Code block style option cannot be empty.".to_string());
                }

                match option.to_ascii_lowercase().as_str() {
                    "show-name" => config.show_name = true,
                    "show-icon" => config.show_icon = true,
                    _ => return Err(format!("Unknown code block style option '{}'.", option)),
                }
            }
        }

        Ok(config)
    }
}

impl fmt::Display for CodeBlockStyleConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = match self.style {
            CodeBlockStyle::Basic => "basic",
            CodeBlockStyle::Simple => "simple",
            CodeBlockStyle::Pretty => "pretty",
        };

        let mut options = Vec::new();
        if self.show_name {
            options.push("show-name");
        }
        if self.show_icon {
            options.push("show-icon");
        }

        if options.is_empty() {
            write!(f, "{}", style)
        } else {
            write!(f, "{}:{}", style, options.join(";"))
        }
    }
}

impl TryFrom<String> for CodeBlockStyleConfig {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CodeBlockStyleConfig::parse(&value)
    }
}

impl From<CodeBlockStyleConfig> for String {
    fn from(value: CodeBlockStyleConfig) -> Self {
        value.to_string()
    }
}

fn parse_code_block_style_config(value: &str) -> Result<CodeBlockStyleConfig, String> {
    CodeBlockStyleConfig::parse(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalloutStyle {
    #[value(help = "Callout label with the quote gutter")]
    Simple,
    #[value(help = "Box-drawn frame with callout label on top")]
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckboxShape {
    #[value(help = "Square Nerd Font icons for task-list checkboxes")]
    Square,
    #[value(help = "Circular Nerd Font icons for task-list checkboxes")]
    Circle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrettyDefinitionStyle {
    #[value(help = "Unicode heavy arrow")]
    Unicode,
    #[value(help = "Nerd Font icon U+F0315")]
    NerdFont,
}

impl PrettyDefinitionStyle {
    pub(crate) fn marker(self) -> &'static str {
        match self {
            Self::Unicode => "🠶 ",
            Self::NerdFont => "\u{f0315} ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CalloutStyleConfig {
    pub style: CalloutStyle,
    pub show_icons: bool,
    pub show_fold_icons: bool,
    pub label_inside: bool,
    pub uppercase: bool,
}

impl Default for CalloutStyleConfig {
    fn default() -> Self {
        Self {
            style: CalloutStyle::Pretty,
            show_icons: false,
            show_fold_icons: false,
            label_inside: false,
            uppercase: false,
        }
    }
}

impl CalloutStyleConfig {
    fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Callout style cannot be empty.".to_string());
        }

        let (style_raw, options_raw) = match input.split_once(':') {
            Some((style, options)) => (style.trim(), Some(options.trim())),
            None => (input, None),
        };

        let style = match style_raw.to_ascii_lowercase().as_str() {
            "simple" => CalloutStyle::Simple,
            "pretty" => CalloutStyle::Pretty,
            _ => {
                return Err(format!(
                    "Unknown callout style '{}'. Expected 'simple' or 'pretty'.",
                    style_raw
                ));
            }
        };

        let mut config = CalloutStyleConfig {
            style,
            ..CalloutStyleConfig::default()
        };

        if let Some(options_raw) = options_raw {
            if options_raw.is_empty() {
                return Err("Callout style options cannot be empty.".to_string());
            }

            for option in options_raw.split(';') {
                let option = option.trim();
                if option.is_empty() {
                    return Err("Callout style option cannot be empty.".to_string());
                }

                match option.to_ascii_lowercase().as_str() {
                    "show-icons" => config.show_icons = true,
                    "fold-icons" => config.show_fold_icons = true,
                    "label-inside" => config.label_inside = true,
                    "uppercase" => config.uppercase = true,
                    _ => return Err(format!("Unknown callout style option '{}'.", option)),
                }
            }
        }

        if matches!(config.style, CalloutStyle::Simple) && config.label_inside {
            return Err(
                "Option 'label-inside' is only supported with 'pretty' callout style.".to_string(),
            );
        }

        Ok(config)
    }
}

impl fmt::Display for CalloutStyleConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = match self.style {
            CalloutStyle::Simple => "simple",
            CalloutStyle::Pretty => "pretty",
        };

        let mut options = Vec::new();
        if self.show_icons {
            options.push("show-icons");
        }
        if self.show_fold_icons {
            options.push("fold-icons");
        }
        if self.label_inside {
            options.push("label-inside");
        }
        if self.uppercase {
            options.push("uppercase");
        }

        if options.is_empty() {
            write!(f, "{}", style)
        } else {
            write!(f, "{}:{}", style, options.join(";"))
        }
    }
}

impl TryFrom<String> for CalloutStyleConfig {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CalloutStyleConfig::parse(&value)
    }
}

impl From<CalloutStyleConfig> for String {
    fn from(value: CalloutStyleConfig) -> Self {
        value.to_string()
    }
}

fn parse_callout_style_config(value: &str) -> Result<CalloutStyleConfig, String> {
    CalloutStyleConfig::parse(value)
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeWrapIndent {
    #[value(help = "Do not add extra indentation to wrapped lines")]
    None,
    #[value(help = "Align wrapped lines to the original indentation")]
    Base,
    #[value(help = "Add two extra spaces on top of the original indentation")]
    Double,
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

    fn parse_link_truncation(value: &str) -> LinkTruncationStyle {
        Cli::parse_from(["mdv", "-l", value])
            .link_truncation
            .expect("link truncation parsed")
    }

    #[test]
    fn short_flag_parses_code_wrap_indent() {
        let cli = Cli::parse_from(["mdv", "-K", "base"]);
        assert!(matches!(
            cli.code_wrap_indent.expect("code wrap indent value"),
            CodeWrapIndent::Base
        ));
    }

    #[test]
    fn short_flag_parses_syntaxes_dir() {
        let cli = Cli::parse_from(["mdv", "-z", "syntaxes"]);
        assert_eq!(cli.syntaxes_dir, Some(PathBuf::from("syntaxes")));
    }

    #[test]
    fn short_flag_accepts_long_link_style_names() {
        assert!(matches!(parse_link_style("inline"), LinkStyle::Inline));
        assert!(matches!(
            parse_link_style("inlinetable"),
            LinkStyle::InlineTable
        ));
        assert!(matches!(parse_link_style("endtable"), LinkStyle::EndTable));
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
        assert!(matches!(parse_link_style("et"), LinkStyle::EndTable));
    }

    #[test]
    fn table_smart_indent_flag_parses() {
        let cli = Cli::parse_from(["mdv", "--table-smart-indent"]);
        assert!(cli.table_smart_indent);

        let cli = Cli::parse_from(["mdv", "-S"]);
        assert!(cli.table_smart_indent);
    }

    #[test]
    fn pretty_table_short_alias_parses() {
        let cli = Cli::parse_from(["mdv", "-q"]);
        assert!(cli.pretty_table);
    }

    #[test]
    fn render_html_short_flag_parses() {
        let cli = Cli::parse_from(["mdv", "-E"]);
        assert!(cli.render_html);
    }

    #[test]
    fn line_numbers_flags_parse() {
        let options = Cli::parse_from(["mdv", "--line-numbers", "separator;source", "README.md"])
            .line_numbers
            .flatten()
            .expect("line-number options");
        assert_eq!(options.target, LineNumberTarget::Source);
        assert!(options.separator);

        for invalid in ["unknown", "show-separator", "source;source", "source;"] {
            assert!(Cli::try_parse_from(["mdv", "--line-numbers", invalid]).is_err());
        }
    }

    #[test]
    fn link_truncation_accepts_only_canonical_tablecut() {
        assert!(matches!(
            parse_link_truncation("tablecut"),
            LinkTruncationStyle::TableCut
        ));
        assert!(Cli::try_parse_from(["mdv", "-l", "table-cut"]).is_err());

        assert!(matches!(
            serde_yaml::from_str::<LinkTruncationStyle>("tablecut")
                .expect("canonical tablecut value"),
            LinkTruncationStyle::TableCut
        ));
        assert!(serde_yaml::from_str::<LinkTruncationStyle>("table-cut").is_err());
    }

    #[test]
    fn init_config_flag_parses() {
        let cli = Cli::parse_from(["mdv", "--init-config"]);
        assert!(cli.init_config.is_some());
        assert!(cli.init_config.unwrap().is_none());

        let cli = Cli::parse_from(["mdv", "-G"]);
        assert!(cli.init_config.is_some());
        assert!(cli.init_config.unwrap().is_none());

        let cli = Cli::parse_from(["mdv", "--init-config", "."]);
        assert_eq!(cli.init_config.unwrap().unwrap(), PathBuf::from("."));
    }

    #[test]
    fn pager_flag_parses() {
        let cli = Cli::parse_from(["mdv", "--pager"]);
        assert!(cli.pager);

        let cli = Cli::parse_from(["mdv", "-p"]);
        assert!(cli.pager);
    }

    #[test]
    fn code_block_style_parses_name_and_icon_options() {
        let cli = Cli::parse_from(["mdv", "--code-block-style", "pretty:show-name;show-icon"]);
        let style = cli.code_block_style.expect("code block style parsed");
        assert!(matches!(style.style, CodeBlockStyle::Pretty));
        assert!(style.show_name);
        assert!(style.show_icon);
    }

    #[test]
    fn code_block_style_defaults_to_basic_without_label() {
        let cli = Cli::parse_from(["mdv"]);
        let style = cli.code_block_style.expect("code block style parsed");
        assert!(matches!(style.style, CodeBlockStyle::Basic));
        assert!(!style.show_name);
        assert!(!style.show_icon);
    }

    #[test]
    fn code_block_style_simple_without_label_parses() {
        let cli = Cli::parse_from(["mdv", "--code-block-style", "simple"]);
        let style = cli.code_block_style.expect("code block style parsed");
        assert!(matches!(style.style, CodeBlockStyle::Simple));
        assert!(!style.show_name);
        assert!(!style.show_icon);
    }

    #[test]
    fn code_block_style_rejects_unknown_option() {
        let result = Cli::try_parse_from(["mdv", "--code-block-style", "pretty:bad-option"]);
        assert!(result.is_err());
    }

    #[test]
    fn code_block_style_options_are_independent() {
        let cli = Cli::parse_from(["mdv", "--code-block-style", "basic:show-icon"]);
        let style = cli.code_block_style.expect("code block style parsed");
        assert!(matches!(style.style, CodeBlockStyle::Basic));
        assert!(!style.show_name);
        assert!(style.show_icon);

        let cli = Cli::parse_from(["mdv", "--code-block-style", "basic:show-name"]);
        let style = cli.code_block_style.expect("code block style parsed");
        assert!(style.show_name);
        assert!(!style.show_icon);
    }

    #[test]
    fn code_block_style_rejects_removed_options() {
        for option in ["show-icons", "icon-only"] {
            let value = format!("simple:{}", option);
            let result = Cli::try_parse_from(["mdv", "--code-block-style", &value]);
            assert!(result.is_err(), "option should be rejected: {}", option);
        }
    }

    #[test]
    fn custom_code_block_flag_parses() {
        let cli = Cli::parse_from(["mdv", "--custom-code-block", "rust:icon=;python:icon="]);
        assert_eq!(
            cli.custom_code_block.expect("custom code block parsed"),
            "rust:icon=;python:icon="
        );
    }

    #[test]
    fn custom_code_block_short_alias_parses() {
        let cli = Cli::parse_from(["mdv", "-J", "rust:icon=;python:icon="]);
        assert_eq!(
            cli.custom_code_block.expect("custom code block parsed"),
            "rust:icon=;python:icon="
        );
    }

    #[test]
    fn pretty_list_rejects_legacy_bare_flag() {
        assert!(Cli::try_parse_from(["mdv", "--pretty-list"]).is_err());
        assert!(Cli::try_parse_from(["mdv", "-D"]).is_err());
        assert!(Cli::try_parse_from(["mdv", "--pretty-list", "README.md"]).is_err());
    }

    #[test]
    fn pretty_list_accepts_spaced_style_value() {
        let cli = Cli::parse_from([
            "mdv",
            "--pretty-list",
            "type:nerd-font;size:small",
            "README.md",
        ]);
        let style = cli.pretty_list.expect("pretty list style parsed");

        assert_eq!(style.to_string(), "type:nerd-font;size:small");
        assert_eq!(cli.filename.as_deref(), Some("README.md"));
    }

    #[test]
    fn block_spacing_rejects_invalid_entries() {
        for value in [
            "unknown:top=1",
            "paragraph:left=1",
            "paragraph:top=-1",
            "paragraph:",
            "paragraph:top=1;paragraph:bottom=2",
            "paragraph:top=1,top=2",
        ] {
            assert!(
                Cli::try_parse_from(["mdv", "--block-spacing", value]).is_err(),
                "accepted invalid block spacing: {value}"
            );
        }
    }
}
