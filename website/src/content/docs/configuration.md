---
title: "Configuration"
description: "Configuration discovery, precedence and structured YAML overrides."
slug: "configuration"
group: "Reference"
order: 40
---

## Settings and precedence

mdv merges settings from several sources in the following order of precedence:

1. CLI options (highest priority).
2. `MDV_COLOR`, for the `color` setting only.
3. The preset selected with `--preset`.
4. The configuration loaded from `--config-file`, `MDV_CONFIG_PATH`, or the user-level directory.
5. Built-in defaults.

Create the default user config with `mdv --init-config`. Add a directory path (`mdv --init-config custom/dir`), use `--config-file <CONFIG_DIR>`, or set `MDV_CONFIG_PATH` to write it somewhere else.

Configuration files must be written in YAML (`.yaml` or `.yml`). See the [configuration template](../configuration-reference/) for all settings and examples:

```yaml
# docs/examples/config.yaml
theme: "terminal"
code_theme: null
inline_style: {}
wrap: "char"
table_wrap: "fit"
table_borders: false
heading_layout: "level"
smart_indent: false
block_spacing: null
code_wrap_indent: "double"
link_style: "clickable"
link_overflow: "wrap"
```

## Configuration paths

mdv checks `config.yaml` before `config.yml`. Candidates from `--config-file DIR` precede `MDV_CONFIG_PATH`; if neither is provided, it uses the platform's user configuration directory. Windows uses `%USERPROFILE%\.config\mdv`; Linux uses `$XDG_CONFIG_HOME/mdv` or `~/.config/mdv`; macOS uses `~/Library/Application Support/mdv`.

The first existing file is loaded.

- Invalid YAML or settings stop startup instead of trying a later file.
- `--init-config` creates a new file and refuses to overwrite an existing one.
- `--no-config` skips the main file but keeps the selected directory available for themes and presets.

```sh
mdv --init-config ./mdv-config
mdv --config-file ./mdv-config README.md
mdv --no-config README.md
```

## Merge and clearing rules

Configuration and preset files accept structured YAML mappings for `custom_theme`, `custom_code_theme`, `block_spacing`, `custom_callout`, `custom_code_block`, `custom_checkbox`, `custom_list` and `callout_style`. Their compact CLI string forms also remain valid in YAML.

| Setting | Merge behavior |
| --- | --- |
| Ordinary values | A supplied higher-priority value replaces the previous value, including `false`, default-valued options and `null`. |
| `custom_theme`, `custom_code_theme` | Merge colors by field. Within a source, syntax colors apply in order: `custom_theme`, `code_theme`, `custom_code_theme`. A higher source wins even across different override parameters. |
| `inline_style` | Merge by element and decoration property. Explicit `false` disables an attribute. |
| `custom_callout`, `custom_code_block`, `custom_checkbox`, `custom_list` | Merge by entry and property; omitted properties retain their previous values and source priority. |
| `block_spacing` | Configuration/preset mappings replace the earlier mapping; CLI entries merge only supplied elements and sides. |

Custom-entry `null` clears a property or entry; root `null` or `{}` clears the entire setting. `aliases` replaces the whole list when supplied; `aliases: []` clears it. Structured `callout_style` accepts `style`, `show_icons`, `show_simple_icons`, `fold_icons`, `label_inside` and `uppercase`.

```yaml
custom_callout:
  note:
    color: red
  tip: null
custom_code_block:
  python:
    aliases: []
inline_style:
  code:
    backticks: false
```

`MDV_COLOR` overrides the preset and configuration but an explicit `--color` overrides it.

- Environment values are validated before CLI overrides.
- Unknown structured properties, duplicates and malformed values are errors.
- [All keys and defaults](../configuration-reference/), [color inheritance](../theme-reference/) and [environment variables](../environment/) are documented on the site.
