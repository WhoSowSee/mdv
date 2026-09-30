# Interactive Mode and Pager

The interactive subsystem has two levels: a Markdown document browser and a pager for one document. Ordinary `--pager` uses the same pager selection without the browser UI.

## Target selection

[src/interactive/mod.rs](../../src/interactive/mod.rs) defines `InteractiveTarget`:

- `Directory(PathBuf)` opens the browser;
- `File(PathBuf)` opens the pager immediately;
- `Stdin` reads standard input and opens a pager without file actions.

`select_interactive_target` considers the filename, `--interactive`, `--pager`, and whether standard input and output are terminals. An explicit pager takes precedence and bypasses interactive target selection.

With no filename and terminal standard input and output, the current directory opens automatically. Redirected output does not select the browser implicitly. Explicit interactive targets still require terminal standard output.

## Browser files

| File | Responsibility |
|---|---|
| [interactive/mod.rs](../../src/interactive/mod.rs) | Target selection, event loop, and browser-to-pager/editor transitions. |
| [interactive/app.rs](../../src/interactive/app.rs) | `App`, `AppAction`, and keyboard, mouse, paste, and resize handling. |
| [interactive/browser.rs](../../src/interactive/browser.rs) | `BrowserState`: sections, selection, paging, filter, errors, and help state. |
| [interactive/browser/loading.rs](../../src/interactive/browser/loading.rs) | Incremental discovery ingestion, refresh state, sorting, and selection preservation. |
| [interactive/browser/tests.rs](../../src/interactive/browser/tests.rs) | Browser discovery-state regression tests. |
| [interactive/discovery.rs](../../src/interactive/discovery.rs) | Background Markdown discovery and fuzzy matching. |
| [interactive/screen.rs](../../src/interactive/screen.rs) | Screen constants and facade for visual submodules. |

### Screen submodules

| File | Responsibility |
|---|---|
| [screen/session.rs](../../src/interactive/screen/session.rs) | Raw mode and alternate screen, pager pause, and editor suspension. |
| [screen/draw.rs](../../src/interactive/screen/draw.rs) | Complete browser frame and error overlay. |
| [screen/frame.rs](../../src/interactive/screen/frame.rs) | In-memory frames, synchronized row diffs, and cursor state. |
| [screen/header.rs](../../src/interactive/screen/header.rs) | Logo, title, filter prompt, pagination, and selection helpers. |
| [screen/help.rs](../../src/interactive/screen/help.rs) | Mini, full, and filter help plus footer rows. |
| [screen/style.rs](../../src/interactive/screen/style.rs) | Crossterm styling, sanitization, and plain-text truncation. |
| [screen/time.rs](../../src/interactive/screen/time.rs) | Relative and local document timestamps. |

## Browser state

`BrowserState` owns:

- discovery results;
- the active section;
- query and filter state;
- filtered indices and selection;
- page size and count;
- help and error overlays.
- whether files excluded by Git ignore rules are visible.

Discovery runs independently and publishes each document or error through a bounded channel. `poll_discovery` consumes a limited number of events on every UI tick, inserts newly found documents into the sorted list, refreshes an active filter, and preserves the selected path while the list grows. The line spinner beside the logo appears only after a 16 ms grace period and starts from its first frame; a final event stops it. Fuzzy matching normalizes Unicode but returns indices into the original string so highlighting remains correct.

Pressing `.` toggles files excluded by `.gitignore`, the global Git ignore file, and `.git/info/exclude`, then restarts discovery. Rediscovery is double-buffered, so the current list remains visible until its replacement is complete. Rapid toggles are debounced and only the final mode is scanned. Hidden paths, `.ignore` rules, and the dedicated `node_modules` exclusion remain active in both modes.

## Event loop

`interactive::run`:

1. enters a `TerminalSession`;
2. calls `app.tick()` and marks discovery or input changes for redraw;
3. coalesces pending states and renders at no more than 120 frames per second;
4. blocks directly on Crossterm input while the browser is idle;
5. maps each event to `AppAction`;
6. temporarily pauses the browser screen for the pager;
7. fully suspends the terminal session for an editor;
8. restores the terminal and records the operation result in application state.

