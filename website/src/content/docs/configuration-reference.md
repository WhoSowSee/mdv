---
title: "YAML reference"
description: "Every configuration key, its default and the full annotated template."
slug: "configuration-reference"
group: "Reference"
order: 41
---

## All configuration keys

The table lists every key in the default configuration. CLI-only controls such as `--monitor`, `--pager`, `--interactive`, `--html`, `--preset` and `--init-config` are not configuration-file keys. Use snake_case in YAML.

| Setting | Default | Meaning |
| --- | --- | --- |
| `theme` | `"terminal"` | chooses a built-in or user theme (default `terminal`). |
| `code_theme` | `null` | selects a separate syntax palette. Without an explicit selection, code follows the document theme. |
| `syntaxes_dir` | `null` | recursively loads custom `.sublime-syntax` files on top of the embedded syntax set. Custom entries take precedence over matching built-in syntaxes. |
| `theme_info` | `false` | Boolean enabling palette information alongside rendered output; pass the input file as a positional CLI argument. |
| `custom_theme` | `null` | overrides UI colors on top of the selected theme. |
| `inline_style` | `{}` | overrides decorations for `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, and `highlight`. Use `element:property=true,property=false` entries separated by `;`; properties are `backticks`, `bold`, `italic`, `underline`, and `strikethrough` (for example `--inline-style 'code:backticks=false,bold=true;highlight:underline=true'`). |
| `custom_code_theme` | `null` | overrides syntax colors using the same format as `--custom-theme`. |
| `custom_callout` | `null` | overrides or adds callout labels. |
| `custom_code_block` | `null` | overrides the icon, label, and aliases for specific code block languages. Multiple languages are separated by `;`, options within one language by `,`, and aliases by `\|` (for example, `python:icon=*,label=Python,aliases=py\|py3;rust:icon=R`). Works for any language hint, even if it is not in the built-in icon mapping. Syntax highlighting is applied only when the language is supported by mdv's syntax highlighting logic. Use `default:icon=...` to set the fallback icon for unknown languages. |
| `cols` | `null` | Configured width used when terminal width is not detected; an explicit CLI `--cols` takes priority over detection. |
| `margin` | `null` | reserves separate left and right terminal margins. An omitted side defaults to `0`. |
| `tab_length` | `4` | replaces tab characters with `N` spaces (default `4`). |
| `color` | `auto` | controls terminal styling. `auto` enables ANSI and OSC 8 only when stdout is a terminal, `always` preserves styling through pipes, and `never` emits plain rendered output. |
| `color_depth` | `auto` | detects the terminal palette or sets its limit explicitly. Applies to document colors and the interactive interface; default `auto`. |
| `wrap` | `"char"` | selects the break strategy for text, code, callouts, and table cells (default `char`). |
| `reflow` | `false` | collapses in-paragraph source newlines (soft breaks) and refills each line to the wrap width, so hard-wrapped source reflows to fit smaller screens. Requires wrapping to be enabled; hard breaks are preserved. |
| `table_wrap` | `"fit"` | chooses table geometry: fit cells, split columns into blocks, or allow overflow (default `fit`). Cell breaks still follow `--wrap`. |
| `table_borders` | `false` | restores full rounded table borders. |
| `heading_layout` | `"level"` | controls heading indentation (default `level`). |
| `show_heading_markers` | `false` | prefixes headings with markdown-style markers matching their level. |
| `smart_indent` | `false` | smooths indentation jumps between heading levels in `level` mode. |
| `table_smart_indent` | `false` | automatic table indent adjustment based on available width. |
| `block_spacing` | `null` | overrides top and bottom blank lines per block. Omitted sides keep their defaults, and adjacent block gaps collapse to the larger value. |
| `horizontal_rule_style` | `"pretty"` | draws Markdown rules and HTML `<hr>` with decorative endpoints or as a continuous line. |
| `hide_comments` | `false` | removes Markdown comments from the rendered output. |
| `front_matter` | `"hidden"` | controls an exact `---`-delimited YAML property mapping beginning on the first line. `hidden` omits it; `panel` renders a property panel; `table` uses Property/Value columns; `plain` emits normalized key/value lines; `inline` joins properties into one flowing row; `blocks` places each value below its key; `code` renders highlighted YAML; and `source` parses the complete input as ordinary Markdown. |
| `render_html` | `false` | renders raw HTML fragments as terminal-formatted content instead of displaying their tags literally. HTML nesting is limited to 64 DOM elements. |
| `line_numbers` | `false` | prefixes terminal and pager rows with line numbers. Without a mode, it numbers rendered rows; `source` uses physical Markdown source lines, `separator` adds a separator, and `"source;separator"` combines both. |
| `code_line_numbers` | `false` | numbers rows inside each code block. Without a mode, every wrapped terminal row is numbered; `source` numbers physical code lines and leaves wrapped continuations unnumbered, `separator` adds a separator, and `"source;separator"` combines both modes. |
| `show_empty_elements` | `false` | keeps normally hidden empty headings, lists, block quotes, code blocks, and tables in the output. |
| `code_guessing` | `true` | Enable heuristic language detection; `false` is equivalent to `--no-code-guessing`. |
| `code_block_style` | `"basic"` | selects an indented borderless block, a single gutter, or a boxed frame. Labels are hidden by default; `show-name` displays the language name, `show-icon` displays its icon, and both options may be combined (default `basic`). |
| `math_block_style` | `"basic"` | selects the presentation for display formulas and fenced `math`/`latex`/`tex` blocks. |
| `callout_style` | `"pretty"` | sets the callout layout and label behavior. `show-icons` uses Nerd Font glyphs; `show-simple-icons` uses portable ASCII markers such as `[i]`, `[*]`, `[!]`, and `[x]`. The two icon options cannot be combined. |
| `code_wrap_indent` | `"double"` | sets the hanging indent applied to wrapped code block lines (default `double`). |
| `reverse` | `false` | renders the document starting from the end while keeping block formatting intact. |
| `checkbox_style` | `null` | renders task-list checkboxes as Nerd Font icons instead of the default `[ ]` / `[x]` markers. Requires a Nerd Font in the terminal. |
| `custom_checkbox` | `null` | overrides built-in checkbox icons or adds new states (only with `--checkbox-style`). Each semicolon-separated pair is either `<char>:<icon>[:<color>]` or `<char>:<color>`. The color-only form keeps an existing icon or uses the default unchecked icon for a new state. Colors accept named, hex, rgb, and `ansi(N)` values. |
| `list_style` | `null` | replaces the default `-` unordered-list markers with a level-aware icon set. The value is required and uses the format `type:<nerd-font\|unicode>;size:<large\|small>`, for example `--list-style 'type:unicode;size:small'`. |
| `definition_marker_style` | `null` | renders definition descriptions with a built-in Unicode or Nerd Font marker. |
| `uniform_list_marker` | `null` | uses one marker at every nesting level (only with `--list-style`). `level:2` reuses level 2 from the selected set; `icon:*` uses a custom glyph. |
| `custom_list` | `null` | overrides the marker icon and/or color for specific nesting levels (only with `--list-style`). Level is 1-based; icon is the marker glyph. Colors accept named (`red`), hex (`#ff0000`), rgb (`255,0,0`), and `ansi(N)` values. |
| `link_style` | `"clickable"` | changes how links are displayed (default `clickable`). |
| `link_overflow` | `"wrap"` | controls whether links wrap, are shortened, or overflow the available width (default `wrap`). |
| `footnote_style` | `"endnotes"` | places footnotes at the end of the document or immediately after the block that references them, including paragraphs, tables, and list items. |
| `missing_footnote_style` | `"show"` | controls placeholder entries for missing, invalid, or empty footnote definitions. |
| `from_text` | `null` | starts rendering from the first match of `<TEXT>`. Adding `:<lines>` limits the number of lines (for example `--from "Install:20"`). |

