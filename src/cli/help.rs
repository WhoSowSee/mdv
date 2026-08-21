pub(super) const LINE_NUMBERS_LONG_HELP: &str = "Show row numbers in terminal and pager output\nWithout a value, number every rendered row without a separator\n\nPossible values:\n- source:    Number physical Markdown source lines instead of rendered rows\n- separator: Display a separator after each rendered row number\n\nExamples:\n  --line-numbers separator\n  --line-numbers source\n  --line-numbers \"source;separator\"";

pub(super) const CODE_LINE_NUMBERS_LONG_HELP: &str = "Number rows inside code blocks\nWithout a value, number every wrapped terminal row without a separator\n\nPossible values:\n- source:    Number physical code lines instead of wrapped terminal rows\n- separator: Display a separator after each code line number\n\nExamples:\n  --code-line-numbers separator\n  --code-line-numbers source\n  --code-line-numbers \"source;separator\"";

pub(super) const SYNTAXES_DIR_LONG_HELP: &str = "Directory containing custom .sublime-syntax files\nFiles are loaded recursively on top of the embedded syntax set\nCustom entries take precedence";

pub(super) const CODE_BLOCK_STYLE_LONG_HELP: &str = "Configure visual style for code blocks\nStyles: basic, simple, pretty\nOptions: show-name, show-icon\nCombine options with ';', for example pretty:show-name;show-icon\nIcons require a Nerd Font in the terminal to display correctly";

pub(super) const CUSTOM_CODE_BLOCK_LONG_HELP: &str = "Override code block icon/label/aliases.\nEntries are separated by ';', options by ',', aliases by '|'.\nAt least one of 'icon' or 'label' is required; 'aliases' is optional.\n\nExample: rust:icon=*,label=russst;py:icon=?,aliases=py|py3";

pub(super) const STYLE_CALLOUT_LONG_HELP: &str = "Configure visual style for callouts\n(pretty:show-icons;label-inside;uppercase;fold-icons\nsimple:show-icons;uppercase;fold-icons)\nOption fold-icons requires show-icons\nIcons require a Nerd Font in the terminal to display correctly";

pub(super) const PRETTY_CHECKBOX_LONG_HELP: &str = "Render task-list checkboxes as Nerd Font icons\nChoose 'square' or 'circle' icon set\nDisabled by default; requires a Nerd Font to display correctly";

pub(super) const CUSTOM_CHECKBOX_LONG_HELP: &str = "Override built-in checkbox icons or add new checkbox states (only with --pretty-checkbox)\n\nFormat: '<char>:<icon>[:<color>];<char>:<icon>[:<color>]'\nIcon is optional: '<char>:<color>' keeps the default icon, just changes the color\n\nOverride:  --custom-checkbox ' :󰀦'         replaces the unchecked icon\nAdd:       --custom-checkbox '*:󰞋'         adds a new '[*]' checkbox state\nColor:     --custom-checkbox ' :󰀦:yellow'  accepts '#ffffff', '128,1,1', 'ansi(200)'\nIconless:  --custom-checkbox '?:red'       keeps the [?] icon and applies red\n           --custom-checkbox '*:yellow'    uses the unchecked icon and applies yellow";

pub(super) const PRETTY_LIST_LONG_HELP: &str = "Render unordered list markers with a built-in icon set per nesting level\n\nFormat: 'type:<nerd-font|unicode>;size:<large|small>'\n\nThe size option only changes Nerd Font icons.\nIt is accepted for Unicode, but both sizes use the same glyphs.\nUnicode glyph spacing may vary by font.\nRendering was verified with Nerd Font families, especially JetBrainsMono Nerd Font.\n\nExamples:\n  --pretty-list 'type:nerd-font;size:large'\n  --pretty-list 'type:nerd-font;size:small'\n  --pretty-list 'type:unicode;size:large'\n  --pretty-list 'size:large'\n  --pretty-list 'type:unicode'";

pub(super) const PRETTY_DEFINITION_LONG_HELP: &str = "Render definition descriptions with a built-in marker\n\nUnicode definition marker spacing may vary by font.\nNerd Font definition marker requires a Nerd Font terminal.\nRendering was verified with Nerd Font families, especially JetBrainsMono Nerd Font.";

pub(super) const UNIFORM_LIST_MARKER_LONG_HELP: &str = "Use one marker for every unordered-list nesting level (only with --pretty-list)\n\nChoose exactly one form:\n  level:<1-4>  reuse that level's icon from the selected --pretty-list set\n  icon:<glyph> use a custom glyph or string\n\nExamples:\n  --uniform-list-marker 'level:2'\n  --uniform-list-marker 'icon:*'";

pub(super) const CUSTOM_LIST_LONG_HELP: &str = "Override list marker icon and/or color per nesting level (only with --pretty-list)\n\nFormat: '<level>:<icon>[:<color>];<level>:<color>'\nLevel is 1-based nesting depth; icon is the marker glyph\n\nIcon + color:  --custom-list '1:*:yellow'   marker '*' in yellow\nIcon only:     --custom-list '1:>'          marker '>' in theme color\nColor only:    --custom-list '1:red'        keep built-in icon, red color\n\nColors accept: named ('red', 'blue'), hex ('#ff0000'), rgb ('255,0,0'), ansi ('ansi(200)')";

pub(super) const MARGIN_LONG_HELP: &str = "Set horizontal margins around terminal output\nFormat: 'left:<columns>;right:<columns>'\nSpecify either side or both; an omitted side defaults to 0\n\nExamples:\n  --margin 'left:4'\n  --margin 'right:5'\n  --margin 'left:4;right:5'";

pub(super) const INLINE_STYLE_LONG_HELP: &str = "Override inline Markdown element decorations\nElements: emphasis, strong, strong_emphasis, code, strikethrough, highlight\nProperties: backticks, bold, italic, underline, strikethrough\n\nFormat: '<element>:<property>=<true|false>,<property>=<true|false>;<element>:...'\nExample: --inline-style 'code:backticks=false,bold=true;highlight:underline=true'";

pub(super) const CONFIG_FILE_LONG_HELP: &str = "Directory containing the configuration file.\nmdv looks for config.yaml or config.yml inside it";

pub(super) const SMART_INDENT_LONG_HELP: &str = "Smart indentation for headings when using `--heading-layout level`\ncompress large jumps between heading levels so consecutive headings \nchange indentation gradually (e.g. H1 → H4 indents like H2)";

pub(super) const TABLE_SMART_INDENT_LONG_HELP: &str = "Automatically adjusts table indentation based on available width\nUses heading content indentation when space allows and reduces it when width is tight";

pub(super) const BLOCK_SPACING_LONG_HELP: &str = "Configure blank lines above and below individual block elements\nEntries are separated by ';', sides by ','\nOmitted elements and sides keep their default spacing\nElements: paragraph, h1..h6, code-block, display-math, table, horizontal-rule\nunordered-list, ordered-list, task-list, blockquote, callout, definition-list\ninline-references, end-references, attached-footnotes, endnotes\n\nExample: --block-spacing 'paragraph:top=0,bottom=1;callout:top=1'";
