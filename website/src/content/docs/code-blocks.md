---
title: "Code blocks"
description: "Block styles, labels, syntax detection, aliases, wrapping and line numbering."
slug: "code-blocks"
group: "Formatting"
order: 21
---

## Block styles

| Value of `--code-block-style` | Presentation |
| --- | --- |
| `basic` | Indented code without a border; the default. |
| `simple` | A single left gutter. |
| `pretty` | A box-drawn frame. |

Append `:show-name`, `:show-icon` or `:show-name;show-icon` to show a title. Both are hidden by default. Icons need a Nerd Font; plain language names do not.

<div data-mdv-preview="code-block-style"></div>

<div data-mdv-preview="code-labels"></div>

## Code line numbers

Numbering restarts for each block. A bare `--code-line-numbers` numbers rendered rows, including wrapped continuations. `source` numbers physical code lines and leaves continuation rows unnumbered. `separator` adds a separator. The options combine as `source;separator` and work in all three styles.

<div data-mdv-preview="code-line-numbers"></div>

## Wrapped-line indentation

`--wrap` chooses code breakpoints. `--code-wrap-indent` controls continuation rows: `none` adds no hanging indent, `base` preserves the original indentation, and `double` adds two spaces to the original indentation (the default). Gutters, frames, margins and enclosing lists reduce available width before wrapping.

<div data-mdv-preview="code-wrap-indent"></div>

## Language detection and syntax colors

mdv uses the language hint, custom aliases and built-in aliases before trying heuristic detection.

- `--no-code-guessing` keeps unknown blocks as plain text.
- Explicit plain/Markdown blocks use their own text path.
- [Custom syntax definitions](../syntaxes/) add new languages; [themes](../themes/) control code colors independently from document colors.

<div data-mdv-preview="code-guessing"></div>

## Custom icons, labels and aliases

`--custom-code-block` uses `language:icon=...,label=...,aliases=...` entries separated by `;`. Separate properties with commas and aliases with `|`. `default:icon=...` sets the unknown-language icon. These definitions change labels and lookup aliases, not the syntax grammar.

<div data-mdv-preview="custom-code-block"></div>

```yaml
custom_code_block:
  python:
    icon: "*"
    label: Python
    aliases: [py, py3]
  default:
    icon: "?"
```

Omitted properties merge with lower-priority settings.

- `null` clears a property or entry; root `null` or `{}` clears the entire custom setting.
- `aliases: []` or CLI `aliases=[]` clears inherited aliases.
- Code titles use `code_block.label`, and frames use `code_block.border` in the [theme schema](../theme-reference/).