## Complete annotated template

This is the complete configuration template distributed with mdv. Copy the settings you need into `config.yaml`; the comments also enumerate compound formats and theme fields.

```yaml
# Default MDV configuration file
# Create it with: mdv --init-config
# Place this file as ~/.config/mdv/config.yaml (or ~/.config/mdv/config.yml)
# and use --config-file ~/.config/mdv or MDV_CONFIG_PATH=~/.config/mdv to load it.

# Theme settings
# Complex settings accept both YAML mappings and their legacy one-line CLI syntax.
# Theme colors merge per field: selected theme < config < preset < CLI.
# Within one source, syntax order is custom_theme < code_theme < custom_code_theme.
# CLI theme colors override lower-source callout, checkbox, and list colors.
# Optional backgrounds inherit when omitted; explicit null clears them.
# Unspecified colors retain lower-priority values.
theme: "terminal"          # Option: "terminal" | "solarized-dark" | "nord" | "tokyonight" | "kanagawa" | "gruvbox" | "monokai" | "material-ocean" | "catppuccin"
code_theme: null           # Use null to follow theme; accepts the same values as theme
syntaxes_dir: null         # Directory with custom .sublime-syntax files; relative paths start from the config directory
theme_info: false          # Print palette information alongside rendered output
custom_theme: null         # Nested YAML mapping of theme values, or a section:field=value string
# Example:
# custom_theme:
#   text: "#cdd6f4"
#   h1: "#89b4fa"
#   horizontal_rule: "#6c7086"
#   math:
#     text: blue
#     border: darkcyan
#   code_block:
#     label: darkgrey
#   syntax:
#     keyword: blue
#     number: magenta
#   background: null
inline_style: {}           # Per-element backticks/bold/italic/underline/strikethrough overrides
custom_code_theme: null    # Flat YAML mapping of syntax colors, or a legacy key=value string
# Example:
# custom_code_theme:
#   keyword: "#cba6f7"
#   string: "#a6e3a1"
# Custom callouts, code blocks, checkboxes, and list markers merge by entry and property:
# main config -> preset -> CLI. Omitted entries/properties retain their values.
# Explicit null clears a property or entry; root null or {} clears the whole setting.
# aliases is replaced when specified; aliases: [] clears inherited aliases.
custom_callout: null       # Mapping of callout names to icon/color, or a legacy definition string
# Example:
# custom_callout:
#   tip:
#     icon: "*"
#     color: green
#   important:
#     color: "#ff5555"
custom_code_block: null    # Mapping of language names to icon/label/aliases, or a legacy definition string
# Example:
# custom_code_block:
#   default:
#     icon: "?"
#   rust:
#     icon: "R"
#     label: Rust
#     aliases: [rs, rustlang]
# Works for any language hint, even if it is not in the built-in icon mapping.
# Syntax highlighting is applied only when the language is supported by mdv's syntax highlighting logic.

# Display behaviour
# Keys must use snake_case (e.g. color, tab_length).
cols: null                 # Use detected terminal width; fallback is 80 when detection fails
margin: null              # null disables margins; otherwise use "left:N", "right:N", or both
tab_length: 4              # Replace tab characters with this many spaces
color: auto                # Terminal styling: auto | always | never
color_depth: auto          # Palette limit: auto | 16 | 256 | truecolor
wrap: "char"               # Text and table-cell breaks: "char" | "word" | "none"
reflow: false              # Collapse in-paragraph source newlines and refill lines to width (requires wrapping)
table_wrap: "fit"          # Table geometry: fit cells | wrap columns into blocks | allow overflow
table_borders: false        # Use full rounded borders for tables
heading_layout: "level"    # Options: "level" | "center" | "flat" | "none"
show_heading_markers: false # Prefix headings with markdown-style markers matching their level
smart_indent: false        # Smooth heading indentation jumps (only with "level" layout)
table_smart_indent: false  # Automatically adjusts table indentation based on available width
block_spacing: null        # Mapping of block elements to top/bottom blank lines, or a legacy string
# Example:
# block_spacing:
#   paragraph:
#     top: 0
#     bottom: 1
#   callout:
#     top: 1
horizontal_rule_style: "pretty" # "pretty" adds decorative endpoints; "simple" draws a continuous line
hide_comments: false       # Hide Markdown comments from the rendered output
front_matter: "hidden"     # hidden | panel | table | plain | inline | blocks | code | source
render_html: false         # Render raw HTML fragments as terminal-formatted content
line_numbers: false        # false | true | "source" | "separator" | "source;separator"
code_line_numbers: false   # false | true | "source" | "separator" | "source;separator"; resets for each code block
show_empty_elements: false # Show otherwise hidden empty lists, block quotes, tables, and code blocks
code_guessing: true        # Disable to keep unknown code blocks as plain text
code_block_style: "basic"  # Styles: "basic" | "simple" | "pretty"; options: "show-name" | "show-icon"
                           # Labels are hidden by default; combine options with ";" to show both parts.
math_block_style: "basic"  # Styles: "basic" | "simple" | "pretty"; no labels or icons
callout_style: "pretty"    # Options: "simple:show-icons;show-simple-icons;fold-icons;uppercase" | "pretty:show-icons;show-simple-icons;fold-icons;label-inside;uppercase"
                           # label-inside is supported only with "pretty"; show-icons and show-simple-icons cannot be combined.
                           # fold-icons requires show-icons.
                           # show-simple-icons uses ASCII markers; show-icons and fold-icons require Nerd Fonts.
# Structured alternative:
# callout_style:
#   style: pretty
#   show_icons: true
#   show_simple_icons: false
#   fold_icons: true
#   label_inside: true
#   uppercase: true
code_wrap_indent: "double" # Options: "none" | "base" | "double"; controls hanging indent inside code blocks
reverse: false             # Render document starting from the end while preserving layout
checkbox_style: null      # Options: | "square" | "circle"; render task-list checkboxes as Nerd Font icons
custom_checkbox: null      # Mapping of checkbox states to icon/color, or a legacy string; requires checkbox_style
# Example:
# custom_checkbox:
#   " ":
#     icon: "U"
#     color: yellow
#   x:
#     color: green
list_style: null          # null | "type:nerd-font;size:small" | "type:unicode;size:large"; booleans are not accepted
                           # size only changes Nerd Font icons; Unicode accepts both sizes and uses the same glyphs.
                           # Unicode spacing depends on the font; verified especially with JetBrainsMono Nerd Font.
definition_marker_style: null    # null | "unicode" | "nerd-font"; null renders definitions with a two-space indent
uniform_list_marker: null  # Use one icon at every level: "level:1".."level:4" or a custom "icon:*"
custom_list: null          # Mapping of nesting levels to icon/color, or a legacy string; requires list_style
# Example:
# custom_list:
#   1:
#     icon: "*"
#     color: yellow
#   2:
#     color: red
# Link handling
link_style: "clickable"    # Options: "clickable" | "fclickable" | "inline" | "inlinetable" | "endtable" | "hide" (aliases: c/clickable, fc/fclickable, i/inline, it/inlinetable, et/endtable, h/hide)
link_overflow: "wrap"    # Options: "wrap" | "cut" | "tablecut" | "none"

# Footnotes
footnote_style: "endnotes" # Options: "endnotes" | "attached"
missing_footnote_style: "show" # Options: "show" | "hide"

# Content filtering
from_text: null            # Render starting from the first line that contains this text

# `table_wrap` accepts the values above; `heading_layout` also understands camel-case variants from earlier releases.

# Available root keys for custom_theme:
# text, text_light, h1..h6,
# quote, link, background, details_border, horizontal_rule, footnote_separator,
# error, warning
# Grouped keys use section:field in CLI strings and nested YAML mappings:
# emphasis, strong, strong_emphasis, code, strikethrough, highlight: text, background
# line_number: number, separator
# table: header, border
# math: text, border
# front_matter: title, key, value, border
# code_block: label, border
# callout: label, border, palette
# callout:palette: note, abstract, info, todo, tip, success, question, warning,
#                  failure, danger, bug, example, quote
# list: ordered, unordered
# todo: checked, unchecked
# pager: transparent (true or false), title, help, file_name, progress, matches
# pager:selection, pager:search, pager:search_current: text, background
# Null pager highlight colors preserve the current automatic/default highlighting.
# Example: --custom-theme 'pager:transparent=true;pager:title=blue;pager:search_current:background=darkblue'
# syntax: keyword, string, comment, number, operator, function, variable, type_name
# An omitted user-theme field inherits its base. Null border colors use terminal text.
# Null callout label uses the custom per-type color or its independent palette.
# Example: --custom-theme 'code:text=darkgrey;code:background=null;details_border=grey'
# Inline decorations remain in inline_style, independently of these color groups.

# Available inline_style elements:
# emphasis, strong, strong_emphasis, code, strikethrough, highlight
# Each element accepts: backticks, bold, italic, underline, strikethrough (true or false)
# Example:
# inline_style:
#   code:
#     backticks: false
#     bold: true
#   highlight:
#     underline: true

# Available keys for custom_code_theme:
# keyword, string, comment, number, operator, function, variable, type_name

# Color value formats accepted in custom_theme/custom_code_theme/custom_callout:
# Hex strings: "#rrggbb" or "#rgb"
# RGB tuples:  "r,g,b" or "rgb(r,g,b)" (values 0..255)
# ANSI 256:    "ansi(N)" or just "N" (0..255)
# Named:       "red", "darkgrey", etc.
```
