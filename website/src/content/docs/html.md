---
title: "HTML"
description: "HTML export and terminal rendering of embedded HTML fragments."
slug: "html"
group: "Formatting"
order: 27
---

## Export a document

`--html` writes HTML instead of terminal-formatted text. Redirect it to a file:

```sh
mdv --html README.md > readme.html
```

Terminal margins and terminal color-depth reduction do not apply to this export.

## Render embedded HTML in the terminal

`--render-html`, or `-E`, interprets raw HTML fragments in the Markdown source and renders their readable contents for the terminal. Without it, raw tags are displayed literally. This is a different operation from exporting HTML.

<div data-mdv-preview="render-html"></div>

Supported content includes headings, paragraphs, inline formatting, quotes, preformatted text, lists, definition lists, tables, details, figures and media descriptions. Nesting is limited to 64 DOM elements; deeper content returns an error.

## Whitespace and containers

Ordinary text collapses whitespace. `<pre>` and `<textarea>` preserve spaces and newlines. `<code>` receives inline-code styling. `<sup>` and `<sub>` use Unicode scripts when representable; `<mark>` uses highlight colors. `<pre>` can take language hints through language attributes or `language-*`/`lang-*` classes.

`<details>` displays its summary and body expanded, regardless of `open`; it does not create interactive folding. Nested details repeat their guide lines. Shared `block_spacing` settings apply to the semantic blocks inside them.

## Lists and tables

Ordered lists support `start`, `reversed`, `li[value]`, decimal, alphabetic and Roman markers.

- HTML tables use the same wrapping and border settings as Markdown tables.
- Block-level content in table cells requires HTML and `--render-html`; a GFM Markdown table alone supports inline cell content.

## Media, comments and controls

Media is represented by a type marker, label and source path or URL; it is not downloaded or displayed as an image.

- Inputs, buttons and selects are static representations.
- Unknown tags keep readable child content.
- Scripts and styles are never executed.

Comments retain their delimiters and whitespace unless `--hide-comments` is enabled.
