---
title: "Complete CLI reference"
description: "Every command-line option, its format, accepted values and behavior."
slug: "cli"
group: "Reference"
order: 43
---

The defaults below describe a run with no configuration overrides. The [configuration guide](../configuration/) explains how YAML, presets, environment variables, and CLI options interact.


## Output and workflow

<section class="cli-option" aria-label="--html">

### `--html`

Exports the rendered document as HTML.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Terminal output</strong><br> Applies terminal layout and the active color policy.

<strong class="cli-example-label">Example</strong>

```sh
mdv --html example.md > example.html
```

</section>

<section class="cli-option" aria-label="--render-html">

### `--render-html`

<p class="cli-syntax"><code>-E, --render-html</code></p>

Renders raw HTML fragments as formatted content in terminal output.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Raw HTML tags are displayed literally.

HTML nesting is limited to 64 DOM elements.

<strong class="cli-example-label">Example</strong>

```sh
mdv --render-html example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="render-html"></div>

</details>

</section>

<section class="cli-option" aria-label="--line-numbers">

### `--line-numbers`

<p class="cli-syntax"><code>-N, --line-numbers [&lt;MODE&gt;]</code></p>

Adds line numbers to terminal and pager output.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Renders document rows without a line-number gutter.

- With no value, numbers rendered rows without a separator.
- `source` numbers physical Markdown source lines.
- `separator` numbers rendered rows and adds a separator.
- `source;separator` numbers source lines and adds a separator.

<strong class="cli-example-label">Example</strong>

```sh
mdv --line-numbers 'source;separator' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--color">

### `--color`

<p class="cli-syntax"><code>--color &lt;WHEN&gt;</code></p>

Controls ANSI styling and OSC 8 hyperlinks.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`auto`</strong><br> Enables terminal styling and OSC 8 hyperlinks when stdout is a terminal.

- `auto` enables styling and hyperlinks when stdout is a terminal.
- `always` keeps styling and hyperlinks even when stdout is redirected.
- `never` prints plain rendered text.

<strong class="cli-example-label">Example</strong>

```sh
mdv --color always example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="color"></div>

</details>

</section>

<section class="cli-option" aria-label="--color-depth">

### `--color-depth`

<p class="cli-syntax"><code>--color-depth &lt;DEPTH&gt;</code></p>

Sets the palette limit for the document and interactive interface. Applies when terminal styling is enabled.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`auto`</strong><br> Detects the palette supported by the terminal.

- `auto` detects the terminal's color capabilities.
- `16` uses the standard ANSI palette.
- `256` uses the ANSI 256-color palette.
- `truecolor` uses 24-bit RGB colors.

<strong class="cli-example-label">Example</strong>

```sh
mdv --color-depth 256 example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="color-depth"></div>

</details>

</section>

<section class="cli-option" aria-label="--hide-comments">

### `--hide-comments`

Removes Markdown comments from rendered output.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Comments remain visible.

<strong class="cli-example-label">Example</strong>

```sh
mdv --hide-comments example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="hide-comments"></div>

</details>

</section>

<section class="cli-option" aria-label="--front-matter">

### `--front-matter`

<p class="cli-syntax"><code>--front-matter &lt;MODE&gt;</code></p>

Controls how mdv renders a YAML mapping enclosed by exact `---` delimiter lines. The opening delimiter must be on the first line.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`hidden`</strong><br> Omits YAML front matter from rendered output.

- `hidden` omits front matter.
- `panel` shows a property panel.
- `table` shows `Property` and `Value` columns.
- `plain` shows normalized key/value lines.
- `inline` puts properties in one wrapping paragraph.
- `blocks` puts each value below its key.
- `code` shows syntax-highlighted YAML.
- `source` parses the complete input as ordinary Markdown.

<strong class="cli-example-label">Example</strong>

```sh
mdv --front-matter panel example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="front-matter"></div>

</details>

</section>

<section class="cli-option" aria-label="--theme-info">

