# Interactive Mode and Pager

The interactive subsystem has two levels: a Markdown document browser and a pager for one document. Ordinary `--pager` uses the same pager without the browser UI.

## Target selection

[src/interactive/mod.rs](../../src/interactive/mod.rs) defines `InteractiveTarget`:

- `Directory(PathBuf)` opens the browser;
- `File(PathBuf)` opens the pager immediately;
- `Stdin` reads standard input and opens a pager without file actions.

`select_interactive_target` considers the filename, `--interactive`, `--pager`, and whether standard input is a terminal. An explicit pager takes precedence and bypasses interactive target selection.

With no filename and terminal standard input, the current directory opens automatically. Interactive mode requires terminal standard output.

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

Discovery runs independently and publishes each document or error through a bounded channel. `poll_discovery` consumes a limited number of events on every UI tick, inserts newly found documents into the sorted list, refreshes an active filter, and preserves the selected path while the list grows. The line spinner beside the logo appears only after a 16 ms grace period and starts from its first frame; a final event stops it. Fuzzy matching normalizes Unicode but returns indices into the original string so highlighting remains correct.

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
| [pager/document.rs](../../src/pager/document.rs) | `PagerDocument`, `RefreshCallback`, and `PagerScreen`. |
| [pager/page.rs](../../src/pager/page.rs) | Configure `minus::Pager` and run the pager/editor loop. |
| [pager/input.rs](../../src/pager/input.rs) | Custom input classifier for help, copy, reload, and editor actions. |
| [pager/footer.rs](../../src/pager/footer.rs) | Opaque/transparent footer, title, progress, and width clamping. |
| [pager/help.rs](../../src/pager/help.rs) | Prompt panel listing available shortcuts. |
| [pager/operations.rs](../../src/pager/operations.rs) | Document replacement, clipboard handling, and status/error messages. |
| [pager/watcher.rs](../../src/pager/watcher.rs) | `notify` watcher and debounced refresh. |

## `PagerDocument`

The document stores these values separately:

- `output`: rendered ANSI text;
- `source`: original Markdown for the clipboard;
- optional `title`;
- `status_bar_transparent` from the selected theme.

This separation is required: copying without a selection uses Markdown, while `pager.set_text` receives rendered output.

## Input classifier

The custom classifier extends the default `minus` classifier with:

- `?` to show or hide the help panel;
- `Esc` to close help without losing search state;
- `/` or `Ctrl+F` to search;
- `c` to copy a selection or the complete source;
- `r` to refresh when a callback exists;
- `e` to open the file in an editor when available.

When an active search has matches, the footer shows the current and total occurrences immediately before document progress. Both status values use the muted `#5a5a5a` foreground. Incremental search updates the matching viewport and highlights after every query edit, before confirmation. Search navigation and counting operate on individual occurrences, including multiple matches in one row, and only the exact current range receives the stronger tint. The viewport stays fixed while the next occurrence is visible; the first result below it is revealed on the bottom row instead of being moved to the top. Match highlighting preserves syntax foreground colors and derives each background tint from the active text color. Mouse selection remains available during search, preserves syntax colors over a neutral `#2e313b` background, and produces a lighter combined tint where selection overlaps a match.

Long clipboard and reload operations run on separate threads so pager input remains responsive. `reload_in_progress` prevents concurrent refreshes of one page.

## Watcher

`ActiveWatcher` watches the parent directory but compares the canonical or normalized event path with one target. `Modify` and `Create` events use a 100 ms debounce interval. Dropping the watcher sets a stop flag and joins its thread.

The refresh callback re-reads and re-renders the document, atomically replaces `RwLock<PagerDocument>`, and calls `pager.set_text`.

## Editor

[src/editor.rs](../../src/editor.rs) resolves commands from `MDV_EDITOR` or `EDITOR`, classifies terminal, GUI, Vim, and GVim editors, and builds platform-correct arguments.

- The browser opens an editor after suspending the terminal session.
- The pager exits, runs the editor, and may refresh the file when the editor returns.
- An unknown or invalid command is an error; no hidden editor is selected.

## Invariants

- The browser and pager never own raw terminal mode simultaneously.
- Every pause or suspension has a matching resume even after an operation fails.
- A watcher updates only the selected file.
- Background refresh does not hold a write lock while reading or rendering the file.
- File discovery never waits for a complete directory scan before publishing matching documents.
- Interactive state updates are coalesced, and terminal output never exceeds 120 frames per second.
- Unchanged browser frames produce no output; ordinary updates rewrite only changed rows.
- Transparent footer and help views do not set a background color.
