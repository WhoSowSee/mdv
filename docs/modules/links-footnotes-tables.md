# Links, Footnotes, and Tables

These subsystems interact closely: links and footnotes may occur inside tables, callouts, and code containers, while reference blocks must finish at the boundary of their owner.

Implementation locations are in the [file index](file-index.md#srcrenderereventlinks-footnotes-and-tables).

## Link styles

| Style | Result |
|---|---|
| `clickable` | Wrap link text in OSC 8. |
| `fclickable` | Clickable presentation with the full URL according to the active rules. |
| `inline` | Render the URL beside its text. |
| `inlinetable` | Place numbered references near the owning block or table. |
| `endtable` | Render one reference table at document end. |
| `hide` | Omit the URL. |

`LinkTruncationStyle` changes only the visible URL representation, including wrapping and ellipsis/table-cut variants. It never changes the OSC 8 destination.

## Links inside tables

`comfy-table` must receive plain visible text; escape bytes would corrupt its width calculations. The pipeline therefore separates layout from styling:

1. The event handler stores plain cell content and `(plain, styled)` replacements.
2. `TableRenderer` calculates layout from plain or ANSI-stripped width.
3. `table::apply_clickable_link_replacements` restores underline and OSC fragments in the rendered output.
4. If `comfy-table` split visible text across rows, `table/links.rs` reconstructs wrappers within the same cell boundary.

Fragment searches advance by whole UTF-8 characters when skipping matches in another column. Zero-width source markers alone cannot establish a link's owning column; matching includes visible link text.

An internal zero-width marker identifies the optional break before `[N]`. Before layout,
`TableRenderer` either reserves enough column width for the link's final word and marker or turns
the marker into an explicit line break when those protected widths cannot fit. No visible-width
content is removed after `comfy-table` has positioned the borders.

## Footnotes

Footnote block separators use the optional `footnote_separator` theme color, independently of Markdown horizontal rules. Without it, they use the terminal text color.

Definitions distinguish normal content, an empty body, and invalid marker syntax. Footnotes
are ordered by first use, with repeated occurrences tracked independently. Missing, invalid,
and empty definitions can produce diagnostic entries according to `MissingFootnoteStyle`.

`FootnoteStyle::Attached` flushes accumulated footnotes at a paragraph or container boundary. `Endnotes` defers them to document end. `MissingFootnoteStyle` controls whether diagnostic entries are visible.

`FootnoteDefinitions` shares immutable definitions through an `Arc` and indexes
each name to its ordered occurrences. Nested renderers borrow the selected body
and share the definition store. An occurrence beyond the defined range retains
the existing last-definition behavior. Mutation detaches a shared store only
when new definitions actually arrive.

## Event-level tables

`TableState` accumulates headers, rows, alignments, the current cell, inline URLs, and replacements. At `TagEnd::Table`, this state becomes parameters for the independent `TableRenderer`.

Math events are rendered before the accumulated cell reaches `TableRenderer`. Ordinary inline formulas stay compact; `\displaystyle` and display math retain their multi-line terminal layout, which `comfy-table` treats as explicit cell lines. Math pipes are protected during Markdown parsing and restored before this stage. Outside tables, `math_block_style` controls the display container independently from `code_block_style`.

## Table layout

- `fit`: `comfy-table` wraps cell content to the available terminal width.
- `wrap`: an oversized table is divided into successive column blocks, each with an indicator.
- `none`: no width limit is applied, so a table may exceed the terminal width.

Cell breakpoints are controlled independently by the document `wrap` mode. `char` hard-wraps at
Unicode grapheme boundaries, while `word` prefers spaces and falls back to graphemes for an
oversized token. `none` avoids soft breaks, but constrained `fit` and `wrap` layouts may still use
the same grapheme fallback when a cell cannot otherwise fit. With `table_wrap=none`, no cell width
is imposed and all three text modes leave cell lines unwrapped. Source spaces that land exactly at
a grapheme-wrap boundary are removed before the final layout pass, so following graphemes consume
the released width while column geometry and alignment padding remain unchanged.

`table_borders=false` uses compact borders without a complete outer grid. `table_borders=true` enables `UTF8_FULL` with rounded corners.
An explicit `table:border` theme color applies to vertical and horizontal structural lines in both layouts and to separators between wrapped table blocks. Without an override, all these lines use the terminal text color. Wrapped-block labels use `text_light`; cell content retains its own styling.

## Smart table indentation

`event/tables/layout.rs` estimates actual column widths and compares layouts against width available in the current heading, list, or blockquote context. The indentation decision occurs before `comfy-table`; afterward, every rendered row receives the same prefix.

## Invariants

- Reference counters are monotonic within their selected scope.
- A reference block cannot escape the blockquote, callout, or table that owns it.
- ANSI and OSC replacements do not change visible cell width.
- A header-only table has no empty body separator.
- HTML and Markdown tables share one low-level model.
- A missing footnote never panics or stops rendering; configuration controls its visible diagnostic.
