# CLI and Configuration

CLI arguments and YAML converge into one `Config` value. Every downstream module receives this effective snapshot and does not inspect `ArgMatches` itself.

## Option parsing

`Cli` retains `Option<T>` for values that may have Clap defaults. `arg_has_user_value(matches, id)` determines whether a value came from the user or from Clap.

`-h` presents action summaries. Long help adds formats, value descriptions, constraints,
and examples. Compound-value examples include the option name and quote the value.
Descriptions of enum values live on the values themselves so Clap can generate the list.
`mdv help` shows the same long help through the selected pager when appropriate.

`--wrap` selects breakpoints for prose, code, callouts, and table cells. `--table-wrap` is
orthogonal: it controls whether table columns are fitted, split into blocks, or left unconstrained.

`--line-numbers` and `--code-line-numbers` share the same optional modes. Startup normalizes a
bare flag to rendered numbering unless it is followed by an explicit mode, keeping the following
Markdown path positional. The `--` separator and all following arguments pass through unchanged.

`--front-matter` controls an exact `---`-delimited YAML property mapping whose opening delimiter is the first line. `hidden` is the default; `panel` uses the callout renderer; `table` uses the two-column table renderer; `plain` emits key/value lines; `inline` emits one wrapping paragraph; `blocks` uses definition-list layout; `code` uses the YAML code-block path; and `source` bypasses front matter recognition so the complete input follows ordinary Markdown parsing.

## Precedence

The effective configuration is assembled in this order:

1. `Config::default()`.
2. The first existing configuration file, with errors propagated immediately.
3. The named `--preset`.
4. `MDV_COLOR`, when present and valid.
5. Explicit CLI arguments.
6. Terminal-theme and code-theme normalization.
7. Compilation of custom callouts, code blocks, checkboxes, and list markers.

Theme overrides merge per color across the configuration file, preset, and CLI. Within each source, `custom_theme` applies before `custom_code_theme`; syntax entries from a higher source override either parameter from a lower source. Unspecified colors are retained. An explicit preset `null` or empty mapping clears the corresponding inherited theme-override setting.

Custom callouts, code blocks, checkboxes, and list markers merge by entry and property, in configuration-file → preset → CLI order. Omitted entries and properties are retained. Each retained color keeps its original source priority, even when another property in its entry is changed. `aliases` is replaced as a whole only when present; an explicit empty list clears it.

Other settings use top-level replacement. A setting omitted from a preset retains the configuration-file value; a setting present in a preset replaces it, including explicit `false`, default-valued, `null`, and structured mapping values. Explicit CLI values then replace the preset value. `inline_style` is merged per property across its layers, while explicit CLI `block_spacing` entries merge only their specified elements and sides into the effective spacing value.

## Configuration discovery

`Config::get_config_paths` builds candidates in this order:

1. `--config-file DIR` → `DIR/config.yaml`, then `DIR/config.yml`;
2. `MDV_CONFIG_PATH` → the same two names;
3. the default configuration directory, only when neither a CLI nor environment path was supplied.

CLI candidates precede environment candidates when both are present. The first existing file is selected; read, YAML, and setting errors stop startup without trying another file or substituting defaults. Absent candidates allow discovery to continue. `--no-config` skips file loading but retains the derived `config_dir`, allowing user presets to be discovered there.

## Color policy

`Config.color` stores `ColorMode::{Auto, Always, Never}`. Clap exposes `auto` as
the help default for `--color`; configuration assembly applies it only when
`arg_has_user_value` confirms explicit input. Explicit `--color` overrides
`MDV_COLOR`, presets, configuration, and default `auto`.
Every loaded layer is validated before overrides are applied. Only exact lowercase
values are accepted; empty, padded, non-Unicode environment values and YAML null,
boolean, or numeric values are errors. An explicit `auto` replaces a lower mode.

Application routing resolves `OutputStyle` once from stdout TTY and passes it
separately to renderers and TUI. Configuration serialization retains the selected
mode and does not include resolved styling. Removed `no_colors` YAML keys are
errors in both configuration and presets; removed color environment variables are
ignored. Generic variables such as `NO_COLOR` do not influence this policy.

`OutputStyle` is defined in [src/terminal.rs](../../src/terminal.rs). The public
`cli::OutputStyle` re-export preserves the original library import path.

`color_depth` / `--color-depth` accepts `auto`, `16`, `256`, and `truecolor`.
YAML accepts 16 and 256 as numbers or strings. Its precedence is explicit CLI,
preset, configuration file, then `auto`; an explicit `auto` replaces a lower
forced value. It does not change the `color` policy and has no environment override.

[src/terminal/detection.rs](../../src/terminal/detection.rs) resolves the depth
once at startup, outside renderers. Explicit depth and disabled styling bypass
detection. Auto uses these rules:

- Basic terminal identities such as `linux`, `dumb`, and `vt100` use ANSI 16.
- Direct-color TERM identities and Unix terminfo Tc/true-color capabilities
  select True Color. Otherwise terminfo supplies the palette size; the generic
  RGB capability alone does not establish 24-bit support.
- Outside `screen`/`tmux`, `COLORTERM=truecolor|24bit` and known modern terminal
  identities can select True Color. `TERM_PROGRAM` and `WT_SESSION` are ignored
  in SSH sessions; multiplexer depth comes from its own TERM/terminfo rather
  than inherited outer-terminal variables. `TMUX` and `STY` also identify a
  multiplexer when its TERM name has been customized.
