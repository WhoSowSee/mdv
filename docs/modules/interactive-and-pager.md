# Interactive Mode and Pager

The browser selects Markdown documents; the pager displays one document. Both use the
[shared document pipeline](architecture.md#document-flow). Implementation locations are
listed in the [file index](file-index.md#srcinteractive).

## Target selection

A directory opens the browser. A file or stdin selected through interactive mode opens
the pager directly; stdin has no file actions. With no arguments and terminal stdin and
stdout, mdv browses the current directory. Explicit `--pager` bypasses browser selection.
Interactive output requires terminal stdout.

## Browser discovery and rendering

Discovery publishes files incrementally so navigation and filtering remain available during
a scan. New results preserve the selected path. Fuzzy matching normalizes Unicode and maps
matches back to the original characters for highlighting. A delayed spinner avoids flashing
for short scans.

`.` toggles files excluded by `.gitignore`, the global Git ignore file, and `.git/info/exclude`.
The current results stay visible during rediscovery. Rapid toggles are debounced; hidden
paths, `.ignore`, and `node_modules` remain excluded in both modes.

Browser updates are coalesced at a maximum of 120 frames per second. Idle browsing blocks
on input. A frame is assembled in memory and written as one synchronized update; unchanged
frames produce no output, and ordinary updates rewrite only changed rows. Initial display,
resize, and resuming a session require a full redraw. Slow writes reduce the frame rate
without accumulating delayed frames.

## Pager backends and terminal ownership

The built-in backend provides mdv's search, copy, reload, editor, help, and line-number
controls. An external pager receives the initial rendered view and owns its own controls.
Selection and command validation follow [Pager policy](cli-configuration.md#pager-policy).

External commands run directly, receive document text on stdin, and inherit output streams.
mdv waits for successful completion and handles Ctrl+C in the parent while waiting. Commands
need their own color-preserving flags, such as `less -R`. Redirected stdout starts neither
backend.

The browser pauses for the built-in pager, which shares its terminal session. It fully
suspends for an external pager or editor, then restores the alternate screen and raw mode.
The cursor stays hidden during built-in pager handoff and return to the browser; returning
to the shell or launching an editor restores it. Every pause or suspension has a matching
resume, including error paths.

The vendored pager restores the preceding panic hook and resets its run mode after each
session, including errors and unwinding. Failed initialization permits retry. Setup, cleanup,
and event errors preserve their system cause through `std::error::Error::source()`.

## PagerDocument

`PagerDocument` keeps the source, title, styling, and source-line navigation together with
cached output views. Each layout can have unnumbered, rendered-numbered, and source-numbered
variants. The first screen prepares only the configured variant; switching views prepares
others on demand. Up to two layout widths are cached.

Cold views render outside the document lock. Installation checks the document identity and
numbering mode, discarding results from a document replaced during rendering. Refresh clears
the view cache and installs new source, output, and navigation atomically. A failed refresh
preserves the current document and reports the error.

Five seconds after the first screen, one background worker prepares rendered numbering and
then source numbering. Concurrent requests share active renders. Refresh and width changes
cancel pending work and restart the delay; ordinary view switches reuse the cache. Exit and
editor launch cancel the wait immediately. An active render may finish, but exit does not
wait for it. Preparation errors remain in the cache for the next on-demand request.

Markdown reflow uses a prepared renderer for the new width, including code/callout frames,
tables, margins, and gutters. Explicit CLI column limits remain an upper bound. Static pager
content uses ordinary pager wrapping.

## Selection and copying

Copy commands use selected rendered text with ANSI and OSC sequences removed. Without a
nonempty selection, they leave the clipboard and status unchanged. `Ctrl+A` selects the
whole rendered document, including rows outside the viewport. Clipboard operations run
outside the input thread.

Horizontal offsets are terminal display columns. Cropping preserves ANSI/OSC sequences,
the line-number gutter, and complete grapheme clusters. A left edge inside a grapheme
advances past it; a grapheme that cannot fit at the right edge is omitted. Search and
selection use the same cropped view. Text positions use `usize`; only final terminal
coordinates are bounded by screen dimensions.

## Input and help

| Key | Action |
|---|---|
| `?` | Toggle the help panel. |
| `/`, `Ctrl+F` | Open search. |
| `c`, `Ctrl+C`, right-click | Copy the selection. |
| `Ctrl+A` | Select the document. |
| `r` | Refresh a file-backed document. |
| `l` | Cycle off, rendered, and source line numbers. |
| `:` | Jump to a Markdown source line. |
| `e` | Open the configured editor. |
| `Esc` | Close the active panel or mode before exiting. |

Unavailable file or navigation actions are omitted from help. The panel uses three columns,
two below 78 columns, and one below 52 columns; resizing recomputes its layout. Shortcut keys
use the theme's help color and descriptions use the muted help-description color.

`l` uses mdv-rendered gutters and takes precedence over the pager's horizontal-scroll alias;
the right arrow remains available. `:` accepts a one-based source line and prepares the
source-numbered view on demand. The target stays highlighted with `:N` in the footer.
`Esc` restores the view selected by `l`; missing lines use the standard timed notification.

## Search

The search prompt restores and selects the previous query. Editing uses Unicode character
positions. Incremental results update the viewport before confirmation; `Esc` closes the
prompt while retaining the query and previewed viewport. Clearing the query explicitly
prevents it from being restored later. Outside the prompt, `Esc` clears highlights before
a subsequent `Esc` exits.

Navigation and the footer counter track individual occurrences, including several in one
row. The viewport remains fixed while the next occurrence is visible. Search and mouse
selection can overlap without losing syntax colors. Incremental previews use the same
highlight palette as confirmed results.

## Table of contents

The renderer extracts source-mapped headings from the processed document. A unique highest
level title is omitted; the next two distinct levels form sections and subsections.
`LineNavigation::with_toc` attaches the outline to mapped-text updates.

`t` toggles the panel. `J/K` navigate sections, `Alt+Up/Down` adjacent entries, and `1` through
`9` select or cycle section groups. `D/U` and the wheel over the panel scroll it. `Shift+Up/Down`
alias `K/J`; `Alt+Shift+Up/Down` alias `U/D` while the panel is open. Mouse clicks select
entries. Keyboard outline navigation dismisses the first-use footer hint for the session.

At 72 columns or wider, the panel reserves up to 36 columns; narrower terminals use an
overlay. Width changes reflow content and preserve source position and numbering. Outline-only
scrolling redraws the panel and footer. Search and source-line prompts close it before input.
Reload preserves visibility, replaces entries atomically, and clears obsolete positions.

## Watcher

The pager watches the selected file's parent directory and filters events to that file.
`Modify` and `Create` events are debounced for 100 ms. Refresh reads and renders outside
the write lock, retains the current numbering mode, and clears obsolete selection and
navigation highlights. Only one reload runs at a time. Dropping the watcher stops its loop.

## Editor

[src/editor.rs](../../src/editor.rs) resolves `MDV_EDITOR` before `EDITOR`. Known GUI editors
launch asynchronously; terminal editors wait for exit. Vim and Emacs variants use their
arguments to determine the mode. Unknown commands use terminal mode.

`MDV_EDITOR_MODE=tui` forces waiting. `gui` permits asynchronous launch for unknown commands,
but conflicts with known terminal editors. Empty values follow the editor selection rules;
malformed quoting, missing programs, non-Unicode values, and invalid modes produce contextual
errors without launching the editor. Returning from an editor can refresh the displayed file.

## Styling

The selected theme's [pager palette](themes-and-styling.md#pager-palette) controls the footer,
help, selection, and search. Transparent views set no background. Color depth applies to
final rows and dynamic highlights; external pagers receive converted document text and style
their own interface.

`--color never` disables decorative ANSI and OSC 8, including search and selection colors,
while retaining controls, copying, navigation, and refresh. Cursor, screen-buffer, mouse,
and cleanup sequences remain active. Styling is resolved before entering the interface and
passed to the pager through `set_output_styling`.