`draw_browser` builds a `ScreenFrame` in memory instead of writing directly to standard output. `TerminalSession` compares it with the last displayed frame, encodes only changed or removed rows, and sends the complete ANSI update in one synchronized write. Identical frames produce no terminal output. The first frame, a terminal resize, and every resume after pager, editor, or suspension invalidate the cache and force a full redraw. Frame deadlines are scheduled from the completion of the previous write, so slow terminals lower the effective frame rate without accumulating catch-up frames.

## Pager files

| File | Responsibility |
|---|---|
| [src/pager.rs](../../src/pager.rs) | Module facade and internal re-exports. |
| [pager/command.rs](../../src/pager/command.rs) | Backend precedence, command parsing, recursion checks, and external process lifecycle. |
| [pager/document.rs](../../src/pager/document.rs) | `PagerDocument`, line-number view state, `RefreshCallback`, and `PagerScreen`. |
| [pager/page.rs](../../src/pager/page.rs) | Configure `minus::Pager` and run the pager/editor loop. |
| [pager/rendering.rs](../../src/pager/rendering.rs) | Supply the pager view factory, prefixes, and source-line maps. |
| [pager/document/views.rs](../../src/pager/document/views.rs) | Cache line-number variants on first use and defer the source-navigation view. |
| [pager/warmup.rs](../../src/pager/warmup.rs) | Start delayed background preparation of numbered views after the first screen. |
| [pager/input.rs](../../src/pager/input.rs) | Custom input classifier for help, copy, reload, and editor actions. |
| [pager/toc.rs](../../src/pager/toc.rs) | Extract and normalize source-mapped headings for the contents panel. |
| [pager/interrupt.rs](../../src/pager/interrupt.rs) | Preserve parent interrupt handling and configure external pager children. |
| [pager/footer.rs](../../src/pager/footer.rs) | Opaque/transparent footer, title, progress, and width clamping. |
| [pager/help.rs](../../src/pager/help.rs) | Prompt panel listing available shortcuts. |
| [pager/operations.rs](../../src/pager/operations.rs) | Document replacement, clipboard handling, and status/error messages. |
| [pager/watcher.rs](../../src/pager/watcher.rs) | `notify` watcher and debounced refresh. |

## Pager backend selection

The default backend is the built-in `minus` pager. A command from `MDV_PAGER`
replaces it wherever mdv already opens a pager: explicit `--pager`, full help,
and a document selected in the interactive browser. The variable does not turn
ordinary output into paged output. An explicit `--pager=<COMMAND>` overrides the
environment. `--pager=default` selects the default backend, currently the
built-in `minus`, while `--pager=builtin` restores `minus` explicitly. A bare
`--pager` only requests paging and retains the environment-selected backend.

External commands accept quoted program paths and arguments. mdv splits the
command without shell evaluation, rejects empty or recursive commands, pipes the
active rendered view to the child's standard input, inherits its output streams,
temporarily handles Ctrl+C in the parent while waiting for the child, and waits
for a successful exit. It does not silently fall back to stdout and does not
inject pager-specific flags; a color-capable `less` command therefore normally
includes `-R`. Redirected stdout retains the existing direct-output behavior and
does not start either pager backend. External paging uses one rendered view;
the built-in pager caches its switchable views and prepares source navigation
on demand.

The built-in pager owns mdv-specific search, copy, reload, editor, help, and
line-number switching. An external pager owns its controls and receives only the
initial view selected by the effective line-number configuration.

The resolved color depth also reaches the browser and `Pager::set_color_depth`.
The built-in pager limits final content rows, prompts, help panels, incremental
search previews, and selection/navigation highlights. ANSI 16 uses contrasting
fixed pairs for search, selection, and navigation rather than RGB background
tints. External pagers receive already converted document text and control
their own interface colors.

When the browser opens an external pager, mdv fully suspends the browser session,
then restores the alternate screen and raw mode after the child exits. The
built-in pager uses the lighter in-place pause path because it shares mdv's
terminal session. This pause explicitly hides the cursor while the selected file
is read and rendered, including when the browser's filter prompt was visible.
The in-place pager does not show the cursor on exit to the browser. Returning to
the shell or launching an editor restores cursor visibility.

## `PagerDocument`

For the built-in backend, the document stores these values separately:

- lazily prepared unnumbered, rendered-numbered, and source-numbered ANSI views with their source-line maps, or one static output for non-Markdown pager content;
- the active line-number mode;
- optional `title`;
- `status_bar_transparent` from the selected theme.
- the resolved `OutputStyle` shared by document rendering and pager UI.

