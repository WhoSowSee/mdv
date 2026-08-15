# Renderer Embedded HTML

This module serves `--render-html`: HTML fragments embedded in Markdown become terminal content. It is not the backend for `--html`, which creates an HTML document through the separate `TerminalRenderer::to_html` path.

## Model

[src/renderer/event/html.rs](../../src/renderer/event/html.rs) uses `scraper` and `ego-tree` for DOM-like traversal and defines:

- `HtmlContext`, including alignment, whitespace preservation, highlighting, script position, and list depth;
- `HtmlAlignment`;
- ordered and unordered list-marker state;
- shared helper imports.

Every HTML node passes through `render_html_node`; element dispatch selects the specialized renderer for its tag.

## Module files

| File | Responsibility |
|---|---|
| [html/dispatch.rs](../../src/renderer/event/html/dispatch.rs) | Traverse nodes, elements, and children and dispatch by tag. |
| [html/blocks.rs](../../src/renderer/event/html/blocks.rs) | Common block, heading, link, code, abbreviation, literal, `pre`, and `textarea` elements. |
| [html/text.rs](../../src/renderer/event/html/text.rs) | Text collapsing, separators, and `<br>`. |
| [html/text_helpers.rs](../../src/renderer/event/html/text_helpers.rs) | Preformatted-text normalization, escaping, and void elements. |
| [html/styles.rs](../../src/renderer/event/html/styles.rs) | Alignment and CSS-like inline styles mapped to `ThemeElement`. |
| [html/layout.rs](../../src/renderer/event/html/layout.rs) | Block boundaries, indentation/alignment spans, and table-cell line context. |
| [html/buffer.rs](../../src/renderer/event/html/buffer.rs) | Accumulate HTML containers that include Markdown events. |
| [html/buffer_helpers.rs](../../src/renderer/event/html/buffer_helpers.rs) | Buffered-tag lists and block/inline-container classification. |
| [html/blockquotes.rs](../../src/renderer/event/html/blockquotes.rs) | Render `<blockquote>` through the shared quote context. |
| [html/definitions.rs](../../src/renderer/event/html/definitions.rs) | Render `<dl>`, `<dt>`, `<dd>`, `<figure>`, and `<figcaption>`. |
| [html/lists.rs](../../src/renderer/event/html/lists.rs) | Render `<ol>`, `<ul>`, `<li>`, `<details>`, `<summary>`, and styled block lines. |
| [html/list_helpers.rs](../../src/renderer/event/html/list_helpers.rs) | Checkbox detection plus `start`, `reversed`, `value`, alpha, and Roman markers. |
| [html/forms.rs](../../src/renderer/event/html/forms.rs) | Static terminal representation of input, button, and select controls. |
| [html/media.rs](../../src/renderer/event/html/media.rs) | Media markers and lines for image, video, audio, and source elements. |
| [html/media_helpers.rs](../../src/renderer/event/html/media_helpers.rs) | Extract `src`, `srcset`, labels, and filenames. |
| [html/tables.rs](../../src/renderer/event/html/tables.rs) | HTML table sections, rows, and cells. |
| [html/table_cells.rs](../../src/renderer/event/html/table_cells.rs) | Nested block elements inside table cells. |
| [html/table_helpers.rs](../../src/renderer/event/html/table_helpers.rs) | Alignment attributes/styles and rectangular-table normalization. |

## Buffered containers

Some HTML tags may contain ordinary Markdown events between opening and closing fragments. `HtmlBlockBuffer` temporarily stores both HTML and Markdown events, then `render_html_fragment_as_terminal` processes the complete container.

This preserves context for alignment, `<details>`, definition lists, figures, and HTML tables. Printing an opening tag immediately would lose information from its closing element and descendants.

## Text modes

- Ordinary HTML text collapses whitespace runs.
- `<pre>` and `<textarea>` preserve spaces and line breaks.
- `<code>` uses code-like semantic styling without entering the fenced-code pipeline.
- `<sup>` and `<sub>` use `math::convert_script` when a Unicode representation exists.
- `<mark>` enables highlight context.

## Lists

Ordered lists support:

- `start`;
- `reversed`;
- `li[value]`;
- decimal, alphabetic, and Roman marker types.

An unordered marker is selected from tag attributes or style. An HTML checkbox inside a list item becomes a task marker rather than an independent form control.

## HTML tables

An HTML table is first normalized to a rectangular structure. `thead`, `tbody`, `tfoot`, `tr`, `th`, and `td` populate the same `TableState` consumed by the shared `event/tables` renderer.

Cells support headings, blockquotes, figures, preformatted blocks, and horizontal rules. The internal `HTML_TABLE_HORIZONTAL_RULE` marker carries a rule through the intermediate string model and expands before final table rendering.

## Media and forms

Media tags do not download resources. They show a typed marker, label, and source path or URL; `srcset` uses its first candidate.

Form controls are also non-interactive. Input, button, and select elements render as compact static representations, matching the viewer's read-only role.

## Invariants

- DOM traversal does not mutate global configuration or theme state.
- Buffered HTML is flushed at document end even when a closing fragment is missing.
- Preformatted containers bypass whitespace collapsing.
- HTML tables use the shared table renderer and wrapping strategies.
- Unknown tags retain readable child content; scripts and styles are never executed.
