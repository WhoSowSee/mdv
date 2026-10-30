# Renderer Embedded HTML

`--render-html` converts HTML fragments embedded in Markdown into terminal content.
`--html` exports an HTML document through `TerminalRenderer::to_html`.
The [file index](file-index.md#srcrenderereventhtml) lists HTML handlers.

## Model

[src/renderer/event/html.rs](../../src/renderer/event/html.rs) uses `scraper` and `ego-tree`
to traverse HTML. `HtmlContext` carries alignment, whitespace preservation, inline styling,
script position, list state, and DOM depth into specialized tag handlers.

`HtmlContext` carries the DOM depth through every traversal path. Terminal HTML
rendering rejects nesting beyond 64 elements with a regular error before another
recursive call, including lists, details, definition lists, and table cells.

## Buffered containers

Some HTML tags may contain ordinary Markdown events between opening and closing fragments. `HtmlBlockBuffer` temporarily stores both HTML and Markdown events, then `render_html_fragment_as_terminal` processes the complete container.

`HtmlBlockKind` distinguishes element containers from standalone comments. Multiline comments are buffered through their closing delimiter before literal rendering, retaining whitespace and source-line markers. An unclosed standalone comment retains its original source through document end. Text after a closing comment delimiter continues through HTML rendering. Container detection and details-depth tracking retain comment state across events so tags inside comments cannot close or open a container.

Generic containers retain the existing Markdown block boundaries: a comment spanning a blank-line boundary inside a `div` can still be split by the Markdown parser.

This preserves context for alignment, `<details>`, definition lists, figures, and HTML tables. Printing an opening tag immediately would lose information from its closing element and descendants.

`<details>` buffers Markdown events and HTML fragments across blank lines until the matching closing tag. Outside table cells, its first summary receives an expanded `` marker and code-colored bold text. Body lines use a border-colored `│` guide and two spaces, included in wrapping and table width calculations. Nested sections repeat the guide. Content blocks honor explicit and inherited HTML alignment within the available width after the guides; summary markers stay at the tree's left edge. Sections remain expanded regardless of `open`, with no interactive folding. The container adds no fixed blank lines: body blocks use the same `block_spacing` settings as outside the section, and trailing blank lines retain their count and source metadata when the guide ends. Table cells retain their existing plain summary/body representation.

HTML `p`, `pre`, `blockquote`, `dl`, and top-level lists use paragraph, code-block, blockquote, definition-list, and ordered/unordered/task-list spacing respectively. Headings, tables, and horizontal rules already use their shared renderers' spacing. Generic layout containers such as `div` add no semantic block spacing, and table cells keep their compact layout.

## Text modes

- Ordinary HTML text collapses whitespace runs.
- Comments retain their literal `<!-- ... -->` delimiters and whitespace, including inside HTML containers, and use the same wrapping as literal HTML. `hide_comments` suppresses them without hiding surrounding content.
- `<pre>` and `<textarea>` preserve spaces and line breaks.
- `<code>` uses code-like semantic styling without entering the fenced-code pipeline.
- `<sup>` and `<sub>` use `math::convert_script` when a Unicode representation exists.
- `<mark>` enables highlight context.

Raw `code`, `pre`, and `textarea` content is combined before Markdown smart punctuation,
preserving quotes, dashes, and ellipses. Preformatted blocks use the shared code layouts;
`pre` language hints can come from `language-*`/`lang-*` classes or language attributes.

## Lists

Ordered lists support:

- `start`;
- `reversed`;
- `li[value]`;
- decimal, alphabetic, and Roman marker types.

Numbering supports the signed 64-bit range. If an implicit item number would
exceed this range, rendering returns an error. A final item at either boundary
is valid, and an explicit `li[value]` can restart numbering after that boundary.

An unordered marker is selected from tag attributes or style. An HTML checkbox inside a list item becomes a task marker rather than an independent form control.

## HTML tables

An HTML table is first normalized to a rectangular structure. `thead`, `tbody`, `tfoot`, `tr`, `th`, and `td` populate the same `TableState` consumed by the shared `event/tables` renderer.

Cells support headings, blockquotes, figures, preformatted blocks, and horizontal rules. The internal `HTML_TABLE_HORIZONTAL_RULE` marker carries a rule through the intermediate string model and expands before final table rendering using the same `horizontal_rule_style` and theme color as rules outside tables.

Block-level nesting in cells requires HTML and `--render-html`. GFM Markdown tables accept
inline content; Markdown syntax alone does not create nested block lists or tables in cells.

## Media and forms

Media tags do not download resources. They show a typed marker, label, and source path or URL; `srcset` uses its first candidate.

Form controls are also non-interactive. Input, button, and select elements render as compact static representations, matching the viewer's read-only role.

## Invariants

- DOM traversal does not mutate global configuration or theme state.
- Buffered HTML is flushed at document end even when a closing fragment is missing.
- Preformatted containers bypass whitespace collapsing.
- HTML tables use the shared table renderer and wrapping strategies.
- Unknown tags retain readable child content; scripts and styles are never executed.
