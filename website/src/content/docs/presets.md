---
title: "Presets"
description: "Built-in reading profiles, custom presets and partial overrides."
slug: "presets"
group: "Customization"
order: 32
---

## Built-in and user presets

mdv ships with four presets:

- `reader` - word wrapping, paragraph reflow, attached footnotes, an inline link table, and the `nord` theme for long-form reading.
- `compact` - flat headings, simple code/callout blocks, hidden code labels, and end-of-document links for dense terminal output.
- `pretty` - framed tables and code blocks, with list and checkbox icons. Icons require a Nerd Font.
- `showcase` - centered headings, `tokyonight`, rich code/callout frames, checkboxes, and list icons. This preset requires a Nerd Font for its icons.

Select one with `mdv --preset reader README.md`. Explicit CLI options override the selected preset, so `mdv --preset compact --cols 100 README.md` uses the compact preset at 100 columns.

User presets are `*.yaml` or `*.yml` files under `<config_dir>/presets/`. The directory follows the same resolution order as `config.yaml`: `--config-file`, `MDV_CONFIG_PATH`, then the platform user configuration directory. It is optional, and `--no-config` does not disable it.

```yaml
# <config_dir>/presets/project-docs.yaml
name: project-docs
theme: monokai
wrap: word
reflow: true
table_wrap: wrap
link_style: inlinetable
inline_style:
  code:
    backticks: false
```

Every preset accepts the same keys and values as [`docs/examples/config.yaml`](../configuration-reference/), plus the required non-empty `name` field.

- Presets are partial layers: omitted settings remain unchanged, while explicit values-including `false`, default-valued options, and `null`-override the configuration beneath them.
- A user preset with the same name as a built-in replaces the built-in.
- Run `mdv --preset-info` to list the active catalog.
- When a file and preset are both provided, `mdv --preset-info FILE --preset NAME` renders the file with `Current preset: NAME`; without `--preset`, the flag does not alter file rendering.

<div data-mdv-preview="preset"></div>
