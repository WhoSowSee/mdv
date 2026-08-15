# Links, Footnotes, and Tables

These subsystems interact closely: links and footnotes may occur inside tables, callouts, and code containers, while reference blocks must finish at the boundary of their owner.

## Links

| File | Responsibility |
|---|---|
| [event/links.rs](../../src/renderer/event/links.rs) | Shared table-link helpers and submodule registration. |
| [event/links/start.rs](../../src/renderer/event/links/start.rs) | Start a link and enable `in_link`. |
| [event/links/end.rs](../../src/renderer/event/links/end.rs) | Select clickable, inline, or reference behavior when a link closes. |
| [event/links/inline.rs](../../src/renderer/event/links/inline.rs) | Render an inline URL beside link text. |
| [event/links/clickable.rs](../../src/renderer/event/links/clickable.rs) | OSC 8 links, wrapping, and current-line width limits. |
| [event/links/references.rs](../../src/renderer/event/links/references.rs) | Paragraph, table, and document reference blocks plus counter finalization. |
| [event/links/wrapping.rs](../../src/renderer/event/links/wrapping.rs) | URL breakpoints, indentation, ellipses, and reference-aware wrapping. |

### Link styles

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

The zero-width delimiter `U+200B` permits a break before `[N]` without changing visible width.

## Footnotes

| File | Responsibility |
|---|---|
| [event/footnotes.rs](../../src/renderer/event/footnotes.rs) | Definition types, diagnostic messages, and the module facade. |
| [footnotes/extraction.rs](../../src/renderer/event/footnotes/extraction.rs) | Remove definitions from the stream and register references. |
| [footnotes/scanning.rs](../../src/renderer/event/footnotes/scanning.rs) | Find placeholder and bare markers across text events. |
| [footnotes/markdown.rs](../../src/renderer/event/footnotes/markdown.rs) | Handle definitions in Markdown-like plaintext code and invalid placeholders. |
| [footnotes/rendering.rs](../../src/renderer/event/footnotes/rendering.rs) | Attached/endnote blocks, occurrences, wrapping, and missing-definition policy. |

### Model

- `FootnoteDefinitionKind::Normal` contains an event body.
- `EmptyBody` records a declaration without content.
- `InvalidSyntax` retains a diagnosable malformed marker.
- `footnote_order` follows first use rather than definition order.
- `footnote_use_count` distinguishes repeated occurrences.

`FootnoteStyle::Attached` flushes accumulated footnotes at a paragraph or container boundary. `Endnotes` defers them to document end. `MissingFootnoteStyle` controls whether diagnostic entries are visible.

## Event-level tables

| File | Responsibility |
|---|---|
| [event/tables.rs](../../src/renderer/event/tables.rs) | Constants and the connection from `EventRenderer` to low-level `TableRenderer`. |
| [event/tables/layout.rs](../../src/renderer/event/tables/layout.rs) | Column limits, smart indentation, URL truncation, and container prefixes. |
| [event/tables/rendering.rs](../../src/renderer/event/tables/rendering.rs) | Close a table, prepare data, handle embedded/HTML tables, and perform final rendering. |

`TableState` accumulates headers, rows, alignments, the current cell, inline URLs, and replacements. At `TagEnd::Table`, this state becomes parameters for the independent `TableRenderer`.

## Low-level `TableRenderer`

| File | Responsibility |
|---|---|
| [src/table.rs](../../src/table.rs) | `TableRenderer`, compact style, and the shared table-block type. |
| [src/table/layout.rs](../../src/table/layout.rs) | Cells, maximum widths, total-width estimation, and column-block partitioning. |
| [src/table/rendering.rs](../../src/table/rendering.rs) | `fit`, `wrap`, and `none` modes, alignment, and pretty/compact borders. |
| [src/table/links.rs](../../src/table/links.rs) | Restore ANSI and OSC wrappers after table layout. |

### Table wrap modes

- `fit`: `comfy-table` wraps cell content to the available terminal width.
- `wrap`: an oversized table is divided into successive column blocks, each with an indicator.
- `none`: no width limit is applied, so a table may exceed the terminal width.

`pretty_table=false` uses compact borders without a complete outer grid. `pretty_table=true` enables `UTF8_FULL` with rounded corners.

## Smart table indentation

`event/tables/layout.rs` estimates actual column widths and compares layouts against width available in the current heading, list, or blockquote context. The indentation decision occurs before `comfy-table`; afterward, every rendered row receives the same prefix.

## Invariants

- Reference counters are monotonic within their selected scope.
- A reference block cannot escape the blockquote, callout, or table that owns it.
- ANSI and OSC replacements do not change visible cell width.
- A header-only table has no empty body separator.
- HTML and Markdown tables share one low-level model.
- A missing footnote never panics or stops rendering; configuration controls its visible diagnostic.
