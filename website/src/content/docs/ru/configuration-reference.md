---
title: "Справочник YAML"
description: "Каждый ключ конфигурации, его значение по умолчанию и полный шаблон."
slug: "configuration-reference"
group: "Reference"
order: 41
---

## Все ключи конфигурации

Таблица содержит каждый ключ стандартной конфигурации. Управляющие параметры CLI, например `--monitor`, `--pager`, `--interactive`, `--html`, `--preset`, `--init-config`, не являются ключами файла. В YAML используется snake_case.

| Настройка | По умолчанию | Назначение |
| --- | --- | --- |
| `theme` | `"terminal"` | выбор встроенной или пользовательской темы (по умолчанию `terminal`). |
| `code_theme` | `null` | отдельная палитра подсветки кода. Без явного выбора код использует палитру темы документа. |
| `syntaxes_dir` | `null` | рекурсивная загрузка пользовательских `.sublime-syntax` поверх встроенного набора. Пользовательские определения имеют приоритет при совпадении со встроенными. |
| `theme_info` | `false` | Булево значение, включающее сведения о палитре вместе с выводом; имя файла передаётся позиционным аргументом CLI. |
| `custom_theme` | `null` | переопределение цветов интерфейса поверх выбранной темы. |
| `inline_style` | `{}` | переопределение оформления `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough` и `highlight`. Записи формата `element:property=true,property=false` разделяются `;`; доступны свойства `backticks`, `bold`, `italic`, `underline` и `strikethrough` (например, `--inline-style 'code:backticks=false,bold=true;highlight:underline=true'`). |
| `custom_code_theme` | `null` | переопределение цветов подсветки кода в том же формате, что и `--custom-theme`. |
| `custom_callout` | `null` | переопределение или добавление подписей callout. |
| `custom_code_block` | `null` | переопределение иконки, подписи и псевдонимов для конкретных языков блоков кода. Несколько языков разделяются `;`, опции внутри одного языка - `,`, а псевдонимы - `\|` (например, `python:icon=*,label=Python,aliases=py\|py3;rust:icon=R`). Работает для любого указанного языка, даже если его нет во встроенной карте иконок. Подсветка синтаксиса применяется только к языкам, которые поддерживает текущая логика подсветки mdv. Используйте `default:icon=...`, чтобы задать fallback-иконку для неизвестных языков. |
| `cols` | `null` | Ширина при отсутствии определённой ширины терминала; явный CLI `--cols` важнее определения терминала. |
| `margin` | `null` | задаёт левый и/или правый отступ. Неуказанная сторона равна `0`. |
| `tab_length` | `4` | замена символов табуляции на `N` пробелов (по умолчанию `4`). |
| `color` | `auto` | управление терминальным оформлением. `auto` включает ANSI и OSC 8 только при выводе в терминал, `always` сохраняет оформление в pipe, а `never` выводит отрендерированный текст без оформления. |
| `color_depth` | `auto` | автоматическое определение палитры терминала или её явное ограничение. Действует на цвета документа и интерактивного интерфейса; по умолчанию `auto`. |
| `wrap` | `"char"` | стратегия переноса текста, кода, callout-блоков и содержимого ячеек таблиц (по умолчанию `char`). |
| `reflow` | `false` | склеивает внутриабзацные мягкие переносы строк (soft breaks) и заново заполняет каждую строку до ширины переноса, так что жёстко разбитый исходник перетекает под узкие экраны. Требует включённого переноса; жёсткие переносы сохраняются. |
| `table_wrap` | `"fit"` | геометрия таблиц: подогнать ячейки, разделить колонки на блоки или разрешить переполнение (по умолчанию `fit`). Точки переноса в ячейках определяет `--wrap`. |
| `table_borders` | `false` | возвращает полные скруглённые рамки у таблиц. |
| `heading_layout` | `"level"` | схема выравнивания заголовков (по умолчанию `level`). |
| `show_heading_markers` | `false` | добавляет перед заголовками markdown-маркеры, соответствующие их уровню. |
| `smart_indent` | `false` | сглаживание скачков отступов между уровнями заголовков в режиме `level`. |
| `table_smart_indent` | `false` | автоматический подбор отступа таблицы в зависимости от доступной ширины. |
| `block_spacing` | `null` | переопределяет количество пустых строк сверху и снизу для отдельных блоков. Неуказанные стороны сохраняют стандартные значения, а отступы соседних блоков схлопываются до большего значения. |
| `horizontal_rule_style` | `"pretty"` | рисует разделители Markdown и HTML `<hr>` с декоративными символами по краям или сплошной линией. |
| `hide_comments` | `false` | скрытие Markdown-комментариев в итоговом выводе. |
| `front_matter` | `"hidden"` | управление YAML-словарём свойств с точными разделителями `---`, начинающимся с первой строки. `hidden` скрывает его; `panel` показывает панель свойств; `table` выводит колонки «Property» и «Value»; `plain` - нормализованные строки «ключ: значение»; `inline` объединяет свойства в одну переносимую строку; `blocks` помещает значение под ключом; `code` выводит YAML с подсветкой; `source` разбирает весь ввод как обычный Markdown. |
| `render_html` | `false` | рендер встроенных HTML-фрагментов как форматированного терминального содержимого вместо буквального показа тегов. Вложенность HTML ограничена 64 DOM-элементами. |
| `line_numbers` | `false` | добавление номеров строк в терминальный вывод и pager. Без режима нумеруются отрендеренные строки; `source` использует физические строки Markdown, `separator` добавляет разделитель, а `"source;separator"` сочетает оба режима. |
| `code_line_numbers` | `false` | нумерация строк внутри каждого блока кода. Без режима нумеруется каждая перенесённая терминальная строка; `source` нумерует физические строки кода и оставляет продолжения без номера, `separator` добавляет разделитель, а `"source;separator"` сочетает оба режима. |
| `show_empty_elements` | `false` | отображение обычно скрытых пустых заголовков, списков, цитат, блоков кода и таблиц. |
| `code_guessing` | `true` | Эвристическое определение языка; `false` соответствует `--no-code-guessing`. |
| `code_block_style` | `"basic"` | стиль оформления блоков кода: безрамочный блок с отступом, одинарная граница или полная рамка. По умолчанию подпись скрыта; `show-name` показывает название языка, `show-icon` - его иконку, обе опции можно комбинировать (по умолчанию `basic`). |
| `math_block_style` | `"basic"` | стиль display-формул и fenced-блоков `math`/`latex`/`tex`. |
| `callout_style` | `"pretty"` | стиль callout-блоков и правила отображения подписи. `show-icons` использует глифы Nerd Font, а `show-simple-icons` - переносимые ASCII-маркеры вроде `[i]`, `[*]`, `[!]` и `[x]`. Эти два параметра нельзя комбинировать. |
| `code_wrap_indent` | `"double"` | управляет «висячим» отступом при переносе строк внутри блоков кода (по умолчанию `double`). |
| `reverse` | `false` | рендер документа с конца, сохраняя форматирование блоков. |
| `checkbox_style` | `null` | рендерит чекбоксы task-list как иконки Nerd Font вместо стандартных `[ ]` / `[x]`. Требует шрифт Nerd Font в терминале. |
| `custom_checkbox` | `null` | переопределяет встроенные иконки чекбоксов или добавляет новые состояния (только с `--checkbox-style`). Каждая разделённая точкой с запятой пара задаётся как `<char>:<icon>[:<color>]` или `<char>:<color>`. Форма только с цветом сохраняет существующую иконку, а для нового состояния использует стандартную иконку неотмеченного чекбокса. Цвета принимают именованные, hex, rgb и `ansi(N)`. |
| `list_style` | `null` | заменяет стандартные маркеры `-` неупорядоченных списков на набор иконок по уровням. Значение обязательно и задаётся в формате `type:<nerd-font\|unicode>;size:<large\|small>`, например `--list-style 'type:unicode;size:small'`. |
| `definition_marker_style` | `null` | выводит описания definition list со встроенным Unicode- или Nerd Font-маркером. |
| `uniform_list_marker` | `null` | использует один маркер на всех уровнях вложенности (только с `--list-style`). `level:2` берёт второй маркер выбранного набора, а `icon:*` задаёт собственный глиф. |
| `custom_list` | `null` | переопределяет иконку и/или цвет маркера для конкретных уровней вложенности (только с `--list-style`). Уровень - 1-based; иконка - глиф маркера. Цвета принимают именованные (`red`), hex (`#ff0000`), rgb (`255,0,0`) и `ansi(N)`. |
| `link_style` | `"clickable"` | способ отображения ссылок (по умолчанию `clickable`). |
| `link_overflow` | `"wrap"` | определяет, переносить, обрезать или разрешать переполнение длинных ссылок (по умолчанию `wrap`). |
| `footnote_style` | `"endnotes"` | размещение сносок в конце документа или сразу после блока со ссылкой, включая абзацы, таблицы и элементы списков. |
| `missing_footnote_style` | `"show"` | поведение для отсутствующих, некорректных или пустых определений сносок. |
| `from_text` | `null` | рендер начиная с первой строки, содержащей `<TEXT>`. Добавление `:<lines>` ограничит число строк (например, `--from "Install:20"`). |

## Полный шаблон с комментариями

Это полный шаблон конфигурации из поставки mdv. Перенесите нужные настройки в `config.yaml`; комментарии также перечисляют составные форматы и поля темы.

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