Copying uses only selected rendered text, with ANSI and OSC sequences removed.
Without a selection, copy commands do not access the clipboard or update the status.
The initial display and source-line map prepare only the selected numbering mode. Other variants share a cache that renders each mode once per layout width. The source-numbered view needed by `:` shares the same cache used by `l` and background preparation.

Five seconds after the `PostPagerStart` hook confirms the first screen was drawn, one background worker prepares rendered numbering followed by source numbering. Already prepared or currently rendering variants use the existing shared cache, so background preparation and early `l`/`:` requests do not duplicate work. Rendering holds no document lock and does not switch modes or redraw the screen. A background failure stays in the cache and is reported through the normal on-demand error path if that mode is requested.

Refresh and layout-width changes cancel pending work and start a new five-second delay once the replacement view is available. Repeated snapshots and numbering changes at the same width do not reset the timer. Exiting the pager or opening an editor cancels the wait immediately; an already running render may finish in its old cache, but no further mode is started and pager exit does not wait for it.

Refresh clears the view cache. The replacement renders its selected mode outside the document lock before being installed; an error preserves the current document. If the user changes numbering while refresh renders, the replacement prepares the newly selected mode before committing.

## Input classifier

The custom classifier extends the default `minus` classifier with:

- `?` to show or hide the help panel;
- `Esc` to close help without losing search state;
- `/` or `Ctrl+F` to search;
- `c`, `Ctrl+C`, or right-click to copy selected text;
- `Ctrl+A` to select the entire rendered document, including offscreen rows;
- `r` to refresh when a callback exists;
- `l` to cycle the mdv line-number views;
- `:` to open the source-line navigation prompt when source metadata is available;
- `e` to open the file in an editor when available.

The help panel uses three columns in wide terminals, two below 78 columns, and one below 52 columns; resizing an open panel recomputes its layout. Entries are ordered by the visible width of the key combination; descriptions do not affect ordering. Optional entries are removed when line navigation, line-number switching, reload, or editor integration is unavailable.

`l` cycles `off → rendered → source → off`. The initial position comes from the effective `line_numbers` setting, so the first transition depends on how mdv was launched. When line-number switching is available, this mdv binding takes precedence over the default `minus` horizontal-scroll binding for `l`; the right arrow remains available for horizontal scrolling. The built-in `minus` gutter remains disabled; every numbered view is produced by `TerminalRenderer` and therefore uses the configured mdv colors, separator, margins, and wrapping.

The `:` prompt accepts a one-based Markdown source line. It prepares and swaps in the source-numbered rendering when another line-number mode is active and reuses the current rendering in source mode. After a successful jump, that rendering and a fixed muted highlight remain active, while the footer shows `:N` immediately before document progress. `Esc` leaves line-navigation mode and restores the view selected by `l`. A missing line uses the same two-second status-message timeout as a search with no matches.

When an active search has matches, the footer shows the current and total occurrences immediately before document progress. Both status values use the muted `#5a5a5a` foreground. Incremental search updates the matching viewport and highlights after every query edit, before confirmation. Search navigation and counting operate on individual occurrences, including multiple matches in one row, and only the exact current range receives the stronger tint. The viewport stays fixed while the next occurrence is visible; the first result below it is revealed on the bottom row instead of being moved to the top. Match highlighting preserves syntax foreground colors and derives each background tint from the active text color. Mouse selection remains available during search, preserves syntax colors over a neutral `#2e313b` background, and produces a lighter combined tint where selection overlaps a match.

`Esc` closes the search prompt without restoring the pre-search viewport. If incremental search displayed a match, the pager remains at that displayed position while retaining the query for reuse.

Long clipboard and reload operations run on separate threads so pager input remains responsive. `reload_in_progress` prevents concurrent refreshes of one page.

## Table of contents

Outline-only scrolling redraws just the panel and footer, without clearing the screen or rewriting the document. Full redraws are assembled in memory and sent as a synchronized update so intermediate clearing and partial rows are not flushed to the terminal.

Width changes reuse the parsed Markdown and initialized renderer. The document keeps its current layout and at most two cached widths, preserving numbering mode when switching between them. Initial pager setup reuses the layout already rendered by the file selector. A background worker prepares the sidebar width without holding the document lock during rendering; its result is discarded if the document was refreshed meanwhile. Refresh creates a new cache, so older layouts never overwrite new source content.

