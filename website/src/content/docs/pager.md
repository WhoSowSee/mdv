---
title: "Pager and editor"
description: "Complete navigation, search, outline, selection, refresh and editor controls."
slug: "pager"
group: "Reading"
order: 11
---

## Open the built-in pager

```sh
mdv --pager README.md
```

A bare `--pager` uses `MDV_PAGER` when it is set, otherwise the default backend. Select the built-in backend explicitly with:

```sh
mdv --pager=builtin README.md
```

<div data-mdv-preview="builtin-pager"></div>

## Find your way around

| Key | Action |
| --- | --- |
| `?` | Open or close help. |
| `/` or `Ctrl+F` | Search the document. |
| `t` | Toggle the table of contents. |
| `:` | Jump to a Markdown source line. |
| `l` | Cycle line-number modes. |
| `r` | Refresh the current file. |
| `Esc` | Close the current panel or mode before exiting. |
| `k` / `Up`, `j` / `Down` | One row up/down. |
| `b` / `PageUp`, `f` / `PageDown` | One page up/down. |
| `u` / `Ctrl+U`, `d` / `Ctrl+D` | Half a page up/down. |
| `g` / `Home`, `G` / `End` | Top/bottom of the document. |
| `h` / `Left`, `Right` | Horizontal scrolling; `l` is reserved for line numbers in mdv. |
| `n`, `N` / `p` | Next/previous search occurrence. |
| `q` | Exit the pager. |

The help panel shows the available controls for the current view.

## Copy text

Select rendered text, then use `c`, `Ctrl+C` or right-click to copy it. `Ctrl+A` selects the whole rendered document, including content outside the viewport.

Copied text excludes terminal color and hyperlink escape sequences.

## Open your editor

Press `e` to open the current file in your editor. Set `MDV_EDITOR` or `EDITOR` to choose the editor command; `MDV_EDITOR` takes priority.

File actions are available for file-backed documents. A document read from standard input has no source file to open or refresh.

## Use an external pager

```sh
mdv --pager="less -R" README.md
```

An external pager receives rendered content and uses its own controls. Color-preserving options such as `less -R` belong to the external pager command.

## Search behavior

Search previews matches while typing.

- Reopening it restores and selects the previous query.
- Esc closes the prompt while retaining the query and previewed position; outside the prompt, Esc clears search highlighting before exiting on a later Esc.
- Explicitly clearing the query prevents it from being restored.
- Drag with the left mouse button to select text, then copy; without a nonempty selection, copy leaves the clipboard unchanged.

## Table of contents

`t` toggles the outline.

- A unique top-level title is omitted; the next two distinct heading levels form sections and subsections.
- `J`/`K` or Shift+Down/Up move between sections.
- Alt+Up/Down selects adjacent entries.
- `1` through `9` select or cycle section groups.
- `D`/`U` or Alt+Shift+Down/Up scroll the outline independently.
- Clicking selects an entry; the wheel over the outline scrolls it.
- At 72 terminal columns or more the outline reserves space beside the document, otherwise it overlays the document.
- Search and source-line prompts close it.

## Reloading and source positions

`:` accepts a one-based physical Markdown line and highlights the target.

- Esc restores the numbering view selected by `l`.
- A file-backed document can be refreshed with `r`; source changes are also watched in the built-in view.
- Reload preserves numbering and outline visibility while clearing obsolete selections and navigation highlights.
- A failed reload keeps the current document.

## Editor modes and backend selection

`MDV_EDITOR` takes precedence over `EDITOR`.

- Known GUI editors launch asynchronously; terminal editors wait for exit.
- `MDV_EDITOR_MODE=tui` forces waiting, while `gui` allows asynchronous launch for unknown commands but is rejected for known terminal editors.
- Invalid quoting, modes or commands report an error without launching.
- After editor return the document can refresh.
- [Environment variables](../environment/) describe these controls in full.

The backend is selected in order: an explicit `--pager=COMMAND`, then `MDV_PAGER`, then the built-in pager.

- `default` selects the default backend (currently built-in), and `builtin` explicitly selects it.
- `MDV_PAGER` alone does not enable paging for ordinary output.
- Empty, malformed or recursive mdv commands are errors, even when a CLI backend overrides the environment.
- External commands run directly without shell expansion and need their own color-preserving arguments.
- Redirected stdout disables paging.
- `--color never` disables decorative colors and hyperlinks while retaining navigation, selection, copying and refresh.