### `--theme-info`

<p class="cli-syntax"><code>--theme-info [FILE]</code></p>

With no value, shows the active themes and lists available themes. Pass a file to render it with the active theme information.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Theme information is suppressed.

<strong class="cli-example-label">Example</strong>

```sh
mdv --theme-info example.md
```

</section>

<section class="cli-option" aria-label="--from">

### `--from`

<p class="cli-syntax"><code>--from &lt;TEXT&gt;[:&lt;LINES&gt;]</code></p>

Starts rendering at the first line containing `<TEXT>`. Add `:<LINES>` to limit the number of source lines, including the matching line. If there is no match, rendering starts at the first line.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Full document</strong><br> Renders the full document without filtering.

<strong class="cli-example-label">Example</strong>

```sh
mdv --from 'Installation:20' README.md
```

</section>

<section class="cli-option" aria-label="--reverse">

### `--reverse`

<p class="cli-syntax"><code>-r, --reverse</code></p>

Renders the document from the end while preserving block formatting.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Document order</strong><br> Renders from the beginning of the document.

<strong class="cli-example-label">Example</strong>

```sh
mdv --reverse example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="reverse"></div>

</details>

</section>

<section class="cli-option" aria-label="--pager">

### `--pager`

<p class="cli-syntax"><code>-p, --pager[=&lt;COMMAND&gt;]</code></p>

Opens rendered output in a pager when stdout is a terminal. With redirected stdout, writes directly to stdout.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Direct output</strong><br> Ordinary document output goes directly to stdout.

- `--pager` uses `MDV_PAGER` when set, otherwise it uses the built-in `minus` pager.
- `--pager=default` or `--pager=builtin` selects the built-in `minus` pager.
- `--pager=<COMMAND>` runs an external pager and passes rendered output to its stdin.

<strong class="cli-example-label">Example</strong>

```sh
mdv --pager=builtin example.md
```

<details class="mdv-reference-preview">
<summary>Preview the built-in pager</summary>
<div data-mdv-preview="builtin-pager"></div>
</details>

</section>

<section class="cli-option" aria-label="--interactive">

### `--interactive`

<p class="cli-syntax"><code>-i, --interactive</code></p>

Opens the interactive document browser for a directory, or opens a file or standard input directly in the selected pager. Requires a terminal on stdout and conflicts with `--pager`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Automatic</strong><br> A directory argument opens the browser. With no arguments, terminal stdin and stdout open the current directory. Ordinary files and piped input use direct output.

The browser searches recursively, honoring hidden paths and `.gitignore` rules.

<strong class="cli-example-label">Example</strong>

```sh
mdv --interactive .
```

<details class="mdv-reference-preview">
<summary>Preview the document browser</summary>
<div data-mdv-preview="document-browser"></div>
</details>

</section>

<section class="cli-option" aria-label="--monitor">

### `--monitor`

Watches a file and re-renders it when it changes. Applies to direct output from a file.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Renders once without watching for changes.

<strong class="cli-example-label">Example</strong>

```sh
mdv --monitor example.md
```

</section>

<section class="cli-option" aria-label="--config-file">

### `--config-file`

<p class="cli-syntax"><code>-F, --config-file &lt;CONFIG_DIR&gt;</code></p>

Looks for `config.yaml` and then `config.yml` in the specified directory. If neither exists, also checks `MDV_CONFIG_PATH` when set.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Automatic lookup</strong><br> Looks in `MDV_CONFIG_PATH` when set, otherwise in the [platform's default configuration directory](../configuration/).

<strong class="cli-example-label">Example</strong>

```sh
mdv --config-file ./config example.md
```

</section>

<section class="cli-option" aria-label="--preset">

### `--preset`

<p class="cli-syntax"><code>-P, --preset &lt;NAME&gt;</code></p>

Applies a built-in or user preset after loading the configuration file and before applying environment and CLI overrides.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No preset</strong><br> Keeps the loaded configuration and applies environment and CLI overrides.

<strong class="cli-example-label">Example</strong>

```sh
mdv --preset reader example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="preset"></div>