- Without a database entry, a `-256color`/`-256` TERM suffix selects ANSI 256.
- Insufficient evidence selects True Color, preserving the original colors.
  TTY detection controls whether styling is enabled, not this palette choice.

No terminal queries read stdin, and no `tput` process is launched. Terminfo is
read with the pure-Rust `terminfo` crate on Unix; absence of its database is
handled by the environment rules. Resolved depth travels with `OutputStyle`
through rendering, reloads, and browser/pager transitions. It is not serialized.

## Pager policy

Pager selection is runtime routing state rather than a YAML `Config` field. The
backend is resolved after configuration loading with this priority:

1. an explicit `--pager=<COMMAND>` value;
2. `MDV_PAGER`;
3. the built-in `minus` pager.

A bare `--pager` requests paging but contributes no backend value, so it uses
`MDV_PAGER` when present. `MDV_PAGER` selects the backend for explicit paging,
full help, and documents opened from the interactive browser; it does not enable
paging for ordinary output by itself. `default` selects the default backend,
currently the built-in `minus`, while `builtin` explicitly selects `minus`.

Like `MDV_COLOR`, the environment layer is validated before a higher-priority
CLI value is applied. Empty, malformed, non-Unicode, and recursive mdv commands
are errors even when `--pager=<COMMAND>` is present. External commands are split
with platform-appropriate quoting rules and launched directly without a shell.
Redirected stdout still suppresses paging. Color resolution remains independent:
the selected `OutputStyle` is applied before either backend receives the output,
and mdv does not add color-preserving flags to a user-supplied pager command.

## Runtime settings

Fields marked `#[serde(skip)]` are derived runtime data and do not appear in YAML.

`color_priorities` records the sources of theme colors, syntax overrides, code-theme selection, and individual callout/list/checkbox colors. Preset application preserves these origins across its serialization step. Renderers compare source priority before setting specificity. Within a source, syntax settings apply as `custom_theme` → `code_theme` → `custom_code_theme`.

Configuration fields are declared once for the public `Config` and a private
serde schema. The public `Deserialize` implementation validates removed keys
before delegating to that schema, so direct decoder calls cannot bypass validation.

## Width and margins

An explicit CLI `--cols` sets output width. Otherwise, a detected terminal width of at least
20 columns takes precedence over configured `cols`; if neither is available, width is 80.
Content width subtracts horizontal margins and the document line-number gutter.
The reserved width must be smaller than the total width.

Margins accept `left:N`, `right:N`, or both separated by `;`. Omitted sides are zero;
YAML `null` disables margins. HTML export does not apply terminal margins.

## Presets

[src/preset.rs](../../src/preset.rs) loads embedded presets from `assets/config/presets/` and user `presets/*.yaml` files under `config_dir`.

The embedded catalog contains `compact`, `pretty`, `reader`, and `showcase`. `pretty` adds frames and icons while preserving the selected themes.

- A user preset may replace an embedded preset with the same name.
- Two user files declaring the same name are an error.
- Files are sorted lexically for deterministic diagnostics.
- `PresetFile::apply_to` changes only fields present in the preset.

## Structured YAML settings

Setting parsers reject unknown keys, duplicates, and malformed values. Partial overrides
inherit unspecified properties from defaults, the theme, or a lower-priority layer.

`custom_theme`, `custom_code_theme`, `block_spacing`, `custom_callout`, `custom_code_block`, `custom_checkbox`, `custom_list`, and `callout_style` accept native YAML mappings in configuration and preset files. Their legacy scalar syntax remains accepted and is still used by CLI arguments. Deserialization preserves omission and explicit null separately until custom entries are merged and compiled.

- Theme mappings contain root scalar values or nested color sections. Their leaves normalize to colon paths such as `math:text` and `syntax:number`; unknown paths are rejected by the theme override parser.
- `block_spacing` maps block names to partial `top` and `bottom` values.
- Custom callouts map names to `icon` and `color`.
- Custom code blocks map language hints to `icon`, `label`, and an `aliases` sequence.
- Checkbox states and list levels map to optional `icon` and `color` values.
- Structured `callout_style` uses `style`, `show_icons`, `show_simple_icons`, `fold_icons`, `label_inside`, and `uppercase`.

For the four custom-entry settings, YAML `null` removes an individual property or entry override, restoring the corresponding theme or built-in value. Root `null` or `{}` clears the entire setting. CLI uses `note:color=null`, `note:null`, or `--custom-callout null` for the corresponding operations. Checkboxes and list levels support named `icon`/`color` properties alongside their legacy positional forms. Use `aliases: []` in YAML or `aliases=[]` in CLI to remove inherited code-block aliases.

An empty override mapping clears that setting when it appears in a higher-priority preset. See [docs/examples/config.yaml](../examples/config.yaml) for canonical examples.

## YAML compatibility

The reference is [docs/examples/config.yaml](../examples/config.yaml). When `Config` changes, verify that:

- existing keys still deserialize, or are rejected by an explicit compatibility test;
- structured and legacy scalar forms produce the same runtime behavior;
- preset and CLI overrides follow the setting-specific merge rules in [Precedence](#precedence);
- new runtime-only fields use `serde(skip)`;
- a CLI override is not activated solely by a Clap default;
- `MDV_COLOR` accepts exactly `auto`, `always`, or `never` and overrides configuration files and presets.
