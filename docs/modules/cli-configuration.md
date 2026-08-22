# CLI and Configuration

CLI arguments and YAML converge into one `Config` value. Every downstream module receives this effective snapshot and does not inspect `ArgMatches` itself.

## CLI files

| File | Types and responsibility |
|---|---|
| [src/cli.rs](../../src/cli.rs) | `Cli`: arguments, aliases, conflicts, help groups, and Clap defaults. |
| [src/cli/commands.rs](../../src/cli/commands.rs) | `CliCommand`, including full-format help. |
| [src/cli/layout.rs](../../src/cli/layout.rs) | `TextWrapMode`, `TableWrapMode`, and `HeadingLayout`. |
| [src/cli/links.rs](../../src/cli/links.rs) | `LinkStyle`, `LinkTruncationStyle`, `FootnoteStyle`, and `MissingFootnoteStyle`. |
| [src/cli/line_numbers.rs](../../src/cli/line_numbers.rs) | Shared `LineNumberOptions` and `LineNumberTarget` values for document and code-block gutters. |
| [src/cli/margins.rs](../../src/cli/margins.rs) | Parsing, serde support, and total width for `HorizontalMargins`. |
| [src/cli/callouts.rs](../../src/cli/callouts.rs) | Callout, checkbox, definition-list styles, and `CalloutStyleConfig`. |
| [src/cli/code_blocks.rs](../../src/cli/code_blocks.rs) | `CodeBlockStyleConfig`, `CodeBlockStyle`, and `CodeWrapIndent`. |
| [src/cli/help.rs](../../src/cli/help.rs) | Long help text kept outside the Clap structure. |

`Cli` retains `Option<T>` for values that may have Clap defaults. `arg_has_user_value(matches, id)` determines whether a value came from the user or from Clap.

## Argument groups

| Group | Examples | Consumer |
|---|---|---|
| Output and flow | `--pager`, `--interactive`, `--html`, `--render-html`, `--monitor`, `--reverse` | `lib::run`, `Config`, or an output adapter. |
| Layout and wrapping | `--cols`, `--margin`, `--wrap`, `--table-wrap`, `--heading-layout`, `--block-spacing` | Runtime layout and the event renderer. |
| Themes and code | `--theme`, `--code-theme`, `--code-block-style`, `--code-line-numbers`, `--syntaxes-dir` | Theme, syntax, and code-block rendering. |
| Callouts and lists | `--callout-style`, `--pretty-checkbox`, `--pretty-list`, custom overrides | Normalized maps and settings in `Config`. |
| Links and footnotes | `--link-style`, `--link-truncation`, footnote options | Link and footnote event handlers. |
| Configuration | `--config-file`, `--no-config`, `--preset`, `--init-config` | Configuration and preset loading. |

`--wrap` selects breakpoints for prose, code, callouts, and table cells. `--table-wrap` is
orthogonal: it controls whether table columns are fitted, split into blocks, or left unconstrained.

`--line-numbers` and `--code-line-numbers` share the same optional modes. Startup normalizes a
bare flag to rendered numbering unless it is followed by an explicit mode, keeping the following
Markdown path positional.

`--front-matter` controls an exact `---`-delimited YAML property mapping whose opening delimiter is the first line. `hidden` is the default; `panel` uses the callout renderer; `table` uses the two-column table renderer; `plain` emits key/value lines; `inline` emits one wrapping paragraph; `blocks` uses definition-list layout; `code` uses the YAML code-block path; and `source` bypasses front matter recognition so the complete input follows ordinary Markdown parsing.

## Configuration files

| File | Responsibility |
|---|---|
| [src/config.rs](../../src/config.rs) | `Config`, defaults, serde helpers, environment helpers, and the module facade. |
| [src/config/files.rs](../../src/config/files.rs) | Discover, read, and create `config.yaml` or `config.yml`. |
| [src/config/from_cli.rs](../../src/config/from_cli.rs) | Assemble `Config` from files, presets, environment, and explicit CLI input. |
| [src/config/merge.rs](../../src/config/merge.rs) | Overlay non-empty or non-default values from one configuration onto another. |
| [src/config/runtime.rs](../../src/config/runtime.rs) | Derived widths, wrapping flags, margin validation, and compiled overrides. |
| [src/config/structured.rs](../../src/config/structured.rs) | Deserialize structured YAML settings into the existing runtime formats. |

## Precedence

The effective configuration is assembled in this order:

1. `Config::default()`.
2. The first successfully loaded configuration file.
3. The named `--preset`.
4. `MDV_NO_COLOR`, when recognized.
5. Explicit CLI arguments.
6. Terminal-theme and code-theme normalization.
7. Compilation of custom callouts, code blocks, checkboxes, and list markers.