</details>

</section>

<section class="cli-option" aria-label="--preset-info">

### `--preset-info`

Lists available presets when no positional file is supplied. With a file and `--preset`, renders the file and prints `Current preset: NAME`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Preset information is suppressed.

<strong class="cli-example-label">Example</strong>

```sh
mdv --preset-info example.md --preset pretty
```

</section>

<section class="cli-option" aria-label="--no-config">

### `--no-config`

<p class="cli-syntax"><code>-n, --no-config</code></p>

Skips loading `config.yaml` and `config.yml`. Presets remain available.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Config loading on</strong><br> Loads the first configuration file found.

<strong class="cli-example-label">Example</strong>

```sh
mdv --no-config example.md
```

</section>

<section class="cli-option" aria-label="--init-config">

### `--init-config`

<p class="cli-syntax"><code>--init-config [CONFIG_DIR]</code></p>

Creates `config.yaml` and exits. An existing `config.yaml` is preserved and the command returns an error.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">On request</strong><br> Regular runs leave configuration files intact.

A bare `--init-config` uses the directory from `--config-file`, then `MDV_CONFIG_PATH`, then the platform default.

<strong class="cli-example-label">Example</strong>

```sh
mdv --init-config ./config
```

</section>

## Theming

<section class="cli-option" aria-label="--theme">

### `--theme`

<p class="cli-syntax"><code>-t, --theme &lt;NAME&gt;</code></p>

Chooses a built-in or [user theme](../themes/).

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`terminal`</strong><br> Uses mdv's built-in terminal palette.

<strong class="cli-example-label">Example</strong>

```sh
mdv --theme catppuccin example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-theme">

### `--code-theme`

<p class="cli-syntax"><code>-T, --code-theme &lt;NAME&gt;</code></p>

Chooses a separate syntax-highlighting theme. Accepts the same theme names as `--theme`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Document theme</strong><br> Syntax highlighting follows the document theme.

<strong class="cli-example-label">Example</strong>

```sh
mdv --code-theme monokai example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--syntaxes-dir">

### `--syntaxes-dir`

<p class="cli-syntax"><code>--syntaxes-dir &lt;DIR&gt;</code></p>

Recursively loads custom `.sublime-syntax` files. Custom definitions take precedence over matching embedded definitions. Relative CLI paths are resolved from the working directory.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Embedded syntaxes</strong><br> Uses only the embedded syntax set.

<strong class="cli-example-label">Example</strong>

```sh
mdv --syntaxes-dir ./syntaxes example.md
```

</section>

<section class="cli-option" aria-label="--code-block-style">

### `--code-block-style`

<p class="cli-syntax"><code>-b, --code-block-style &lt;STYLE&gt;[:&lt;OPTIONS&gt;]</code></p>

Sets the layout and optional label for fenced code blocks.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`basic`</strong><br> Indents code without a border or language label.

**Styles**

- `basic` indents code without a border.
- `simple` adds a single left border.
- `pretty` adds a frame.

Add `show-name` for the language name or `show-icon` for its icon. Combine them with `;`. Icons require a Nerd Font.

<strong class="cli-example-label">Example</strong>

```sh
mdv --code-block-style 'pretty:show-name;show-icon' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-block-style"></div>

<div data-mdv-preview="code-labels"></div>

</details>

</section>

<section class="cli-option" aria-label="--math-block-style">

### `--math-block-style`

<p class="cli-syntax"><code>--math-block-style &lt;STYLE&gt;</code></p>

Sets the layout for display formulas and fenced `math`, `latex`, and `tex` blocks. Math blocks do not use code labels, icons, or line numbers.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`basic`</strong><br> Indents formulas without a border.

- `basic` indents the formula without a border.
- `simple` adds a single left border.
- `pretty` adds a frame.

<strong class="cli-example-label">Example</strong>

```sh
mdv --math-block-style pretty example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="math-block-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-line-numbers">

