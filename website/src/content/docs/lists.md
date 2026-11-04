---
title: "Lists and checkboxes"
description: "Task states, nested list markers, definition markers and custom colors."
slug: "lists"
group: "Formatting"
order: 24
---

## Task-list checkboxes

- `-x, --checkbox-style <square|circle>` - renders task-list checkboxes as Nerd Font icons instead of the default `[ ]` / `[x]` markers. Requires a Nerd Font in the terminal.
- `--custom-checkbox <PAIRS>` - overrides built-in checkbox icons or adds new states (only with `--checkbox-style`). Each semicolon-separated pair is either `<char>:<icon>[:<color>]` or `<char>:<color>`. The color-only form keeps an existing icon or uses the default unchecked icon for a new state. Colors accept named, hex, rgb, and `ansi(N)` values.
  - Override:  `--custom-checkbox ' :!'`          replace the unchecked icon
  - Add:       `--custom-checkbox '*:*'`          add a new `[*]` checkbox state
  - Color:     `--custom-checkbox ' :!:yellow'`   or `#ff0000`, `128,1,1`, `ansi(200)`
  - Color only: `--custom-checkbox '?:red'` keeps the Nerd Font icon for the `?` state and applies red.
  - New state, color only: `--custom-checkbox '*:yellow'` uses the default unchecked icon and applies yellow.

<div data-mdv-preview="checkbox-style"></div>

<div data-mdv-preview="custom-checkbox"></div>

## List markers and definitions

- `-L, --list-style <style>` - replaces the default `-` unordered-list markers with a level-aware icon set. The value is required and uses the format `type:<nerd-font|unicode>;size:<large|small>`, for example `--list-style 'type:unicode;size:small'`.
  - `size` changes only Nerd Font markers. Unicode accepts either value but renders the same markers; their spacing may vary by font. Verified with Nerd Font families, especially JetBrainsMono Nerd Font.
- `-D, --definition-marker-style <unicode|nerd-font>` - renders definition descriptions with a built-in Unicode or Nerd Font marker.
- `--uniform-list-marker <level:1-4|icon:glyph>` - uses one marker at every nesting level (only with `--list-style`). `level:2` reuses level 2 from the selected set; `icon:*` uses a custom glyph.
- `--custom-list <level>:<icon>[:<color>];...` - overrides the marker icon and/or color for specific nesting levels (only with `--list-style`). Level is 1-based; icon is the marker glyph. Colors accept named (`red`), hex (`#ff0000`), rgb (`255,0,0`), and `ansi(N)` values.
  - Icon + color:  `--custom-list '1:*:yellow'`   marker `*` in yellow
  - Icon only:     `--custom-list '1:>'`          marker `>` in theme color
  - Color only:    `--custom-list '1:red'`        keep built-in icon, red color

<div data-mdv-preview="list-style"></div>

<div data-mdv-preview="uniform-list-marker"></div>

<div data-mdv-preview="custom-list"></div>

<div data-mdv-preview="definition-marker-style"></div>

## Structured overrides

```yaml
checkbox_style: square
custom_checkbox:
  " ":
    icon: "!"
    color: yellow
  x:
    color: green
list_style: "type:unicode;size:small"
uniform_list_marker: "level:2"
custom_list:
  1:
    icon: "*"
    color: yellow
  2:
    color: red
definition_marker_style: unicode
```

List levels are one-based.

- Named CLI forms such as `1:icon=*,color=yellow` are accepted alongside positional forms.
- Each entry/property merges with lower-priority configuration and preset values; explicit `null` clears a property or entry, and root `null` or `{}` clears the entire setting.
- Without `definition_marker_style`, definitions keep a two-space indentation.
- Numbered lists keep their numbering; marker customization targets unordered lists.