Later sources take precedence at the top-level setting key. A setting omitted from a preset retains the configuration-file value; a setting present in a preset replaces it, including explicit `false`, default-valued, `null`, and structured mapping values. Explicit CLI values then replace the preset value. `inline_style` is merged per property across its layers, while explicit CLI `block_spacing` entries merge only their specified elements and sides into the effective spacing value.

## Configuration discovery

`Config::get_config_paths` builds candidates in this order:

1. `--config-file DIR` → `DIR/config.yaml`, then `DIR/config.yml`;
2. `MDV_CONFIG_PATH` → the same two names;
3. the default configuration directory, only when neither a CLI nor environment path was supplied.

CLI candidates precede environment candidates when both are present. The first existing file that parses successfully is loaded. `--no-config` skips file loading but retains the derived `config_dir`, allowing user presets to be discovered there.

## `Config` field groups

- Display and layout: colors, width, margins, tabs, wrapping, headings, spacing, visibility, front matter, and document line numbers.
- Code, callouts, and lists: language guessing, custom syntaxes, styles, code-block line numbers, and compiled override maps.
- Themes: terminal theme, code theme, inline styles, and custom palette mappings or legacy strings.
- Links and footnotes: link presentation plus footnote placement and missing-definition behavior.
- Content filtering: `from_text` and reverse output.
- Runtime paths: the loaded `config_file` and its `config_dir`.

Fields marked `#[serde(skip)]` are derived runtime data and must not appear in YAML.

## Runtime helpers

`src/config/runtime.rs` centralizes calculations that must remain consistent across renderers:

- terminal and content width after margins;
- conversion from `TextWrapMode` to `utils::WrapMode`;
- source and rendered line-number activation;
- validation that margins leave usable content width;
- compilation of custom callout, code, checkbox, and list definitions.

## Presets

[src/preset.rs](../../src/preset.rs) loads embedded presets from `assets/config/presets/` and user `presets/*.yaml` files under `config_dir`.

- A user preset may replace an embedded preset with the same name.
- Two user files declaring the same name are an error.
- Files are sorted lexically for deterministic diagnostics.
- `PresetFile::apply_to` changes only fields present in the preset.

## Specialized setting parsers

| File | Purpose |
|---|---|
| [src/block_spacing.rs](../../src/block_spacing.rs) | Per-element blank lines and merging of partial overrides. |
| [src/callout.rs](../../src/callout.rs) | `name:icon=...,color=...` custom callout definitions. |
| [src/checkbox_override.rs](../../src/checkbox_override.rs) | One checkbox state with an optional icon and color. |
| [src/custom_code_block.rs](../../src/custom_code_block.rs) | Custom label, icon, and aliases for a code-block language. |
| [src/list_marker.rs](../../src/list_marker.rs) | Unicode or Nerd Font list styles, a uniform marker, and per-level overrides. |
| [src/inline_style.rs](../../src/inline_style.rs) | Semantic foreground, background, and attributes for inline elements. |
| [src/config/structured.rs](../../src/config/structured.rs) | YAML mappings for theme, callout, code-block, checkbox, and list overrides. |

These parsers reject unknown keys, duplicates, and malformed values. A valid partial override inherits remaining values from defaults or the active theme.

## Structured YAML settings

`custom_theme`, `custom_code_theme`, `block_spacing`, `custom_callout`, `custom_code_block`, `custom_checkbox`, `custom_list`, and `callout_style` accept native YAML mappings in configuration and preset files. Their legacy scalar syntax remains accepted and is still used by CLI arguments. Deserialization normalizes a mapping into the existing internal representation before runtime compilation, so the precedence and renderer contracts are unchanged.

- Theme mappings contain override keys and scalar color, boolean, numeric, or `null` values.
- `block_spacing` maps block names to partial `top` and `bottom` values.
- Custom callouts map names to `icon` and `color`.
- Custom code blocks map language hints to `icon`, `label`, and an `aliases` sequence.
- Checkbox states and list levels map to optional `icon` and `color` values.
- Structured `callout_style` uses `style`, `show_icons`, `show_simple_icons`, `fold_icons`, `label_inside`, and `uppercase`.

An empty override mapping clears that setting when it appears in a higher-priority preset. See [docs/examples/config.yaml](../examples/config.yaml) for canonical examples.

## YAML compatibility

The reference is [docs/examples/config.yaml](../examples/config.yaml). When `Config` changes, verify that:

- existing keys still deserialize, or are rejected by an explicit compatibility test;
- structured and legacy scalar forms produce the same runtime behavior;
- preset and explicit CLI priority remains field-based for structured settings;
- new runtime-only fields use `serde(skip)`;
- a CLI override is not activated solely by a Clap default;
- `MDV_NO_COLOR` retains its hard precedence over configuration files.
