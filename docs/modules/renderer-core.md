# Renderer Facade and Event Core

`TerminalRenderer` prepares document-wide resources. `EventRenderer` owns the mutable state
of one terminal-rendering pass. See the [file index](file-index.md#srcrenderer) for handlers
and supporting modules.

## TerminalRenderer

The facade owns a `Config` snapshot, terminal theme, shared `SyntaxSet`, code-highlight theme,
and resolved `OutputStyle`. Construction loads theme and syntax resources and applies the
[source-ordered overrides](themes-and-styling.md#application-order).

Library callers supply the output policy explicitly. Renderers do not inspect process
stdout or color environment variables. Disabled styling prevents SGR and OSC 8 generation;
structural terminal controls remain the responsibility of interactive adapters.

### Rendering

A pass resolves available width once and shares it with layout helpers and nested renderers.
Document margins and line-number gutters reduce content width. The left margin is added to
nonempty output rows; the right margin reserves space without appending padding.

| Numbering | Layout |
|---|---|
| Off | Render content without a document gutter. |
| Rendered | Number visual rows after wrapping. |
| Source | Decode the Markdown pipeline's source markers; continuation and generated rows may have blank gutters. |

The pager uses the same rendering rules for its
[cached views](interactive-and-pager.md#pagerdocument). Monitor refreshes resolve width again;
pager reflow supplies an explicit layout width.

`render_document` and `to_html_document` accept front matter alongside body events. Property
views use the configured callout, table, paragraph, definition-list, or code layout with
independent theme roles. Generated property rows participate in rendered numbering but have
no source number. In reverse mode, front matter follows the reversed body. Source mode
bypasses front matter extraction.

HTML export consumes parsed events directly, without going through ANSI output.

## EventRenderer

Element handlers share one renderer's state within `renderer::event`. The facade can obtain
code-gutter measurements through a renderer-scoped accessor without accessing mutable fields.

| State | Responsibility |
|---|---|
| Output and indentation | Position tracking, container prefixes, heading scope, and spacing. |
| Formatting | Active semantic styles and inline backticks. |
| Containers | Lists, quotes/callouts, tables, HTML buffers, and definition descriptions. |
| Links and footnotes | Definitions, counters, ownership, and deferred output. |
| Code | Active block content, language, recursive rendering depth, and gutters. |
| Math diagnostics | Warning deduplication shared by layouts of one document. |

Rendering borrows the event slice and its text payloads. Only content that outlives a pass,
such as buffered HTML and footnote bodies, becomes owned. Nested renders share the relevant
reference stores, styling policy, and math diagnostics.

## Document lifecycle

Before dispatch, the renderer extracts footnote definitions, prepares block-spacing boundaries,
and calculates smart heading indentation when enabled. Specialized handlers process the
remaining event stream in order.

Closing a container completes the references and attached footnotes it owns. At document end,
the renderer closes pending inline formatting, flushes buffered HTML and heading placeholders,
and emits document references and endnotes. Mathematical layout metadata is removed after
all containers finish, and output ends with one newline.

New element behavior belongs in its specialized handler. Add finalization at the owning
end-tag boundary or document boundary; emitting a deferred block early can move it outside
its callout, table, or code container.

## Line-number markers

Source markers occupy zero display columns, survive styling and wrapping, and distinguish
number and separator colors. At line start, container prefixes and indentation precede the
marker. Final output removes the internal encoding.

## Layout invariants

- Measure visible text in terminal columns with Unicode-aware width helpers.
- Preserve the ANSI/OSC state of wrapped content without counting escape bytes as columns.
- Rebase saved output positions when spacing changes inside open blocks.
- Complete deferred output within its owning scope.
- Keep source-line metadata out of user-visible terminal and HTML output.