### `--code-line-numbers`

<p class="cli-syntax"><code>-K, --code-line-numbers [&lt;MODE&gt;]</code></p>

Adds line numbers inside code blocks. Numbering restarts for each block.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Renders code rows without a line-number gutter.

- With no value, numbers each rendered code row without a separator.
- `source` numbers physical code lines and leaves wrapped continuations unnumbered.
- `separator` numbers rendered code rows and adds a separator.
- `source;separator` numbers source lines and adds a separator.

<strong class="cli-example-label">Example</strong>

```sh
mdv --code-line-numbers 'source;separator' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-theme">

### `--custom-theme`

<p class="cli-syntax"><code>--custom-theme &lt;key=value;...&gt;</code></p>

Overrides colors from the selected theme.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses the selected theme without color overrides.

Separate assignments with `;`. Use `key=value` for top-level colors and `section:field=value` for grouped colors. See the [theme field reference](../theme-reference/) for available keys. Colors can be named, hex, RGB, or `ansi(N)`.

<strong class="cli-example-label">Example</strong>

```sh
mdv --custom-theme 'line_number:number=grey;h2=blue' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--inline-style">

### `--inline-style`

<p class="cli-syntax"><code>--inline-style &lt;STYLES&gt;</code></p>

Overrides decorations for inline Markdown elements.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses the active theme's decorations.

Use `element:property=true,property=false`. Separate elements with `;` and properties with `,`.

**Elements** `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight`.

**Properties** `backticks`, `bold`, `italic`, `underline`, `strikethrough`.

<strong class="cli-example-label">Example</strong>

```sh
mdv --inline-style 'code:backticks=false,bold=true;highlight:underline=true' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="inline-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-theme">

### `--custom-code-theme`

<p class="cli-syntax"><code>--custom-code-theme &lt;key=value;...&gt;</code></p>

Overrides syntax-highlighting colors.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses the effective syntax palette without these overrides.

Separate `key=value` assignments with `;`. Both `keyword` and `syntax:keyword` forms are accepted.

<strong class="cli-example-label">Example</strong>

```sh
mdv --custom-code-theme 'keyword=#ffffff;string=128,0,128' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-block">

### `--custom-code-block`

<p class="cli-syntax"><code>--custom-code-block &lt;BLOCKS&gt;</code></p>

Overrides icons, labels, or aliases for code-block language hints. Syntax highlighting requires a matching syntax definition.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses built-in icons, labels, and aliases.

Separate language entries with `;`, properties with `,`, and aliases with `|`. An explicit list replaces inherited aliases. Use `aliases=[]` to clear them. Use `default:icon=…` to set the icon for unknown languages.

<strong class="cli-example-label">Example</strong>

```sh
mdv --code-block-style pretty:show-name --custom-code-block 'rust:label=Rust sample' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-code-block"></div>

</details>

</section>

## Callouts

<section class="cli-option" aria-label="--callout-style">

### `--callout-style`

<p class="cli-syntax"><code>-C, --callout-style &lt;STYLE&gt;[:&lt;OPTIONS&gt;]</code></p>

Sets the callout layout, icons, and label formatting.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`pretty`</strong><br> Uses a frame with the label above it and icons off.

**Styles**

- `pretty` adds a frame.
- `simple` uses a single left border.

**Options**

- `show-icons` uses Nerd Font icons.
- `show-simple-icons` uses ASCII markers such as `[i]`, `[*]`, `[!]`, and `[x]`. It cannot be combined with `show-icons`.
- `fold-icons` shows fold markers and requires `show-icons`.
- `label-inside` moves the label into the frame and requires `pretty`.
- `uppercase` converts labels to uppercase.

Separate multiple options with `;`.

<strong class="cli-example-label">Example</strong>

```sh
mdv --callout-style 'pretty:show-icons;fold-icons;label-inside' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="callout-style"></div>

