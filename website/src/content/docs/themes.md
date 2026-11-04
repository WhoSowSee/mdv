---
title: "Themes"
description: "Built-in palettes, custom YAML themes, inheritance and visual examples."
slug: "themes"
group: "Customization"
order: 30
---

## Select a theme

Built-in themes: `terminal`, `solarized-dark`, `nord`, `tokyonight`, `kanagawa`, `gruvbox`, `monokai`, `material-ocean`, `catppuccin`.

The default is `terminal`. Code follows the document palette unless `--code-theme` selects another palette. `--theme-info` shows the active colors; with a filename, it also renders that document.

<div data-mdv-preview="theme"></div>

<div data-mdv-preview="code-theme"></div>

## User themes

Drop one or more `*.yaml`/`*.yml` files into `<config_dir>/themes/` to register your own themes. The directory follows the same resolution order as `config.yaml` (the `--config-file` flag, `$MDV_CONFIG_PATH`, then the platform user configuration directory). Two ready-to-use examples ship with the repository:

- [`docs/examples/themes/theme-warm.yaml`](https://github.com/WhoSowSee/mdv/blob/main/docs/examples/themes/theme-warm.yaml) - uses `extends: monokai` to override just a handful of fields.
- [`docs/examples/themes/theme-custom.yaml`](https://github.com/WhoSowSee/mdv/blob/main/docs/examples/themes/theme-custom.yaml) - example of a standalone palette.

```yaml
# <config_dir>/themes/warm.yaml
name: warm
description: Warm red accent layered on top of monokai.
extends: monokai
pager:
  transparent: true

h1: "#ff5577"
link: "#66ccff"
background: "#1c1c1c"

inline_style:
  code:
    backticks: false
    underline: true

syntax:
  keyword: "#ff5577"
```

Theme file basics:

- `name` (required) - the value accepted by `--theme`.
- `description` (optional) - shown in `mdv --theme-info`; inherits from the base theme.
- `extends` (optional) - a built-in theme or a user theme loaded earlier in alphabetical order. Without it, missing fields inherit from `terminal`.
- `pager` - footer colors, transparency, selection, and search highlighting in the built-in pager.
- `inline_style` - inline Markdown decorations; colors belong to the corresponding color sections.

Colors inherit per field.

- Explicit `null` clears an inherited optional border or background color override; `reset` uses the terminal's default color.
- See the [field reference](../theme-reference/) and [inheritance rules](../theme-reference/) for the full schema and special cases.
- The [bundled theme files](https://github.com/WhoSowSee/mdv/blob/main/assets/config/themes/) show all fields; [terminal.yaml](https://github.com/WhoSowSee/mdv/blob/main/assets/config/themes/terminal.yaml) defines the default palette.

A user theme with the same name as a built-in takes precedence and fully replaces it, which is the supported way to fork a built-in without copying every field. Broken or unrecognized files are skipped with a warning, so a single bad theme does not break the rest.