Markdown pager documents retain a width-aware render callback. Uncached widths rebuild the selected view, including callout/code frames and table geometry, before terminal rows are formatted. Other numbering modes are prepared on demand or by delayed background work. Explicit CLI column limits remain an upper bound. Static pager output uses ordinary pager wrapping.

`Alt+Up/Down` traverses adjacent entries, including subsections. The footer displays a first-use navigation hint while the panel is open, until a keyboard outline-navigation or outline-scroll shortcut is used. Mouse actions and normal document scrolling do not acknowledge it. Acknowledgment lasts for the pager session and survives mapped-text replacement; status messages take temporary priority.

The outline uses the terminal's default background and the interactive browser's blue accent (`#7e9cd8`). `Shift+Up/Down` aliases `K/J`; `Alt+Shift+Up/Down` aliases `U/D`. These arrow bindings apply only while the panel is open and do not intercept other modifier combinations.

`pager/toc.rs` extracts heading labels and source targets from the processed Markdown events, preserving preprocessing and front-matter source maps. Only headings present in the unnumbered view are retained. A unique highest-level title is omitted once; the next two distinct levels become sections and subsections, following the outline organization in [leaf](https://github.com/rivolink/leaf).

`LineNavigation::with_toc` carries these entries in the same mapped-text update as all three line-number views. The vendored `toc` module owns panel state, input routing, source-to-display lookup, and rendering. `t` toggles it; `Esc` closes it; clicking selects an entry; `J/K` navigate sections; `1`–`9` select and cycle section groups; `D/U` and the wheel over the panel scroll the outline. Ordinary document movement resumes automatic outline tracking. Jump targets use the current view and do not activate source-line navigation.

At 72 columns or wider the panel reserves up to 36 columns and the pager reformats content within the remaining width. Narrow terminals use an overlay. Selection coordinates account for the document offset; full and selection redraws restore the panel. Search and source-line prompts close it before taking input. Reload preserves visibility and replaces entries atomically, clearing obsolete outline positions. The panel uses the same output-styling and color-depth policy as the pager, with a visible active marker even without colors.

## Watcher

`ActiveWatcher` watches the parent directory but compares the canonical or normalized event path with one target. `Modify` and `Create` events use a 100 ms debounce interval. Dropping the watcher sets a stop flag and joins its thread.

The refresh callback re-reads the document and prepares the selected line-number mode. Width-aware documents use `Pager::refresh_layout` after refresh or numbering changes; static content uses `Pager::set_mapped_text`. Both replace text and navigation atomically, preserve source position, and clear obsolete selection and navigation highlights. `set_mapped_text` always installs the supplied content; it does not invoke the layout callback. The new document inherits the active warmup scheduler but uses a fresh view cache and delay.

## Editor

[src/editor.rs](../../src/editor.rs) resolves commands from `MDV_EDITOR` or `EDITOR`, classifies terminal, GUI, Vim, and GVim editors, and builds platform-correct arguments.

- The browser opens an editor after suspending the terminal session.
- The pager exits, runs the editor, and may refresh the file when the editor returns.
- An unknown or invalid command is an error; no hidden editor is selected.

## Invariants

`auto` resolves from stdout before entering TUI. In `never`, browser decoration,
footer/help styling, query-selection inversion, search/selection backgrounds,
and source-line highlights are disabled. Search, selection-aware copying,
source navigation, numbering changes, and refresh remain functional. Cursor,
screen-buffer, mouse, and terminal-cleanup commands remain active. The vendored
pager receives this decision through `Pager::set_output_styling`; environment
variables such as `NO_COLOR` cannot override enabled prompt/highlight rendering.

Footer and help builders only construct `PromptLine` layout and style metadata.
The pager applies the shared policy when rendering those lines, including narrow
help panels and refresh. There is no separate plain-footer layout or repeated
color flag in the builders and input classifier.

- The browser and pager never own raw terminal mode simultaneously.
- Every pause or suspension has a matching resume even after an operation fails.
- A watcher updates only the selected file.
- Background refresh does not hold a write lock while reading or rendering the file.
- File discovery never waits for a complete directory scan before publishing matching documents.
- Interactive state updates are coalesced, and terminal output never exceeds 120 frames per second.
- Unchanged browser frames produce no output; ordinary updates rewrite only changed rows.
- Transparent footer and help views do not set a background color.
