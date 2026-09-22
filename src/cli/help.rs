pub(super) const LINE_NUMBERS_LONG_HELP: &str = r#"Show row numbers in terminal and pager output

Without a value, number every rendered row without a separator
Combine modes with ';'

Possible values:
- source:    Number physical Markdown source lines instead of rendered rows
- separator: Display a separator after each rendered row number

Examples:
  --line-numbers
  --line-numbers separator
  --line-numbers source
  --line-numbers 'source;separator'"#;

pub(super) const PAGER_LONG_HELP: &str = r#"Show output in a pager instead of printing everything at once

A bare flag uses MDV_PAGER when set and the default pager otherwise
Use --pager=default to select the default pager or --pager=builtin for the built-in pager
Use --pager='<command>' to select an external pager
External commands may include quoted paths and arguments

Examples:
  --pager
  --pager=default
  --pager=builtin
  --pager='less -R'"#;

pub(super) const CODE_LINE_NUMBERS_LONG_HELP: &str = r#"Show row numbers inside code blocks

Without a value, number every wrapped terminal row without a separator
Combine modes with ';'

Possible values:
- source:    Number physical code lines instead of wrapped terminal rows
- separator: Display a separator after each code line number

Examples:
  --code-line-numbers
  --code-line-numbers separator
  --code-line-numbers source
  --code-line-numbers 'source;separator'"#;

pub(super) const SYNTAXES_DIR_LONG_HELP: &str = r#"Load custom .sublime-syntax files from a directory

Files are loaded recursively on top of the embedded syntax set
Custom entries take precedence"#;

pub(super) const CODE_BLOCK_STYLE_LONG_HELP: &str = r#"Set the visual style for code blocks

Format: <style>[:<option>;<option>]

Styles:
  basic   Indent code without a border
  simple  Show a single left border
  pretty  Show a frame around the block

Options:
  show-name  Show the language name
  show-icon  Show the language icon (requires a Nerd Font)

Examples:
  --code-block-style basic
  --code-block-style 'pretty:show-name;show-icon'"#;

pub(super) const MATH_BLOCK_STYLE_LONG_HELP: &str = r#"Set the visual style for display math and fenced math blocks

Math blocks do not use code block labels, icons, or line numbers"#;

pub(super) const CUSTOM_CODE_BLOCK_LONG_HELP: &str = r#"Override code block icons, labels, and language aliases

Format: <language>:icon=<glyph>,label=<text>,aliases=<alias>|<alias>
Separate entries with ';' and properties with ','
Each entry requires an icon or label; aliases are optional

Examples:
  --custom-code-block 'rust:icon=*,label=Rust;python:label=Python,aliases=py|py3'"#;

pub(super) const STYLE_CALLOUT_LONG_HELP: &str = r#"Set the visual style for callouts

Format: <style>[:<option>;<option>]

Styles:
  simple  Show a label with a single left border
  pretty  Show a frame with a label on top

Options:
  show-icons         Show Nerd Font icons
  show-simple-icons  Show portable ASCII markers
  fold-icons         Show fold markers (requires show-icons)
  label-inside       Place the label inside the frame (pretty only)
  uppercase          Convert labels to uppercase

The show-icons and show-simple-icons options cannot be combined
The show-icons and fold-icons options require a Nerd Font

Examples:
  --callout-style simple
  --callout-style 'pretty:show-icons;fold-icons;label-inside'
  --callout-style 'simple:show-simple-icons;uppercase'"#;

pub(super) const PRETTY_CHECKBOX_LONG_HELP: &str = r#"Render task-list checkboxes as Nerd Font icons

Disabled by default; requires a Nerd Font"#;

pub(super) const CUSTOM_CHECKBOX_LONG_HELP: &str = r#"Override checkbox icons and colors or add checkbox states

