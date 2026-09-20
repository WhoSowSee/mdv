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
| [pager/rendering.rs](../../src/pager/rendering.rs) | Build the three pager line-number views, prefixes, and source-line maps. |
| [pager/input.rs](../../src/pager/input.rs) | Custom input classifier for help, copy, reload, and editor actions. |
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
the built-in pager additionally prepares its three switchable views and source
navigation maps.

The built-in pager owns mdv-specific search, copy, reload, editor, help, and
line-number switching. An external pager owns its controls and receives only the
initial view selected by the effective line-number configuration.

When the browser opens an external pager, mdv fully suspends the browser session,
then restores the alternate screen and raw mode after the child exits. The
built-in pager uses the lighter in-place pause path because it shares mdv's
terminal session.

## `PagerDocument`

For the built-in backend, the document stores these values separately:

- unnumbered, rendered-numbered, and source-numbered ANSI views with their source-line maps, or one static output for non-Markdown pager content;
- the active line-number mode;
- `source`: original Markdown for the clipboard;
- optional `title`;
- `status_bar_transparent` from the selected theme.
- the resolved `OutputStyle` shared by document rendering and pager UI.

This separation is required: copying without a selection uses Markdown, while `pager.set_text` receives the active rendered view. Source-line navigation data is derived from that view and the prepared source-numbered view.

## Input classifier

The custom classifier extends the default `minus` classifier with:

- `?` to show or hide the help panel;
- `Esc` to close help without losing search state;
- `/` or `Ctrl+F` to search;
- `c` to copy a selection or the complete source;
- `r` to refresh when a callback exists;
- `l` to cycle the mdv line-number views;
- `:` to open the source-line navigation prompt when source metadata is available;
- `e` to open the file in an editor when available.

The help panel uses three columns in wide terminals, two below 78 columns, and one below 52 columns; resizing an open panel recomputes its layout. Entries are ordered by the visible width of the key combination; descriptions do not affect ordering. Optional entries are removed when line navigation, line-number switching, reload, or editor integration is unavailable.

`l` cycles `off → rendered → source → off`. The initial position comes from the effective `line_numbers` setting, so the first transition depends on how mdv was launched. When line-number switching is available, this mdv binding takes precedence over the default `minus` horizontal-scroll binding for `l`; the right arrow remains available for horizontal scrolling. The built-in `minus` gutter remains disabled; every numbered view is produced by `TerminalRenderer` and therefore uses the configured mdv colors, separator, margins, and wrapping.

The `:` prompt accepts a one-based Markdown source line. It swaps in the prepared source-numbered rendering when another line-number mode is active and reuses the current rendering in source mode. After a successful jump, that rendering and a fixed muted highlight remain active, while the footer shows `:N` immediately before document progress. `Esc` leaves line-navigation mode and restores the view selected by `l`. A missing line uses the same two-second status-message timeout as a search with no matches.

When an active search has matches, the footer shows the current and total occurrences immediately before document progress. Both status values use the muted `#5a5a5a` foreground. Incremental search updates the matching viewport and highlights after every query edit, before confirmation. Search navigation and counting operate on individual occurrences, including multiple matches in one row, and only the exact current range receives the stronger tint. The viewport stays fixed while the next occurrence is visible; the first result below it is revealed on the bottom row instead of being moved to the top. Match highlighting preserves syntax foreground colors and derives each background tint from the active text color. Mouse selection remains available during search, preserves syntax colors over a neutral `#2e313b` background, and produces a lighter combined tint where selection overlaps a match.

`Esc` closes the search prompt without restoring the pre-search viewport. If incremental search displayed a match, the pager remains at that displayed position while retaining the query for reuse.

Long clipboard and reload operations run on separate threads so pager input remains responsive. `reload_in_progress` prevents concurrent refreshes of one page.

## Watcher

`ActiveWatcher` watches the parent directory but compares the canonical or normalized event path with one target. `Modify` and `Create` events use a 100 ms debounce interval. Dropping the watcher sets a stop flag and joins its thread.

The refresh callback re-reads and re-renders all three views and preserves the selected line-number mode. Refresh and numbering changes use `Pager::set_mapped_text` to replace text and navigation in one command, preserve the source position, and clear obsolete selection and navigation highlights.

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
