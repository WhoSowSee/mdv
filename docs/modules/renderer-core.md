# Renderer Facade and Event Core

Rendering is split between an outer document facade and one stateful `EventRenderer`.

## Root files

| File | Responsibility |
|---|---|
| [src/renderer/mod.rs](../../src/renderer/mod.rs) | Declares the terminal renderer, event handlers, line numbering, and syntax resources. |
| [src/renderer/terminal.rs](../../src/renderer/terminal.rs) | `TerminalRenderer`: themes, syntax set, code theme, and event-stream ANSI/HTML entry points. |
| [src/renderer/terminal/pager.rs](../../src/renderer/terminal/pager.rs) | Builds unnumbered, rendered-numbered, and source-numbered pager variants with source-line maps. |
| [src/renderer/front_matter.rs](../../src/renderer/front_matter.rs) | Front matter panel, table, normalized text, blocks, YAML source, and HTML event formatting. |
| [src/renderer/line_numbers.rs](../../src/renderer/line_numbers.rs) | Source and rendered gutters plus removal of internal line markers. |
| [src/renderer/syntax_set.rs](../../src/renderer/syntax_set.rs) | Cached embedded `SyntaxSet` with optional user `.sublime-syntax` files. |
| [src/renderer/syntax_theme.rs](../../src/renderer/syntax_theme.rs) | Adapts a `syntect` code theme to the terminal palette. |
| [src/renderer/event/mod.rs](../../src/renderer/event/mod.rs) | Shared imports and registration of all event-handler groups. |

## `TerminalRenderer`

`TerminalRenderer` owns the prepared resources for one render configuration:

- a cloned `Config`;
- the selected terminal `Theme`;
- an `Arc<SyntaxSet>`;
- a `CodeHighlightTheme`;
- a resolved `terminal::OutputStyle`, separate from the serialized `Config.color` mode; `cli::OutputStyle` remains a compatible public re-export.

### Construction

`TerminalRenderer::new(config, output_style)`:

1. builds a `ThemeManager` from embedded and user themes;
2. selects the terminal theme;
3. applies `custom_theme` and inline-style overrides;
4. loads the syntax set, including `syntaxes_dir`;
5. selects or builds the code theme.

Construction does not inspect process stdout or color environment variables.
Library callers explicitly select `OutputStyle::Enabled`/`Disabled` or resolve
`config.color` for their own destination. Nested renderers and table layouts
inherit the same policy. Disabled styling prevents generation of SGR and OSC 8;
it is not implemented by stripping all ANSI from the completed document.

### Rendering

`render(events)` chooses one of three paths:

- no line numbers: call `render_events` directly;
- rendered line numbers: render first, then number visual rows;
- source line numbers: decode markers emitted by the Markdown pipeline.

Pager rendering supplies a factory for unnumbered, rendered-numbered, and source-numbered output with an independent source-line map for each layout. The pager prepares only the selected variant for its first screen and caches other variants on first use. Every variant passes through the same renderer, theme, gutter-width, margin, wrapping, and separator logic as ordinary CLI output. The configured line-number target selects the initial pager variant.

Only the left margin is added to output lines at the end. The right margin reduces available width but does not append spaces.

`to_html(events)` is a separate export backend and does not treat ANSI output as an intermediate representation.

`render_document(parsed_document)` and `to_html_document(parsed_document)` convert front matter into synthetic callout, table, paragraph, definition-list, code-block, or HTML events before using the existing event-only entry points. Source mode bypasses front matter extraction entirely. The properties callout uses dedicated title, key, value, and border theme roles and defers wrapping until the frame width is known. Generated property rows have an empty source-number gutter; rendered numbering includes them as ordinary visual rows.

In reverse mode, front matter follows the reversed Markdown body so it remains at the visible end of long terminal output.

## `EventRenderer`

[src/renderer/event/core.rs](../../src/renderer/event/core.rs) defines the only stateful renderer for the event stream. Its fields are grouped by subsystem:

| Group | Example state |
|---|---|
| Output and layout | `output`, `current_indent`, and heading/content indentation. |
| Blockquotes and callouts | quote depth, `callout_stack`, palette, and pending marker buffers. |
| Lists and definitions | `list_stack`, prepared spacing queues, and the definition-list stack. |
| Tables and HTML | optional `TableState` and pending HTML-block buffer. |
| Links | current link text, paragraph/document references, and counters. |
| Code | code-block buffer, language, plaintext depth, and captured references. |
| Math | shared `Rc<MathDiagnostics>` for the current document render. |
| Footnotes | definitions, order, occurrences, scan buffer, and suppression flags. |
| Inline formatting | semantic formatting stack and active backtick style. |
| Paragraph spacing | content flags, blank-line streak, and soft-break suppression. |

