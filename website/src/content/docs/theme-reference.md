---
title: "Theme field reference"
description: "All theme color paths, null rules, inline decorations and pager colors."
slug: "theme-reference"
group: "Customization"
order: 31
---

## Theme fields

Theme files require a nonempty `name`. `description` is optional; `extends` names a built-in theme or an earlier alphabetically loaded user theme. Missing fields inherit from the base; without `extends`, the base is `terminal`.

| Section | Fields | Purpose |
| --- | --- | --- |
| Root | `text`, `text_light`, `h1`...`h6`, `quote`, `link`, `error`, `warning` | Text, muted labels, headings, quotes, links and diagnostics. |
| Root | `background`, `details_border`, `horizontal_rule`, `footnote_separator` | Optional background and independent structural colors. |
| `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight` | `text`, `background` | Inline text and background colors. |
| `line_number` | `number`, `separator` | Numbers and their separator. |
| `table` | `header`, `border` | Headers and table lines, including wrapped-table separators. |
| `math` | `text`, `border` | Math text and frames. |
| `front_matter` | `title`, `key`, `value`, `border` | Properties-panel heading, keys, values and frame. |
| `code_block` | `label`, `border` | Combined title/icon color and code frame. |
| `callout` | `label`, `border`, `palette` | Callout labels/icons, borders and per-type colors. |
| `callout.palette` | `note`, `abstract`, `info`, `todo`, `tip`, `success`, `question`, `warning`, `failure`, `danger`, `bug`, `example`, `quote` | Independent callout type colors. |
| `list` | `ordered`, `unordered` | Ordered and unordered markers. |
| `todo` | `checked`, `unchecked` | Task-list state colors. |
| `syntax` | `keyword`, `string`, `comment`, `number`, `operator`, `function`, `variable`, `type_name` | Syntax highlighting roles. |
| `inline_style` | `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight` | Per-element decoration attributes, documented below. |
| `pager` | `transparent`, `title`, `help`, `file_name`, `progress`, `matches`, `selection`, `search`, `search_current` | Built-in pager palette, documented below. |

## Color formats and CLI paths

Colors accept named values such as `red` and `darkgrey`, hex `#rrggbb` or `#rgb`, RGB `r,g,b`, ANSI `ansi(N)` or an index from 0 to 255, and `reset` for the terminal default.

- Optional overrides also accept `null` or `none`.
- Quote hex values in YAML.
- CLI assignments are separated by `;`; grouped paths use `:`.
- `custom_code_theme` accepts bare syntax keys and `syntax:` paths.

<div data-mdv-preview="custom-theme"></div>

<div data-mdv-preview="custom-code-theme"></div>

```yaml
custom_theme:
  callout:
    palette:
      note: blue
custom_code_theme:
  keyword: "#89b4fa"
```

## Inheritance and null

Omitted values inherit per field.

- In theme files, null optional borders, optional backgrounds, `callout.label` and pager footer foregrounds clear inherited overrides.
- Null `strong_emphasis.text`, `highlight.text`, required foregrounds and `highlight.background` retain base values.
- In color overrides, optional colors can be cleared, while required foregrounds and `highlight.background` reject null.
- Clearing a structural border color leaves the frame visible in terminal text color.
- `reset` emits the terminal default color rather than removing a color assignment.

Null pager selection/search highlight colors restore automatic/default highlights.

- Markdown `highlight.background` colors `==text==` independently of pager search and selection.
- Sources apply in order: selected theme, configuration, preset, CLI.
- Inside each source, syntax colors apply as `custom_theme`, `code_theme`, `custom_code_theme`; higher-priority partial assignments preserve other fields.

## Inline decorations

Every `inline_style` element accepts `backticks`, `bold`, `italic`, `underline` and `strikethrough`.

- Defaults: emphasis is italic, strong is bold, strong_emphasis is both, code has backticks, strikethrough has a strike, and highlight adds no extra decoration beyond its colors.
- Omitted or null attributes retain defaults; explicit false disables an attribute.
- Overrides merge by element and property.

<div data-mdv-preview="inline-style"></div>

## Pager palette

`pager.transparent: true` removes footer/help backgrounds.

- `title`, `help`, `file_name`, `progress` and `matches` color the corresponding footer/help roles.
- `selection`, `search` and `search_current` each accept `text` and `background`.
- Null highlight fields restore the automatic/default highlight.
- `--color-depth` and `--color never` apply to this palette too.

```yaml
pager:
  transparent: true
  title: blue
  search_current:
    text: black
    background: yellow
```

The former root `border`, `details`, flattened names such as `code_background` and `math_border`, and `pager_status_bar_transparent` are rejected. Use the fields above.
