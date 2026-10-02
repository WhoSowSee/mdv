# Themes and Styling

The theme subsystem separates the semantic role of a Markdown element from terminal escape sequences. A renderer selects `ThemeElement`; `theme::create_style` then creates `AnsiStyle` from the active theme and inline-style overrides.

## Theme files

| File | Responsibility |
|---|---|
| [src/theme.rs](../../src/theme.rs) | Public re-exports and module facade. |
| [src/theme/types.rs](../../src/theme/types.rs) | `Theme`, `SyntaxTheme`, and inline colors. |
| [src/theme/pager.rs](../../src/theme/pager.rs) | Pager footer, selection, and search colors plus transparency. |
| [src/theme/sections.rs](../../src/theme/sections.rs) | Typed color groups for inline foreground/background pairs, numbers, tables, math, metadata, labels, lists, and tasks. |
| [src/theme/colors.rs](../../src/theme/colors.rs) | Independent `Color` type and conversion to `crossterm::Color`. |
| [src/theme/element.rs](../../src/theme/element.rs) | `ThemeElement` roles for text, headings, links, tables, and other semantics. |
| [src/theme/builtin.rs](../../src/theme/builtin.rs) | Embedded YAML themes and lazy `BUILTIN_THEMES`. |
| [src/theme/manager.rs](../../src/theme/manager.rs) | Theme lookup, insertion, file loading, and luminosity sorting. |
| [src/theme/color_parse.rs](../../src/theme/color_parse.rs) | Named, hex, RGB, and ANSI-index parsing plus luminosity. |
| [src/theme/overrides.rs](../../src/theme/overrides.rs) | `key=value` overrides for terminal and syntax themes. |
| [src/theme/overrides/merging.rs](../../src/theme/overrides/merging.rs) | Merge color assignments by source across both theme-override settings. |
| [src/theme/priority.rs](../../src/theme/priority.rs) | Color-source priorities and syntax override origins. |
| [src/theme/syntax.rs](../../src/theme/syntax.rs) | Shared syntax field parsing and color access. |
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

