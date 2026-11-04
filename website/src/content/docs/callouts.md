---
title: "Callouts"
description: "Callout types, styles, icons, custom definitions and YAML forms."
slug: "callouts"
group: "Formatting"
order: 23
---

## Style and options

- `-C, --callout-style <pretty|simple>[:show-icons;show-simple-icons;fold-icons;label-inside;uppercase]` - sets the callout layout and label behavior. `show-icons` uses Nerd Font glyphs; `show-simple-icons` uses portable ASCII markers such as `[i]`, `[*]`, `[!]`, and `[x]`. The two icon options cannot be combined.
  `label-inside` is only supported with `pretty`, `fold-icons` requires `show-icons`.
- `--custom-callout <name:icon=...,color=...;...>` - overrides or adds callout labels.
  `icon` and `color` are optional; color formats match `--custom-theme`.
- A Nerd Font is required in the terminal for icons enabled by `show-icons` and `fold-icons` to render correctly. `show-simple-icons` does not require a Nerd Font.

<div data-mdv-preview="callout-style"></div>

<div data-mdv-preview="callout-icons"></div>

<div data-mdv-preview="callout-label"></div>

<div data-mdv-preview="callout-uppercase"></div>

## Markdown input

```markdown
> [!NOTE]
> Keep related information together.

> [!WARNING]
> Check the selected destination before continuing.
```

Built-in types are `note`, `abstract`, `info`, `todo`, `tip`, `success`, `question`, `warning`, `failure`, `danger`, `bug`, `example` and `quote`. Fold indicators describe presentation; terminal content remains visible.

<div data-mdv-preview="callout-fold"></div>

## Examples

<div data-mdv-preview="custom-callout"></div>

## YAML and color precedence

```yaml
callout_style:
  style: pretty
  show_simple_icons: true
  label_inside: true
  uppercase: false
custom_callout:
  tip:
    icon: "*"
    color: green
  custom:
    color: "#89b4fa"
```

Custom definitions merge by name and property. A color-only override keeps the icon. `null` clears a property or entry, and root `null` or `{}` clears the setting. CLI also accepts `tip:color=null`, `tip:null` and `--custom-callout null`.

At the same source priority, a type's custom color wins over the common `callout.label`, then `callout.palette`. Higher-priority CLI colors beat lower-priority configuration colors. Frames use the independent `callout.border`; see the [theme field reference](../theme-reference/).