Fields use `pub(super)` in `event/core.rs`, so sibling handlers within
`renderer::event` can share state while the outer facade cannot read its fields.
The type, constructor, render entry point, and `max_code_line_number_width()`
accessor are scoped to `renderer`. The accessor supplies the gutter measurement
required by the facade's iterative layout without exposing mutable state.

## `event/core` files

| File | Responsibility |
|---|---|
| [core/state.rs](../../src/renderer/event/core/state.rs) | `ListState`, `TableState`, and callout, footnote, link, and HTML state types. |
| [core/constructor.rs](../../src/renderer/event/core/constructor.rs) | Complete initialization in `EventRenderer::new`. |
| [core/render.rs](../../src/renderer/event/core/render.rs) | Document lifecycle, gutter measurement, and smart-indent pre-analysis. |
| [core/process.rs](../../src/renderer/event/core/process.rs) | Dispatch one `Event` and process source markers. |
| [core/output.rs](../../src/renderer/event/core/output.rs) | Replace output spacing while rebasing positions held by open blocks. |
| [core/start_tags.rs](../../src/renderer/event/core/start_tags.rs) | Route `Event::Start(Tag)` to specialized handlers. |
| [core/end_tags.rs](../../src/renderer/event/core/end_tags.rs) | Route `Event::End(TagEnd)` and hard breaks. |
| [core/end_paragraph.rs](../../src/renderer/event/core/end_paragraph.rs) | Finish paragraphs, references, attached footnotes, and spacing. |
| [core/end_blockquote.rs](../../src/renderer/event/core/end_blockquote.rs) | Close a quote or callout and restore indentation state. |
| [core/end_lists.rs](../../src/renderer/event/core/end_lists.rs) | Close lists and items while reconciling nesting. |
| [core/callouts.rs](../../src/renderer/event/core/callouts.rs) | Resolve callout type, base palette, and unique fallback colors. |

## Document lifecycle

`core/subsystems.rs` groups `LinkState`, `CodeState`, and `FootnoteState` under
the same event renderer. Active links and fenced code blocks use `Option`;
link destinations distinguish direct URLs, scoped references, and callout
labels. Temporary string keys such as `current_N` are no longer used.

Internal rendering borrows the original event slice and its text payloads.
Repeated gutter/layout passes do not clone the full owned event stream. Only
events that outlive the current pass, such as captured HTML details and footnote
bodies, become owned.

`render_events` runs these operations in order:

1. Extract footnote definitions from the primary event stream.
2. Prepare block-spacing queues.
3. For `heading_layout=level` with smart indentation, build the heading-level map.
4. Call `process_event` for every event.
5. Close unfinished inline backticks.
6. Flush buffered HTML.
7. Complete a pending empty-heading placeholder.
8. Complete attached footnotes.
9. Render document link references.
10. Render endnote footnotes.
11. Remove mathematical layout metadata after all containers finish.
12. Normalize the trailing newline to one.

Ordering matters. A link inside code, a table, or a callout may defer its reference block until the containing element closes.

## Dispatch

`process_event` contains routing rather than element implementations. It sends:

- start and end tags to `start_tags.rs` and `end_tags.rs`;
- text to `text/handling.rs`;
- inline and block code to `code/*`;
- HTML to `html/*` or the literal HTML handler;
- links, images, math, footnotes, task markers, and rules to their dedicated modules.

Add new behavior to a specialized handler and keep the core dispatcher compact.

## Line numbers

`renderer/line_numbers.rs` uses an invisible internal marker encoding. Markers must:

- occupy zero display columns;
- survive ANSI styling and wrapping;
- distinguish the number and separator so themes can color them independently;
- be removed by `strip_internal_markers` before final output.

## Extension invariants

- Every event is processed exactly once.
- A handler must not instantiate another `EventRenderer` to bypass current state.
- Deferred blocks finish at the boundary owned by their paragraph, table, callout, or document.
- All width operations use `display_width`, never `str::len`.
- Finalization for new state belongs in `render_events` or the corresponding end-tag handler.
- Removing or replacing existing spacing must rebase the saved byte positions of open paragraphs, quotes, headings, lists, and definition descriptions before those positions are reused.
