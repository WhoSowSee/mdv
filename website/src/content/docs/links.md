---
title: "Links and footnotes"
description: "All link styles, overflow modes, reference placement and missing footnotes."
slug: "links"
group: "Formatting"
order: 25
---

## Link presentation

| `--link-style` | Alias | Result |
| --- | --- | --- |
| `clickable` | `c` | Clickable text using OSC 8, without a visible URL; the default. |
| `fclickable` | `fc` | Underlined clickable text using OSC 8. |
| `inline` | `i` | The URL beside the link text. |
| `inlinetable` | `it` | Numbered links and a URL table after their owning text block or table. |
| `endtable` | `et` | Numbered links and a URL table at document end. |
| `hide` | `h` | Link text without its URL. |

OSC 8 links need terminal hyperlink support. `--color never` disables generated terminal styling and OSC 8; visible text and the selected presentation remain. `--color always` preserves styling through a pipeline.

<div data-mdv-preview="link-style"></div>

## URL overflow

`--link-overflow` accepts `wrap` (the default), `cut` (shorten long URLs with `...`), `tablecut` (shorten in normal text and table cells), or `none` (allow overflow). It changes the visible URL, never the hyperlink destination.

<div data-mdv-preview="link-overflow"></div>

## Footnote placement

`--footnote-style endnotes` collects definitions at document end; `attached` places them immediately after the referencing block, including paragraphs, tables and list items. Definitions follow first-use order.

```markdown
A statement with a source.[^source]

[^source]: Additional information.
```

<div data-mdv-preview="footnote-style"></div>

## Missing and empty definitions

`--missing-footnote-style show` displays placeholder entries for missing, invalid or empty definitions; it is the default. `hide` omits those entries from the footnote block. The optional theme color `footnote_separator` controls separators independently from Markdown rules.

<div data-mdv-preview="missing-footnote-style"></div>
