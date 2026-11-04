---
title: "Document browser"
description: "Directory discovery, fuzzy filtering, ignored files and all browser controls."
slug: "browser"
group: "Reading"
order: 10
---

## Open the browser

Browse the current directory:

```sh
mdv -i
```

Or choose a directory:

```sh
mdv docs/
```

With terminal input and output, running `mdv` without arguments also browses the current directory. Redirected output does not open the browser implicitly.

With a file or standard input, `--interactive` opens that document directly in the selected pager. Interactive output requires a terminal:

```sh
mdv -i README.md
cat notes.md | mdv -i -
```

<div data-mdv-preview="document-browser"></div>

## Find a document

The browser discovers Markdown files recursively and supports fuzzy filtering. Results appear as discovery progresses, so you can start looking for a document before the scan finishes.

Select a document to open it in a pager. The built-in pager provides search, copying, an outline and source-line navigation.

## Understand ignored files

Discovery respects hidden-file and ignore rules, including `.gitignore`.

Press `.` to toggle files excluded by Git ignore rules. Hidden paths, `.ignore` rules and `node_modules` remain excluded in both modes.

## Customize the view

```sh
mdv --theme nord docs/
mdv --preset reader -i
```

Themes also define the interactive interface palette. See [Themes](../themes/) and [Pager and editor](../pager/) for the next step.

## Keyboard and mouse controls

| Key | Action |
| --- | --- |
| `Enter` | Open the selected document. |
| `j` / `k`, `Down` / `Up` | Select the next/previous result. |
| `h` / `l`, `Left` / `Right`, `PageUp` / `PageDown` | Move one page backward/forward. |
| `g` / `Home`, `G` / `End` | First/last result. |
| `/`, `Ctrl+F` | Edit the fuzzy filter. |
| `Esc` | Cancel or clear filtering; close help first when it is open. |
| `Tab`, `Shift+Tab`, `H`, `L` | Move between result sections. |
| `.` | Toggle files excluded by Git ignore rules. |
| `e` | Open the selected file in the configured editor. |
| `r` | Rescan the directory. |
| `!` | Show discovery/opening errors. |
| `?` | Toggle help. |
| `q`, `Ctrl+C` | Exit. |

The mouse wheel changes the selected result.

- While editing a filter, type or paste text and use Backspace to remove characters.
- Enter confirms the filter; press Enter again to open the selected result.
- Tab, arrow keys and Ctrl+J/Ctrl+K also finish filter editing.
- Esc cancels the filter.
- The explicit `--interactive` flag cannot be combined with `--pager`; documents opened from the browser use the pager backend selected by `MDV_PAGER`.