<div data-mdv-preview="callout-icons"></div>

<div data-mdv-preview="callout-label"></div>

<div data-mdv-preview="callout-uppercase"></div>

<div data-mdv-preview="callout-fold"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-callout">

### `--custom-callout`

<p class="cli-syntax"><code>--custom-callout &lt;name:icon=...,color=...;...&gt;</code></p>

Overrides or defines callout icons and colors. Specify either property or both.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses the built-in callout styles and active theme colors.

Separate entries with `;` and properties with `,`. Colors use the same formats as [`--custom-theme`](#--custom-theme).

<strong class="cli-example-label">Example</strong>

```sh
mdv --custom-callout 'tip:icon=*,color=red;custom:color=#ff5555' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-callout"></div>

</details>

</section>

## Checkboxes

<section class="cli-option" aria-label="--checkbox-style">

### `--checkbox-style`

<p class="cli-syntax"><code>-x, --checkbox-style &lt;square|circle&gt;</code></p>

Renders task-list checkboxes as `square` or `circle` icons. Requires a Nerd Font.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Text markers</strong><br> Uses text markers such as `[ ]` and `[x]`.

<strong class="cli-example-label">Example</strong>

```sh
mdv --checkbox-style square example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="checkbox-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-checkbox">

### `--custom-checkbox`

<p class="cli-syntax"><code>--custom-checkbox &lt;PAIRS&gt;</code></p>

Overrides checkbox icons and colors or adds new states. Requires `--checkbox-style`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses the icons and colors provided by the selected checkbox style and theme.

Separate entries with `;`. Use `<char>:<icon>[:<color>]`, `<char>:<color>`, or `<char>:icon=<glyph>,color=<color>`. With only a color, a known state keeps its icon and a new state uses the unchecked icon. Colors can be named, hex, RGB, or `ansi(N)`.

<strong class="cli-example-label">Example</strong>

```sh
mdv --checkbox-style square --custom-checkbox ' :!:yellow' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-checkbox"></div>

</details>

</section>

## Lists

<section class="cli-option" aria-label="--list-style">

### `--list-style`

<p class="cli-syntax"><code>-L, --list-style &lt;LIST_STYLE&gt;</code></p>

Uses level-specific icons for unordered-list markers.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Text markers</strong><br> Uses `-` markers.

- `type` accepts `nerd-font` or `unicode`.
- `size` accepts `large` or `small` and changes only Nerd Font markers.

Use `type:TYPE;size:SIZE`, or specify just one option. Omitted options use `type:nerd-font` and `size:large`. Nerd Font markers require a Nerd Font. Unicode marker spacing can vary by font.

<strong class="cli-example-label">Example</strong>

```sh
mdv --list-style 'type:unicode' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="list-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--definition-marker-style">

### `--definition-marker-style`

<p class="cli-syntax"><code>-D, --definition-marker-style &lt;unicode|nerd-font&gt;</code></p>

Adds a `unicode` or `nerd-font` marker to definition descriptions. The `nerd-font` style requires a Nerd Font.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Indent only</strong><br> Indents descriptions by two spaces without a marker.

<strong class="cli-example-label">Example</strong>

```sh
mdv --definition-marker-style unicode example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="definition-marker-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--uniform-list-marker">

### `--uniform-list-marker`

<p class="cli-syntax"><code>--uniform-list-marker &lt;MARKER&gt;</code></p>

Uses one unordered-list marker at all nesting levels. Requires `--list-style`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Per level</strong><br> Keeps the selected set's level-specific markers.

- `level:<1-4>` uses the marker from that level.
- `icon:<GLYPH>` uses a custom marker.

<strong class="cli-example-label">Example</strong>

```sh
mdv --list-style 'type:unicode' --uniform-list-marker 'level:2' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="uniform-list-marker"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-list">

### `--custom-list`

<p class="cli-syntax"><code>--custom-list &lt;PAIRS&gt;</code></p>

Overrides unordered-list markers by nesting level. Requires `--list-style`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">No overrides</strong><br> Uses the selected set's markers and theme colors.

Separate entries with `;`. Levels start at 1. Use `<level>:<icon>[:<color>]`, `<level>:<color>`, or `<level>:icon=<glyph>,color=<color>`. Colors can be named, hex, RGB, or `ansi(N)`.

<strong class="cli-example-label">Example</strong>

```sh
mdv --list-style 'type:unicode' --custom-list '1:*:yellow' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-list"></div>

</details>

</section>

## Layout and wrapping

<section class="cli-option" aria-label="--cols">

### `--cols`

<p class="cli-syntax"><code>-c, --cols &lt;N&gt;</code></p>

Sets the output width in terminal columns.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Automatic</strong><br> Uses a detected terminal width of at least 20 columns, then configured `cols` if available, then 80 columns.

<strong class="cli-example-label">Example</strong>

```sh
mdv --cols 72 example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="cols"></div>

</details>

</section>

<section class="cli-option" aria-label="--margin">

### `--margin`

<p class="cli-syntax"><code>-m, --margin &lt;MARGINS&gt;</code></p>

Reserves left and right margins in terminal columns. Specify either side or both. A side omitted from the argument is zero. HTML export does not use terminal margins.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">0 columns</strong><br> Both sides are zero.

<strong class="cli-example-label">Example</strong>

```sh
mdv --margin 'left:4;right:5' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="margin"></div>

</details>

</section>

<section class="cli-option" aria-label="--tab-length">

### `--tab-length`

<p class="cli-syntax"><code>--tab-length &lt;N&gt;</code></p>

Sets how many spaces replace a tab.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`4`</strong><br> Replaces each tab with four spaces.

<strong class="cli-example-label">Example</strong>

```sh
mdv --tab-length 2 example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="tab-length"></div>

</details>

</section>

<section class="cli-option" aria-label="--wrap">

### `--wrap`

<p class="cli-syntax"><code>-w, --wrap &lt;MODE&gt;</code></p>

Selects where to break prose, code, callouts, and table-cell text.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`char`</strong><br> Wraps at character boundaries.

- `char` breaks at character boundaries.
- `word` prefers word boundaries. A word that cannot fit can still break.
- `none` turns off soft wrapping. Constrained table cells may still break to fit.

<strong class="cli-example-label">Example</strong>

```sh
mdv --wrap word example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--reflow">

### `--reflow`

Collapses soft source line breaks within paragraphs and reflows the text to the output width. Requires wrapping and preserves hard breaks.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Preserves source line breaks within paragraphs.

<strong class="cli-example-label">Example</strong>

```sh
mdv --reflow example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="reflow"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-wrap">

### `--table-wrap`

<p class="cli-syntax"><code>-W, --table-wrap &lt;MODE&gt;</code></p>

Controls how wide tables use the available terminal width. Breakpoints inside cells follow `--wrap` when the layout constrains cell width.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`fit`</strong><br> Wraps cell content to fit within the available width.

- `fit` wraps cell content to fit the table within the terminal width.
- `wrap` splits wide tables into column blocks.
- `none` leaves cells unwrapped and allows horizontal overflow.

<strong class="cli-example-label">Example</strong>

```sh
mdv --table-wrap wrap example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="table-wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-borders">

### `--table-borders`

<p class="cli-syntax"><code>-B, --table-borders</code></p>

Draws a full table grid with rounded outer corners.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Compact borders</strong><br> Uses compact borders without a full outer grid.

<strong class="cli-example-label">Example</strong>

```sh
mdv --table-borders example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="table-borders"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-smart-indent">

### `--table-smart-indent`

<p class="cli-syntax"><code>-S, --table-smart-indent</code></p>

Uses the current content indent for tables when space allows and reduces it when width is tight.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Automatic table indent adjustment is off.

<strong class="cli-example-label">Example</strong>

```sh
mdv --table-smart-indent example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="table-smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--heading-layout">

### `--heading-layout`

<p class="cli-syntax"><code>-H, --heading-layout &lt;LAYOUT&gt;</code></p>

Sets heading alignment and the indentation of content under headings.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`level`</strong><br> Indents headings by level and content one column further.

- `level` indents headings by level and content one column further.
- `center` centers headings and leaves content unindented.
- `flat` aligns headings to the left and indents content one column.
- `none` aligns headings and content to the left.

<strong class="cli-example-label">Example</strong>

```sh
mdv --heading-layout center example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="heading-layout"></div>

</details>

</section>

<section class="cli-option" aria-label="--show-heading-markers">

### `--show-heading-markers`

Prefixes headings with Markdown markers matching their level.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Heading markers are hidden.

<strong class="cli-example-label">Example</strong>

```sh
mdv --show-heading-markers example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="show-heading-markers"></div>

</details>

</section>

<section class="cli-option" aria-label="--smart-indent">

### `--smart-indent`

<p class="cli-syntax"><code>-I, --smart-indent</code></p>

Smooths indentation jumps between consecutive headings. Applies only to `--heading-layout level`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Heading indentation follows the actual heading level.

<strong class="cli-example-label">Example</strong>

```sh
mdv --heading-layout level --smart-indent example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-wrap-indent">

### `--code-wrap-indent`

<p class="cli-syntax"><code>--code-wrap-indent &lt;MODE&gt;</code></p>

Controls hanging indentation on wrapped code lines.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`double`</strong><br> Adds two spaces to the original indent on wrapped code lines.

- `none` adds no hanging indent.
- `base` aligns continuations with the original line indentation.
- `double` adds two spaces to the original indentation.

<strong class="cli-example-label">Example</strong>

```sh
mdv --code-wrap-indent base example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-wrap-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--block-spacing">

### `--block-spacing`

<p class="cli-syntax"><code>--block-spacing &lt;SPACING&gt;</code></p>

Sets how many blank lines appear above and below each block type. Adjacent gaps collapse to the larger value.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Built-in spacing</strong><br> Most blocks use one line above and below. Paragraphs use zero above and one below. `end-references` uses two above and one below.

Use `<element>:top=<lines>,bottom=<lines>`. Separate elements with `;` and sides with `,`. Omitted elements and sides retain their spacing. See [block spacing](../rendering/) for supported block types.

<strong class="cli-example-label">Example</strong>

```sh
mdv --block-spacing 'paragraph:bottom=0;callout:top=2' example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="block-spacing"></div>

</details>

</section>

<section class="cli-option" aria-label="--horizontal-rule-style">

### `--horizontal-rule-style`

<p class="cli-syntax"><code>--horizontal-rule-style &lt;STYLE&gt;</code></p>

Sets the appearance of Markdown rules and HTML `<hr>`.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`pretty`</strong><br> Draws a line with decorative endpoints.

- `pretty` draws a line with decorative endpoints.
- `simple` draws a continuous line.

<strong class="cli-example-label">Example</strong>

```sh
mdv --horizontal-rule-style simple example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="horizontal-rule-style"></div>

</details>

</section>

## Content visibility

<section class="cli-option" aria-label="--show-empty-elements">

### `--show-empty-elements`

Keeps empty headings, list items, block quotes, code blocks, and tables that would otherwise be hidden.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Off</strong><br> Hides empty elements that are unnecessary for layout.

<strong class="cli-example-label">Example</strong>

```sh
mdv --show-empty-elements example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="show-empty-elements"></div>

</details>

</section>

<section class="cli-option" aria-label="--no-code-guessing">

### `--no-code-guessing`

Disables heuristic language detection. Explicitly supported language hints still select syntax highlighting.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">Detection on</strong><br> Tries to detect the language when the code block has no recognized hint.

<strong class="cli-example-label">Example</strong>

```sh
mdv --no-code-guessing example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-guessing"></div>

</details>

</section>

## Links

<section class="cli-option" aria-label="--link-style">

### `--link-style`

<p class="cli-syntax"><code>-u, --link-style &lt;STYLE&gt;</code></p>

Sets what readers see for links. OSC 8 hyperlinks require terminal styling.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`clickable`</strong><br> Displays clickable link text in a terminal that supports OSC 8.

- `clickable` (`c`) makes the link text clickable and hides the URL.
- `fclickable` (`fc`) underlines clickable link text.
- `inline` (`i`) shows the URL after the link text.
- `inlinetable` (`it`) numbers links and shows a reference table near the block.
- `endtable` (`et`) numbers links and shows a reference table at the document end.
- `hide` (`h`) shows link text only.

<strong class="cli-example-label">Example</strong>

```sh
mdv --link-style inline example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="link-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--link-overflow">

### `--link-overflow`

<p class="cli-syntax"><code>-l, --link-overflow &lt;MODE&gt;</code></p>

Controls the layout of visible URLs that do not fit the available width. The actual link destination stays the same.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`wrap`</strong><br> Wraps visible URLs that exceed the available width.

- `wrap` wraps long URLs.
- `cut` shortens long URLs with an ellipsis.
- `tablecut` shortens long URLs in text and inside table cells.
- `none` allows horizontal overflow.

<strong class="cli-example-label">Example</strong>

```sh
mdv --link-style inline --link-overflow cut example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="link-overflow"></div>

</details>

</section>

## Footnotes

<section class="cli-option" aria-label="--footnote-style">

### `--footnote-style`

<p class="cli-syntax"><code>--footnote-style &lt;STYLE&gt;</code></p>

Sets where referenced footnotes appear.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`endnotes`</strong><br> Collects referenced footnotes at the end of the document.

- `endnotes` collects footnotes at the end of the document.
- `attached` places footnotes after the block that references them, including paragraphs, tables, and list items.

<strong class="cli-example-label">Example</strong>

```sh
mdv --footnote-style attached example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="footnote-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--missing-footnote-style">

### `--missing-footnote-style`

<p class="cli-syntax"><code>--missing-footnote-style &lt;STYLE&gt;</code></p>

Controls the display of missing, invalid, or empty footnote definitions.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">`show`</strong><br> Shows a placeholder for missing, invalid, or empty definitions.

- `show` displays a placeholder entry.
- `hide` omits the entry.

<strong class="cli-example-label">Example</strong>

```sh
mdv --missing-footnote-style hide example.md
```

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="missing-footnote-style"></div>

</details>

</section>

## Information

<section class="cli-option" aria-label="mdv help">

### `mdv help`

Shows the full help in a pager when stdin and stdout are terminals. Uses `MDV_PAGER` when set, otherwise the built-in pager. If either stream is redirected, prints the same help directly.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">On request</strong><br> Opens help when the subcommand is supplied.

<strong class="cli-example-label">Example</strong>

```sh
mdv help
```

</section>

<section class="cli-option" aria-label="-h">

### `-h`

Prints a concise option summary.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">On request</strong><br> Prints the summary when the flag is supplied.

<strong class="cli-example-label">Example</strong>

```sh
mdv -h
```

</section>

<section class="cli-option" aria-label="--help">

### `--help`

Prints the full help with value descriptions, formats, and examples.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">On request</strong><br> Prints full help when the flag is supplied.

<strong class="cli-example-label">Example</strong>

```sh
mdv --help
```

</section>

<section class="cli-option" aria-label="--version">

### `--version`

<p class="cli-syntax"><code>-V, --version</code></p>

Prints the current version.

<strong class="cli-default-label">Default</strong> <strong class="cli-default-value">On request</strong><br> Prints the version when the flag is supplied.

<strong class="cli-example-label">Example</strong>

```sh
mdv --version
```

</section>
