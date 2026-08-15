# Renderer Facade and Event Core

Rendering is split between an outer document facade and one stateful `EventRenderer`.

## Root files

| File | Responsibility |
|---|---|
| [src/renderer/mod.rs](../../src/renderer/mod.rs) | Declares the terminal renderer, event handlers, line numbering, and syntax resources. |
| [src/renderer/terminal.rs](../../src/renderer/terminal.rs) | `TerminalRenderer`: themes, syntax set, code theme, and ANSI/HTML entry points. |
| [src/renderer/line_numbers.rs](../../src/renderer/line_numbers.rs) | Source and rendered gutters plus removal of internal line markers. |
| [src/renderer/syntax_set.rs](../../src/renderer/syntax_set.rs) | Cached embedded `SyntaxSet` with optional user `.sublime-syntax` files. |
| [src/renderer/syntax_theme.rs](../../src/renderer/syntax_theme.rs) | Adapts a `syntect` code theme to the terminal palette. |
| [src/renderer/event/mod.rs](../../src/renderer/event/mod.rs) | Shared imports and registration of all event-handler groups. |

## `TerminalRenderer`

`TerminalRenderer` owns the prepared resources for one render configuration:

- a cloned `Config`;
- the selected terminal `Theme`;
- an `Arc<SyntaxSet>`;
- a `CodeHighlightTheme`.

### Construction

`TerminalRenderer::new`:

1. builds a `ThemeManager` from embedded and user themes;
2. selects the terminal theme;
3. applies `custom_theme` and inline-style overrides;
4. loads the syntax set, including `syntaxes_dir`;
5. selects or builds the code theme.

### Rendering

`render(events)` chooses one of three paths:

- no line numbers: call `render_events` directly;
- rendered line numbers: render first, then number visual rows;
- source line numbers: decode markers emitted by the Markdown pipeline.

Only the left margin is added to output lines at the end. The right margin reduces available width but does not append spaces.

`to_html(events)` is a separate export backend and does not treat ANSI output as an intermediate representation.

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
| Footnotes | definitions, order, occurrences, scan buffer, and suppression flags. |
| Inline formatting | semantic formatting stack and active backtick style. |
| Paragraph spacing | content flags, blank-line streak, and soft-break suppression. |

Fields are crate-visible because inherent `impl EventRenderer` blocks are physically distributed across sibling files. They are not independent public state.

## `event/core` files

| File | Responsibility |
|---|---|
| [core/state.rs](../../src/renderer/event/core/state.rs) | `ListState`, `TableState`, and callout, footnote, link, and HTML state types. |
| [core/constructor.rs](../../src/renderer/event/core/constructor.rs) | Complete initialization in `EventRenderer::new`. |
| [core/render.rs](../../src/renderer/event/core/render.rs) | Document lifecycle and smart-indent pre-analysis. |
| [core/process.rs](../../src/renderer/event/core/process.rs) | Dispatch one `Event` and process source markers. |
| [core/start_tags.rs](../../src/renderer/event/core/start_tags.rs) | Route `Event::Start(Tag)` to specialized handlers. |
| [core/end_tags.rs](../../src/renderer/event/core/end_tags.rs) | Route `Event::End(TagEnd)` and hard breaks. |
| [core/end_paragraph.rs](../../src/renderer/event/core/end_paragraph.rs) | Finish paragraphs, references, attached footnotes, and spacing. |
| [core/end_blockquote.rs](../../src/renderer/event/core/end_blockquote.rs) | Close a quote or callout and restore indentation state. |
| [core/end_lists.rs](../../src/renderer/event/core/end_lists.rs) | Close lists and items while reconciling nesting. |
| [core/callouts.rs](../../src/renderer/event/core/callouts.rs) | Resolve callout type, base palette, and unique fallback colors. |

## Document lifecycle

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
11. Normalize the trailing newline to one.

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