Every bundled theme explicitly lists the full schema, including all optional colors and all five attributes for each of the six `inline_style` elements. The meaning of null colors and attributes is described in [Inheritance and null](#inheritance-and-null).

The name inside each YAML file is checked against its embedded-resource name. An invalid embedded theme is a packaged-resource defect and panics while the lazy map initializes.

## `Theme`

A complete theme contains:

- general `text` and `text_light` colors plus independent line-number colors;
- `h1` through `h6`;
- independent code, math content, math border, quote, link, and inline semantic colors;
- optional foreground and background for combined inline styles;
- document background, optional structural border colors, horizontal-rule and footnote-separator colors, and independent front matter colors;
- list, table, error, and warning colors;
- `SyntaxTheme` for code highlighting;
- `pager`.

`ThemeElement` is the stable semantic key. Add a new visual role to this enum and `create_style` instead of inserting hard-coded ANSI sequences in an event handler.

## User themes

| File | Responsibility |
|---|---|
| [src/user_themes.rs](../../src/user_themes.rs) | Paths under `themes/*.yaml` and the public loader. |
| [src/user_themes/schema.rs](../../src/user_themes/schema.rs) | Partial root schema and inheritance. |
| [src/user_themes/groups.rs](../../src/user_themes/groups.rs) | Partial nested color groups and inheritance. |
| [src/user_themes/inline.rs](../../src/user_themes/inline.rs) | Partial inline foreground/background groups and embedded validation. |
| [src/user_themes/pager.rs](../../src/user_themes/pager.rs) | Pager palette inheritance and explicit color clearing. |
| [src/user_themes/colors.rs](../../src/user_themes/colors.rs) | Color parsing and explicit-null handling. |
| [src/user_themes/complete.rs](../../src/user_themes/complete.rs) | Strict embedded-theme validation and construction. |
| [src/user_themes/loading.rs](../../src/user_themes/loading.rs) | File sorting, `extends`, loading, and diagnosable skipping of invalid files. |

User themes are read from `<config_dir>/themes/*.yaml|*.yml` in lexical order. `extends` may refer to an embedded theme or an already loaded user theme. Unspecified fields inherit from the base; without `extends`, the base is `Theme::default()`.

Unlike the partial user schema, an embedded theme must define every required color, its description, syntax palette, and status-bar transparency flag.

Errors while enumerating individual directory entries propagate with directory
context; they are distinct from the documented warning-and-skip policy for an
invalid theme file. Theme-name getters return borrowed `&str` values.

## Theme field reference

Theme files require `name`, which is accepted by `--theme`. Optional `description` is shown by `--theme-info` and inherits from the base theme. `extends` selects that base. See the [partial theme example](../examples/themes/theme-warm.yaml), [standalone palette example](../examples/themes/theme-custom.yaml), and [complete bundled themes](../../assets/config/themes/). The [terminal theme](../../assets/config/themes/terminal.yaml) defines the default colors.

| Root field or section | Fields | Purpose |
|---|---|---|
| Root foregrounds | `text`, `text_light`, `h1` through `h6`, `quote`, `link`, `error`, `warning` | Document text, muted labels, headings, ordinary quotes, links, and diagnostics. |
| Root optional colors | `background`, `details_border`, `horizontal_rule`, `footnote_separator` | Document background, details lines, rules, and footnote separators. |
| `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight` | `text`, `background` | Inline Markdown colors; decorations use `inline_style`. |
| `line_number` | `number`, `separator` | Source-line numbers and their separator. |
| `table` | `header`, `border` | Header text and all table lines, including wrapped-block separators. `Block…` labels use `text_light`. |
| `math` | `text`, `border` | Formula content and block frames. |
| `front_matter` | `title`, `key`, `value`, `border` | Properties-panel heading, keys, values, and frame. |
| `code_block` | `label`, `border` | Code-block title and icon together, and frame. |
| `callout` | `label`, `border`, `palette` | Combined title/icon/fold-marker color, frame, and type palette. |
| `callout.palette` | `note`, `abstract`, `info`, `todo`, `tip`, `success`, `question`, `warning`, `failure`, `danger`, `bug`, `example`, `quote` | Independent colors for each callout type. |
| `list` | `ordered`, `unordered` | Numbered and unnumbered list markers. |
| `todo` | `checked`, `unchecked` | Checkbox colors by state. |
| `syntax` | `keyword`, `string`, `comment`, `number`, `operator`, `function`, `variable`, `type_name` | Code syntax highlighting. |
| `inline_style` | See [Inline styles](#inline-styles). | Inline decorations, separate from colors. |
| `pager` | See [Pager palette](#pager-palette). | Footer, Help panel, selection, and search styles. |

The global `border`, former `details` section, and flattened names such as `code_background`, `math_border`, `table_header`, and `line_number_separator` are rejected. Math borders use their own color without a generic border fallback.

`--custom-theme` uses colon-separated paths: `line_number:number=grey`, `syntax:number=magenta`, or `callout:palette:note=blue`. Root colors use `key=value`; assignments are separated by semicolons. Configuration and preset YAML mappings nest the same sections, for example:

```yaml
custom_theme:
  callout:
    palette:
      note: blue
```

`--custom-code-theme` accepts both bare syntax keys (`keyword=blue`) and `syntax:` paths (`syntax:keyword=blue`).

## Inheritance and null

Omitted theme fields inherit individually, including within partial sections. Explicit `null` depends on the field:

| Field | `null` in a theme file | `null` or `none` in color overrides |
|---|---|---|
| Borders, document/inline optional backgrounds, `callout.label`, pager footer foregrounds | Clears the inherited color override. | Clears the color override. |
| `strong_emphasis.text`, `highlight.text` | Retains the base theme's foreground. | Clears the separate foreground. Strong emphasis then uses `strong.text`; Markdown highlight keeps the surrounding foreground. |
| Required foregrounds and `highlight.background` | Retains the base value. | Rejected; use `reset` for the terminal's default color. |
| `pager.selection`, `pager.search`, `pager.search_current` colors | Clears the override, restoring fixed selection or adaptive search colors. | Same as the theme file. |

`highlight.background` is the required background for Markdown `==text==`; it is independent of pager selection and search backgrounds. Other optional inline backgrounds can be cleared in both theme files and overrides. An uncolored border still renders its structural symbols.

`reset` explicitly selects the terminal's default foreground or background. It emits a reset sequence, unlike a cleared optional color, which emits no color sequence. For YAML `inline_style` attributes, omitted values and `null` retain inherited or semantic defaults; `false` explicitly disables an attribute.

## Application order

Color assignments are combined in source order: selected theme, configuration file, preset, then CLI. Within each source, syntax settings apply as `custom_theme` → `code_theme` → `custom_code_theme`. Partial overrides retain unspecified colors, including across the two parameters. A CLI `custom_theme` syntax entry overrides a configuration-file `custom_code_theme` entry for the same color. Explicit preset `null` or empty mappings clear the corresponding inherited override setting.

`merge_custom_theme_overrides` preserves assignment order while collecting general colors into `custom_theme` and syntax colors into `custom_code_theme`. Configuration assembly and preset application share this helper. `TerminalRenderer::new` loads the selected embedded or user theme and applies these assignments. Syntax overrides are retained or reapplied according to their source priority when the code palette is selected or generated.

Terminal and code themes may be different. Inline decorations apply over semantic defaults and the selected theme in the same configuration → preset → CLI order.

In `config.yaml` and preset files, `custom_theme` accepts nested YAML mappings or `section:field=value` strings; root fields use `key=value`. `custom_code_theme` accepts syntax paths, bare syntax keys, or YAML mappings. Mapping values may use the same named, hexadecimal, RGB, ANSI, boolean, and optional `null` forms as the corresponding string overrides. This changes only deserialization; application order remains the same.

## Color formats

`parse_color_value` accepts:

- named terminal colors, such as `red`, `darkgrey`, or `dark_grey`;
- hexadecimal RGB, such as `#ff5577`;
- comma-separated RGB components, such as `187,154,247`;
- indexed ANSI values, such as `ansi(42)` or `42`;
- `reset` for the terminal's default color.

Optional color overrides also accept `null` or `none`; theme files use YAML `null` as described in [Inheritance and null](#inheritance-and-null). An unknown name, invalid component count, or out-of-range component returns an error containing the override key.

## Low-level ANSI output

[src/terminal.rs](../../src/terminal.rs) owns `OutputStyle`, `AnsiStyle`, and color
helpers. Runtime consumers import `terminal::OutputStyle`; `cli::OutputStyle`
remains a public re-export for existing library callers. `ColorMode` remains a
CLI/configuration value and resolves to the terminal-owned runtime policy.

`AnsiStyle` accumulates foreground, background, bold, italic, underline, and strikethrough. `apply` emits one coherent escape sequence when the resolved `OutputStyle` is enabled and returns the original text when it is disabled.

`ansi256_to_rgb` is re-exported from the shared pager color module and supplies
the same indexed palette to theme handling and color reduction.
`calculate_luminosity` supports theme sorting; neither rewrites user values.

Color mode is independent of the selected palette. Application code resolves
`ColorMode` from stdout once and supplies `OutputStyle` to `AnsiStyle::apply`,
`TerminalRenderer`, and `TableRenderer`. Disabled styling also suppresses OSC 8
links, while all visible symbols and the selected link presentation remain.
This policy does not sanitize terminal controls supplied in the Markdown source.

`OutputStyle::Ansi16` and `Ansi256` additionally limit emitted color sequences;
`Enabled` retains the original colors for library callers. The shared converter
is `minus::ColorDepth::adapt` in `vendor/minus/src/color/`. It processes SGR
colors, preserving text, OSC payloads, attributes, and other controls. ANSI 16
preserves the nearest hue family in Oklab for chromatic colors, choosing its normal or
bright entry by weighted luma. Colors with HSV saturation below 25%, or all RGB
channels below 64, use the neutral ramp. This avoids collapsing pastel accents
to gray when comparing them with a conventional fully saturated ANSI palette.
ANSI indices 0–15 retain their meaning; RGB-to-256 conversion uses weighted RGB
distance to the fixed cube and grayscale entries 16–255.
ANSI 16 uses basic/bright foreground and background codes, including for table
headers emitted by `comfy-table`. Independent underline colors are omitted in
that mode. Distinct explicit foreground/background colors that collapse to one
entry receive a contrasting foreground; original resets restore their state.

The converter runs at style and complete-render boundaries, and on final pager
rows after dynamic highlighting. Reapplying it is idempotent. Themes retain
their original values; HTML export is unaffected. Reduced colors approximate a
conventional ANSI palette because the actual terminal palette is user-defined.

## Inline styles

[src/inline_style.rs](../../src/inline_style.rs) defines:

- `InlineStyleKind`: emphasis, strong, combined strong-emphasis, code, strikethrough, and highlight;
- `InlineStyle`: resolved foreground, background, and attributes;
- `InlineStyleOverride`: a partial override;
- `InlineStyleOverrides`: the user map;
- `InlineStyleSet`: the fully resolved theme set.

Each `inline_style` element accepts `backticks`, `bold`, `italic`, `underline`, and `strikethrough`. Colors remain in the matching theme section.

| Element | Markdown | Default decoration |
|---|---|---|
| `emphasis` | `*text*` or `_text_` | Italic. |
| `strong` | `**text**` | Bold. |
| `strong_emphasis` | `***text***` | Bold and italic. |
| `code` | Inline code | Surrounding backticks. |
| `strikethrough` | `~~text~~` | Strikethrough. |
| `highlight` | `==text==` | No extra decoration; colors come from `highlight`. |

Partial overrides merge per element and property. Duplicate properties in a string override are rejected so ordering cannot change the result implicitly.

## Callouts, lists, checkboxes, and code labels

Visual extensions are parsed before rendering:

- [src/callout.rs](../../src/callout.rs) - custom callout icon and color;
- [src/list_marker.rs](../../src/list_marker.rs) - pretty, uniform, and per-level list markers;
- [src/checkbox.rs](../../src/checkbox.rs) - standard square and circle icons;
- [src/checkbox_override.rs](../../src/checkbox_override.rs) - custom checkbox states;
- [src/custom_code_block.rs](../../src/custom_code_block.rs) - code label, icon, and aliases.

The main configuration and presets may express custom callouts, code blocks, checkbox states, and list levels as nested YAML mappings. CLI arguments retain their compact string syntax. These settings merge by entry and property across the configuration file, preset, and CLI; omitted values remain intact, explicit null clears an override, and an explicitly supplied aliases list replaces its earlier list.

Callout palettes are independent of ordinary quotes and other element colors. Color selection compares source priority before specificity. At equal source priority, per-type `custom_callout` colors win over `callout.label`, then `callout.palette`. Checkbox-state and list-level colors likewise win ties with the corresponding theme colors. An icon-only override retains the source priority of its inherited color. Clearing `callout.label` removes only the common override; type colors still apply. Frames use their own border fields. Code titles and icons share `code_block.label` in every layout.

See [Structured YAML settings](cli-configuration.md#structured-yaml-settings) for property, entry, and whole-setting clearing, named checkbox/list properties, and alias-list replacement. The renderer receives compiled maps in `Config` and never repeats string parsing.

## Invariants

- `color` preserves structural symbols and icons in every mode; `never` disables generated ANSI and OSC 8 styling.
- A custom callout with an existing name changes presentation while type semantics and default-icon resolution remain predictable.
- A user theme cannot silently introduce an unknown key.
- A `None` background means no background sequence, not black.
- Width helpers remove escape sequences before measurement.

## Pager palette

`pager` controls the built-in pager independently of Markdown `highlight` colors:

| Field | Controls |
|---|---|
| `transparent` | With `true`, removes footer and Help-panel backgrounds and separates footer sections with `\|`. |
| `title` | `MDV` title. |
| `help` | `? Help` and Help-panel text. |
| `file_name` | Document name and messages. |
| `progress` | Percentages and source-line indicator. |
| `matches` | Current/total search-match counter. |
| `selection.text`, `selection.background` | Selected document text. |
| `search.text`, `search.background` | Ordinary search matches. |
| `search_current.text`, `search_current.background` | Current match, including incremental search. |

All colors inherit per field; see [Inheritance and null](#inheritance-and-null) for clearing and terminal defaults. Footer foregrounds preserve the previous RGB defaults in all bundled themes. Color-depth limits and `--color never` apply to pager styles.

`pager.transparent` replaces the root `pager_status_bar_transparent` field; the root field and its former CLI alias are rejected. `PagerDocument` carries the resolved palette through document, help, reflow, and editor sessions. The vendored pager applies the highlight palette to final rows and incremental previews before color-depth conversion.
