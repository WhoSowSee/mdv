---
title: "Layout and wrapping"
description: "Width, margins, headings, tables, block spacing and document line numbers."
slug: "rendering"
group: "Formatting"
order: 20
---

## Width and margins

An explicit `--cols` sets the total terminal width.

- Without it, mdv uses a detected terminal width of at least 20 columns, otherwise configured `cols`, otherwise 80 columns.
- Document margins and line-number gutters reduce the space available to content.
- Their reserved width must be smaller than the total.

<div data-mdv-preview="cols"></div>

<div data-mdv-preview="margin"></div>

<div data-mdv-preview="tab-length"></div>

`--margin` accepts either side or both; an omitted side is zero. `--tab-length` controls tab expansion, with a default of four spaces. Terminal margins do not apply to HTML export.

## Wrapping and paragraph reflow

| Value of `--wrap` | Behavior |
| --- | --- |
| `char` | Wrap at Unicode grapheme boundaries; the default. |
| `word` | Prefer word boundaries; break oversized words at grapheme boundaries. |
| `none` | Avoid soft line wrapping. Constrained table cells may still need to break. |

`--reflow` joins soft source line breaks inside paragraphs before wrapping to the current width. Hard Markdown breaks remain. Reflow requires wrapping to be enabled.

<div data-mdv-preview="wrap"></div>

<div data-mdv-preview="reflow"></div>

## Headings

| Value of `--heading-layout` | Behavior |
| --- | --- |
| `level` | Indent headings by their level and their content one column further; the default. |
| `center` | Center headings without indenting their content. |
| `flat` | Left-align headings and indent their content by one column. |
| `none` | Left-align headings and content. |

`--show-heading-markers` adds `#` markers matching the heading level. With `level`, `--smart-indent` smooths jumps such as H1 followed directly by H4.

<div data-mdv-preview="heading-layout"></div>

<div data-mdv-preview="show-heading-markers"></div>

<div data-mdv-preview="smart-indent"></div>

## Tables

| Value of `--table-wrap` | Behavior |
| --- | --- |
| `fit` | Fit cells to the available width; the default. |
| `wrap` | Split a wide table into successive blocks of columns. |
| `none` | Allow horizontal overflow without imposing cell widths. |

Table geometry is independent of `--wrap`, which chooses cell breakpoints.

- With constrained `fit` or `wrap`, a cell can still break at grapheme boundaries when necessary, even with text wrapping disabled.
- `--table-borders` enables the full rounded grid.
- `--table-smart-indent` reduces indentation when width is tight.

<div data-mdv-preview="table-wrap"></div>

<div data-mdv-preview="table-borders"></div>

<div data-mdv-preview="table-smart-indent"></div>

## Blank lines between blocks

`--block-spacing` accepts `element:top=N,bottom=N` entries separated by `;`. Omitted sides keep their current values. Adjacent gaps collapse to the larger value rather than adding together.

Supported elements: `paragraph`, `h1` through `h6`, `code-block`, `display-math`, `table`, `horizontal-rule`, `unordered-list`, `ordered-list`, `task-list`, `blockquote`, `callout`, `definition-list`, `inline-references`, `end-references`, `attached-footnotes`, and `endnotes`.

<div data-mdv-preview="block-spacing"></div>

YAML uses a mapping, for example `paragraph: {top: 0, bottom: 1}` inside `block_spacing`.

## Rules and hidden content

`--horizontal-rule-style pretty` adds decorative endpoints; `simple` draws a continuous line. It also controls HTML `<hr>` with embedded-HTML rendering enabled.

`--hide-comments` suppresses Markdown comments. `--show-empty-elements` preserves normally hidden empty headings, lists, quotes, code blocks and tables. [Front matter](../front-matter/) has its own display mode.

<div data-mdv-preview="horizontal-rule-style"></div>

<div data-mdv-preview="hide-comments"></div>

<div data-mdv-preview="show-empty-elements"></div>

## Document line numbers

`--line-numbers` alone numbers rendered rows. `source` numbers physical Markdown source lines; `separator` adds a separator; `source;separator` combines both. Put compound values in quotes. A following filename remains positional when the flag has no explicit mode.

<div data-mdv-preview="line-numbers"></div>

Numbering inside [code blocks](../code-blocks/) is configured independently.
