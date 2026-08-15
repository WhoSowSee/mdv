# Themes and Styling

The theme subsystem separates the semantic role of a Markdown element from terminal escape sequences. A renderer selects `ThemeElement`; `theme::create_style` then creates `AnsiStyle` from the active theme and inline-style overrides.

## Theme files

| File | Responsibility |
|---|---|
| [src/theme.rs](../../src/theme.rs) | Public re-exports and module facade. |
| [src/theme/types.rs](../../src/theme/types.rs) | `Theme`, `SyntaxTheme`, and the default palette. |
| [src/theme/colors.rs](../../src/theme/colors.rs) | Independent `Color` type and conversion to `crossterm::Color`. |
| [src/theme/element.rs](../../src/theme/element.rs) | `ThemeElement` roles for text, headings, links, tables, and other semantics. |
| [src/theme/builtin.rs](../../src/theme/builtin.rs) | Embedded YAML themes and lazy `BUILTIN_THEMES`. |
| [src/theme/manager.rs](../../src/theme/manager.rs) | Theme lookup, insertion, file loading, and luminosity sorting. |
| [src/theme/color_parse.rs](../../src/theme/color_parse.rs) | Named, hex, RGB, and ANSI-index parsing plus luminosity. |
| [src/theme/overrides.rs](../../src/theme/overrides.rs) | `key=value` overrides for terminal and syntax themes. |
| [src/theme/display.rs](../../src/theme/display.rs) | Theme listings and `ThemeElement` → `AnsiStyle` conversion. |

## Embedded themes

`builtin.rs` embeds YAML with `include_str!` for:

- `terminal`;
- `monokai`;
- `solarized-dark`;
- `nord`;
- `tokyonight`;
- `kanagawa`;
- `gruvbox`;
- `material-ocean`;
- `catppuccin`.

The name inside each YAML file is checked against its embedded-resource name. An invalid embedded theme is a packaged-resource defect and panics while the lazy map initializes.

## `Theme`

A complete theme contains:

- general `text` and `text_light` colors plus independent line-number colors;
- `h1` through `h6`;
- code, quote, link, and inline semantic colors;
- optional foreground and background for combined inline styles;
- document background and border colors;
- list, table, error, and warning colors;
- `SyntaxTheme` for code highlighting;
- `pager_status_bar_transparent`.

`ThemeElement` is the stable semantic key. Add a new visual role to this enum and `create_style` instead of inserting hard-coded ANSI sequences in an event handler.

## User themes

| File | Responsibility |
|---|---|
| [src/user_themes.rs](../../src/user_themes.rs) | Paths under `themes/*.yaml` and the public loader. |
| [src/user_themes/schema.rs](../../src/user_themes/schema.rs) | Partial `ThemeFile`, `SyntaxFile`, color deserialization, and inheritance. |
| [src/user_themes/loading.rs](../../src/user_themes/loading.rs) | File sorting, `extends`, loading, and diagnosable skipping of invalid files. |

User themes are read from `<config_dir>/themes/*.yaml|*.yml` in lexical order. `extends` may refer to an embedded theme or an already loaded user theme. Unspecified fields inherit from the base; without `extends`, the base is `Theme::default()`.

Unlike the partial user schema, an embedded theme must define every required color, its description, syntax palette, and status-bar transparency flag.

## Application order

`TerminalRenderer::new` applies styling in this order:

1. embedded themes;
2. user YAML themes;
3. the selected terminal `theme`;
4. `custom_theme`;
5. `inline_style` overrides;
6. the selected or generated code theme;
7. `custom_code_theme`.

Terminal and code themes may be different.

## Color formats

`parse_color_value` accepts:

- named terminal colors;
- hexadecimal RGB (`#rrggbb`);
- RGB components;
- an indexed ANSI value;
- `none` or reset where a field is optional.

An unknown name, invalid component count, or out-of-range component returns an error containing the override key.

## Low-level ANSI output

[src/terminal.rs](../../src/terminal.rs) contains `AnsiStyle` and color helpers.

`AnsiStyle` accumulates foreground, background, bold, italic, underline, and strikethrough. `apply` emits one coherent escape sequence. With `no_colors`, it returns the original text without escape codes.

`ansi256_to_rgb` and `calculate_luminosity` support theme comparison and sorting; they do not rewrite user values.

## Inline styles

[src/inline_style.rs](../../src/inline_style.rs) defines:

- `InlineStyleKind`: emphasis, strong, combined strong-emphasis, code, strikethrough, and highlight;
- `InlineStyle`: resolved foreground, background, and attributes;
- `InlineStyleOverride`: a partial override;
- `InlineStyleOverrides`: the user map;
- `InlineStyleSet`: the fully resolved theme set.

Partial YAML retains semantic defaults. Duplicate properties in a string override are rejected so ordering cannot change the result implicitly.

## Callouts, lists, checkboxes, and code labels

Visual extensions are parsed before rendering:

- [src/callout.rs](../../src/callout.rs) — custom callout icon and color;
- [src/list_marker.rs](../../src/list_marker.rs) — pretty, uniform, and per-level list markers;
- [src/checkbox.rs](../../src/checkbox.rs) — standard square and circle icons;
- [src/checkbox_override.rs](../../src/checkbox_override.rs) — custom checkbox states;
- [src/custom_code_block.rs](../../src/custom_code_block.rs) — code label, icon, and aliases.

The renderer receives compiled maps in `Config` and never repeats string parsing.

## Invariants

- `no_colors` disables ANSI but does not alter structural symbols or icons.
- A custom callout with an existing name changes presentation while type semantics and default-icon resolution remain predictable.
- A user theme cannot silently introduce an unknown key.
- A `None` background means no background sequence, not black.
- Width helpers remove escape sequences before measurement.