Format: <char>:<icon>[:<color>];<char>:<color>
Requires --checkbox-style
Omit the icon to keep the default icon and change only its color
New states without an icon use the unchecked icon
Colors: named (yellow), hex (#ffffff), RGB (128,1,1), ANSI (ansi(200))

Examples:
  --custom-checkbox ' :󰀦'         # Replace the unchecked icon
  --custom-checkbox '*:󰞋'         # Add a new [*] checkbox state
  --custom-checkbox ' :󰀦:yellow'  # Set an icon and color
  --custom-checkbox '?:red'       # Keep the [?] icon and change its color
  --custom-checkbox '*:yellow'    # Use the unchecked icon for a new state"#;

pub(super) const PRETTY_LIST_LONG_HELP: &str = r#"Render unordered list markers with Nerd Font or Unicode icons

Format: type:<nerd-font|unicode>;size:<large|small>
Specify either option or both
The size option only changes Nerd Font icons; Unicode uses the same glyphs for both sizes
Nerd Font icons require a Nerd Font; Unicode glyph spacing may vary by font

Examples:
  --list-style 'type:nerd-font;size:large'
  --list-style 'type:nerd-font;size:small'
  --list-style 'type:unicode;size:large'
  --list-style 'size:large'
  --list-style 'type:unicode'"#;

pub(super) const PRETTY_DEFINITION_LONG_HELP: &str = r#"Render definition descriptions with a Unicode or Nerd Font marker

Unicode definition marker spacing may vary by font
The nerd-font marker requires a Nerd Font"#;

pub(super) const UNIFORM_LIST_MARKER_LONG_HELP: &str = r#"Use one list marker for every nesting level

Format: level:<1-4> or icon:<glyph>
Requires --list-style
Choose a level's icon from the selected set or supply a custom glyph or string

Examples:
  --uniform-list-marker 'level:2'
  --uniform-list-marker 'icon:*'"#;

pub(super) const CUSTOM_LIST_LONG_HELP: &str = r#"Override list marker icons and colors per nesting level

Format: <level>:<icon>[:<color>];<level>:<color>
Requires --list-style
Levels start at 1; omit the icon to keep the selected set's icon
Colors: named (red), hex (#ff0000), RGB (255,0,0), ANSI (ansi(200))

Examples:
  --custom-list '1:*:yellow'  # Set an icon and color
  --custom-list '1:>'         # Set an icon and keep the theme color
  --custom-list '1:red'       # Keep the selected icon and change its color"#;

pub(super) const MARGIN_LONG_HELP: &str = r#"Set left and right terminal margins

Format: left:<columns>;right:<columns>
Specify either side or both; an omitted side defaults to 0

Examples:
  --margin 'left:4'
  --margin 'right:5'
  --margin 'left:4;right:5'"#;

pub(super) const INLINE_STYLE_LONG_HELP: &str = r#"Override inline Markdown element decorations

Format: <element>:<property>=<true|false>,<property>=<true|false>
Separate elements with ';' and properties with ','
Elements: emphasis, strong, strong_emphasis, code, strikethrough, highlight
Properties: backticks, bold, italic, underline, strikethrough

Examples:
  --inline-style 'code:backticks=false,bold=true;highlight:underline=true'"#;

pub(super) const CONFIG_FILE_LONG_HELP: &str = r#"Set the directory containing the configuration file

mdv looks for config.yaml or config.yml inside this directory"#;

pub(super) const SMART_INDENT_LONG_HELP: &str = r#"Reduce indentation jumps between heading levels

Applies to --heading-layout level
Consecutive headings change indentation gradually
For example, H1 followed by H4 indents the H4 heading like H2"#;

pub(super) const TABLE_SMART_INDENT_LONG_HELP: &str = r#"Adjust table indentation to the available width

Use heading content indentation when space allows and reduce it when width is tight"#;

pub(super) const BLOCK_SPACING_LONG_HELP: &str = r#"Set blank lines above and below block elements

Format: <element>:top=<lines>,bottom=<lines>
Separate elements with ';' and sides with ','
Omitted elements and sides keep their default spacing

Elements:
  paragraph, h1..h6, code-block, display-math, table, horizontal-rule,
  unordered-list, ordered-list, task-list, blockquote, callout, definition-list,
  inline-references, end-references, attached-footnotes, endnotes

Examples:
  --block-spacing 'paragraph:top=0,bottom=1;callout:top=1'"#;

pub(super) const CUSTOM_THEME_LONG_HELP: &str = r#"Override colors of the selected theme

Format: <key>=<color>;<key>=<color>

Examples:
  --custom-theme 'text=#ffffff;h1=187,154,247'"#;

pub(super) const CUSTOM_CODE_THEME_LONG_HELP: &str = r#"Override syntax highlighting colors

Format: <key>=<color>;<key>=<color>

Examples:
  --custom-code-theme 'keyword=#ffffff;string=128,0,128'"#;

pub(super) const CUSTOM_CALLOUT_LONG_HELP: &str = r#"Override existing callout styles or add new ones

Format: <name>:icon=<glyph>,color=<color>
Separate entries with ';'; specify an icon, a color, or both

Examples:
  --custom-callout 'tip:icon=*,color=red;custom:color=#ffffff'"#;
