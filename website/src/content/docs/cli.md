---
title: "Complete CLI reference"
description: "Every command-line option, its format, accepted values and behavior."
slug: "cli"
group: "Reference"
order: 43
---

## Output and workflow

<section class="cli-option" aria-label="--html">

### `--html`

Prints HTML instead of terminal formatting.

</section>

<section class="cli-option" aria-label="--render-html">

### `--render-html`

<p class="cli-syntax"><code>-E, --render-html</code></p>

Renders raw HTML fragments as terminal-formatted content instead of displaying their tags literally.

- HTML nesting is limited to 64 DOM elements.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="render-html"></div>

</details>

</section>

<section class="cli-option" aria-label="--line-numbers">

### `--line-numbers`

<p class="cli-syntax"><code>-N, --line-numbers [&lt;MODE&gt;]</code></p>

Prefixes terminal and pager rows with line numbers.

- Without a mode, it numbers rendered rows
- `source` uses physical Markdown source lines
- `separator` adds a separator
- `"source;separator"` combines both.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--color">

### `--color`

<p class="cli-syntax"><code>--color &lt;auto|always|never&gt;</code></p>

Controls terminal styling.

- `auto` enables ANSI and OSC 8 only when stdout is a terminal
- `always` preserves styling through pipes
- `never` emits plain rendered output.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="color"></div>

</details>

</section>

<section class="cli-option" aria-label="--color-depth">

### `--color-depth`

<p class="cli-syntax"><code>--color-depth &lt;auto|16|256|truecolor&gt;</code></p>

Detects the terminal palette or sets its limit explicitly.

- Applies to document colors and the interactive interface; default `auto`.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="color-depth"></div>

</details>

</section>

<section class="cli-option" aria-label="--hide-comments">

### `--hide-comments`

Removes Markdown comments from the rendered output.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="hide-comments"></div>

</details>

</section>

<section class="cli-option" aria-label="--front-matter">

### `--front-matter`

<p class="cli-syntax"><code>--front-matter &lt;hidden|panel|table|plain|inline|blocks|code|source&gt;</code></p>

Controls an exact `---`-delimited YAML property mapping beginning on the first line.

- `hidden` omits it
- `panel` renders a property panel
- `table` uses Property/Value columns
- `plain` emits normalized key/value lines
- `inline` joins properties into one flowing row
- `blocks` places each value below its key
- `code` renders highlighted YAML
- `source` parses the complete input as ordinary Markdown.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="front-matter"></div>

</details>

</section>

<section class="cli-option" aria-label="--theme-info">

### `--theme-info`

<p class="cli-syntax"><code>--theme-info [FILE]</code></p>

Shows the active palette; when `FILE` is provided it renders the file along with palette information.

</section>

<section class="cli-option" aria-label="--from">

### `--from`

<p class="cli-syntax"><code>--from &lt;TEXT&gt;</code></p>

Starts rendering from the first match of `<TEXT>`.

- Adding `:<lines>` limits the number of lines (for example `--from "Install:20"`).

</section>

<section class="cli-option" aria-label="--reverse">

### `--reverse`

<p class="cli-syntax"><code>-r, --reverse</code></p>

Renders the document starting from the end while keeping block formatting intact.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="reverse"></div>

</details>

</section>

<section class="cli-option" aria-label="--pager">

### `--pager`

<p class="cli-syntax"><code>-p, --pager[=&lt;COMMAND&gt;]</code></p>

Opens the rendered output in a pager.

- A bare flag uses `MDV_PAGER` when set and the default pager otherwise.
- `--pager=default` selects the default backend (currently the built-in `minus`)
- `--pager=builtin` explicitly selects `minus`, and a command such as `--pager="less -R"` receives the rendered output on stdin.

<details class="mdv-reference-preview">
<summary>Preview the built-in pager</summary>
<div data-mdv-preview="builtin-pager"></div>
</details>

</section>

<section class="cli-option" aria-label="--interactive">

### `--interactive`

<p class="cli-syntax"><code>-i, --interactive</code></p>

Opens the interactive document browser.

- A directory opens the browser; a file or standard input opens the document directly in the selected pager.
- Running `mdv` without arguments in a terminal opens the current directory, and passing a directory opens that directory.
- Redirected output does not open the browser implicitly.
- The browser recursively finds Markdown files while honoring hidden-file and `.gitignore` rules.

<details class="mdv-reference-preview">
<summary>Preview the document browser</summary>
<div data-mdv-preview="document-browser"></div>
</details>

</section>

<section class="cli-option" aria-label="--monitor">

### `--monitor`

Watches the source file and re-renders when it changes.

</section>

<section class="cli-option" aria-label="--config-file">

### `--config-file`

<p class="cli-syntax"><code>-F, --config-file &lt;CONFIG_DIR&gt;</code></p>

Reads configuration from the provided directory.

</section>

<section class="cli-option" aria-label="--preset">

### `--preset`

<p class="cli-syntax"><code>-P, --preset &lt;NAME&gt;</code></p>

Applies a built-in or user preset above the configuration file and below explicit CLI options.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="preset"></div>

</details>

</section>

<section class="cli-option" aria-label="--preset-info">

### `--preset-info`

Lists available presets when no positional `FILE` is provided.

- `mdv --preset-info FILE --preset NAME` renders the document and prints `Current preset: NAME`; with `FILE` but no `--preset`, the flag does not alter rendering.

</section>

<section class="cli-option" aria-label="--no-config">

### `--no-config`

<p class="cli-syntax"><code>-n, --no-config</code></p>

Skips `config.yaml`/`config.yml`; presets remain available.

</section>

<section class="cli-option" aria-label="--init-config">

### `--init-config`

<p class="cli-syntax"><code>--init-config [CONFIG_DIR]</code></p>

Creates the default config file.

- Uses the provided directory
- `--config-file`
- `MDV_CONFIG_PATH`, or the default config directory.

</section>

## Theming

<section class="cli-option" aria-label="--theme">

### `--theme`

<p class="cli-syntax"><code>-t, --theme &lt;NAME&gt;</code></p>

Chooses a built-in or user theme (default `terminal`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-theme">

### `--code-theme`

<p class="cli-syntax"><code>-T, --code-theme &lt;NAME&gt;</code></p>

Selects a separate syntax palette.

- Without an explicit selection, code follows the document theme.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--syntaxes-dir">

### `--syntaxes-dir`

<p class="cli-syntax"><code>--syntaxes-dir &lt;DIR&gt;</code></p>

Recursively loads custom `.sublime-syntax` files on top of the embedded syntax set.

- Custom entries take precedence over matching built-in syntaxes.

</section>

<section class="cli-option" aria-label="--code-block-style">

### `--code-block-style`

<p class="cli-syntax"><code>-b, --code-block-style &lt;basic|simple|pretty&gt;[:show-name;show-icon]</code></p>

Selects an indented borderless block, a single gutter, or a boxed frame.

- Labels are hidden by default
- `show-name` displays the language name
- `show-icon` displays its icon, and both options may be combined (default `basic`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-block-style"></div>

<div data-mdv-preview="code-labels"></div>

</details>

</section>

<section class="cli-option" aria-label="--math-block-style">

### `--math-block-style`

<p class="cli-syntax"><code>--math-block-style &lt;basic|simple|pretty&gt;</code></p>

Selects the presentation for display formulas and fenced `math`/`latex`/`tex` blocks.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="math-block-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-line-numbers">

### `--code-line-numbers`

<p class="cli-syntax"><code>-K, --code-line-numbers [&lt;MODE&gt;]</code></p>

Numbers rows inside each code block.

- Without a mode, every wrapped terminal row is numbered
- `source` numbers physical code lines and leaves wrapped continuations unnumbered
- `separator` adds a separator
- `"source;separator"` combines both modes.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-theme">

### `--custom-theme`

<p class="cli-syntax"><code>--custom-theme &lt;key=value;...&gt;</code></p>

Overrides UI colors on top of the selected theme.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--inline-style">

### `--inline-style`

<p class="cli-syntax"><code>--inline-style &lt;STYLES&gt;</code></p>

Overrides decorations for `emphasis`

- `strong`
- `strong_emphasis`
- `code`
- `strikethrough`
- `highlight`.
- Use `element:property=true,property=false` entries separated by `;`; properties are `backticks`
- `bold`
- `italic`
- `underline`
- `strikethrough` (for example `--inline-style 'code:backticks=false,bold=true;highlight:underline=true'`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="inline-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-theme">

### `--custom-code-theme`

<p class="cli-syntax"><code>--custom-code-theme &lt;key=value;...&gt;</code></p>

Overrides syntax colors using the same format as `--custom-theme`.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-block">

### `--custom-code-block`

<p class="cli-syntax"><code>--custom-code-block &lt;lang:icon=...,label=...,aliases=...|...&gt;[;...]</code></p>

Overrides the icon, label, and aliases for specific code block languages.

- Multiple languages are separated by `;`, options within one language by `,`, and aliases by `|` (for example
- `python:icon=*,label=Python,aliases=py|py3;rust:icon=R`).
- Works for any language hint, even if it is not in the built-in icon mapping.
- Syntax highlighting is applied only when the language is supported by mdv's syntax highlighting logic.
- Use `default:icon=...` to set the fallback icon for unknown languages.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-code-block"></div>

</details>

</section>

## Callouts

<section class="cli-option" aria-label="--callout-style">

### `--callout-style`

<p class="cli-syntax"><code>-C, --callout-style &lt;pretty|simple&gt;[:show-icons;show-simple-icons;fold-icons;label-inside;uppercase]</code></p>

Sets the callout layout and label behavior.

- `show-icons` uses Nerd Font glyphs
- `show-simple-icons` uses portable ASCII markers such as `[i]`
- `[*]`
- `[!]`
- `[x]`.
- The two icon options cannot be combined.
- `label-inside` is only supported with `pretty`
- `fold-icons` requires `show-icons`.

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

Overrides or adds callout labels.

- `icon` and `color` are optional; color formats match `--custom-theme`.

- A Nerd Font is required in the terminal for icons enabled by `show-icons` and `fold-icons` to render correctly. `show-simple-icons` does not require a Nerd Font.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-callout"></div>

</details>

</section>

## Checkboxes

<section class="cli-option" aria-label="--checkbox-style">

### `--checkbox-style`

<p class="cli-syntax"><code>-x, --checkbox-style &lt;square|circle&gt;</code></p>

Renders task-list checkboxes as Nerd Font icons instead of the default `[ ]` / `[x]` markers.

- Requires a Nerd Font in the terminal.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="checkbox-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-checkbox">

### `--custom-checkbox`

<p class="cli-syntax"><code>--custom-checkbox &lt;PAIRS&gt;</code></p>

Overrides built-in checkbox icons or adds new states (only with `--checkbox-style`).

- Each semicolon-separated pair is either `<char>:<icon>[:<color>]` or `<char>:<color>`.
- The color-only form keeps an existing icon or uses the default unchecked icon for a new state.
- Colors accept named, hex, rgb
- `ansi(N)` values.

- Override:  `--custom-checkbox ' :!'`          replace the unchecked icon
- Add:       `--custom-checkbox '*:*'`          add a new `[*]` checkbox state
- Color:     `--custom-checkbox ' :!:yellow'`   or `#ff0000`, `128,1,1`, `ansi(200)`
- Color only: `--custom-checkbox '?:red'` keeps the Nerd Font icon for the `?` state and applies red.
- New state, color only: `--custom-checkbox '*:yellow'` uses the default unchecked icon and applies yellow.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-checkbox"></div>

</details>

</section>

## Lists

<section class="cli-option" aria-label="--list-style">

### `--list-style`

<p class="cli-syntax"><code>-L, --list-style &lt;style&gt;</code></p>

Replaces the default `-` unordered-list markers with a level-aware icon set.

- The value is required and uses the format `type:<nerd-font|unicode>;size:<large|small>`, for example `--list-style 'type:unicode;size:small'`.

- `size` changes only Nerd Font markers. Unicode accepts either value but renders the same markers; their spacing may vary by font. Verified with Nerd Font families, especially JetBrainsMono Nerd Font.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="list-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--definition-marker-style">

### `--definition-marker-style`

<p class="cli-syntax"><code>-D, --definition-marker-style &lt;unicode|nerd-font&gt;</code></p>

Renders definition descriptions with a built-in Unicode or Nerd Font marker.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="definition-marker-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--uniform-list-marker">

### `--uniform-list-marker`

<p class="cli-syntax"><code>--uniform-list-marker &lt;level:1-4|icon:glyph&gt;</code></p>

Uses one marker at every nesting level (only with `--list-style`).

- `level:2` reuses level 2 from the selected set
- `icon:*` uses a custom glyph.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="uniform-list-marker"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-list">

### `--custom-list`

<p class="cli-syntax"><code>--custom-list &lt;level&gt;:&lt;icon&gt;[:&lt;color&gt;];...</code></p>

Overrides the marker icon and/or color for specific nesting levels (only with `--list-style`).

- Level is 1-based; icon is the marker glyph.
- Colors accept named (`red`), hex (`#ff0000`), rgb (`255,0,0`)
- `ansi(N)` values.

- Icon + color:  `--custom-list '1:*:yellow'`   marker `*` in yellow
- Icon only:     `--custom-list '1:>'`          marker `>` in theme color
- Color only:    `--custom-list '1:red'`        keep built-in icon, red color

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="custom-list"></div>

</details>

</section>

## Layout and wrapping

<section class="cli-option" aria-label="--cols">

### `--cols`

<p class="cli-syntax"><code>-c, --cols &lt;N&gt;</code></p>

Enforces the output width.

- Without it, mdv uses a detected terminal width of at least 20 columns, then configured `cols`, then 80 columns.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="cols"></div>

</details>

</section>

<section class="cli-option" aria-label="--margin">

### `--margin`

<p class="cli-syntax"><code>-m, --margin &lt;right:N;left:N&gt;</code></p>

Reserves separate left and right terminal margins.

- An omitted side defaults to `0`.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="margin"></div>

</details>

</section>

<section class="cli-option" aria-label="--tab-length">

### `--tab-length`

<p class="cli-syntax"><code>--tab-length &lt;N&gt;</code></p>

Replaces tab characters with `N` spaces (default `4`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="tab-length"></div>

</details>

</section>

<section class="cli-option" aria-label="--wrap">

### `--wrap`

<p class="cli-syntax"><code>-w, --wrap &lt;char|word|none&gt;</code></p>

Selects the break strategy for text, code, callouts, and table cells (default `char`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--reflow">

### `--reflow`

Collapses in-paragraph source newlines (soft breaks) and refills each line to the wrap width, so hard-wrapped source reflows to fit smaller screens.

- Requires wrapping to be enabled; hard breaks are preserved.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="reflow"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-wrap">

### `--table-wrap`

<p class="cli-syntax"><code>-W, --table-wrap &lt;fit|wrap|none&gt;</code></p>

Chooses table geometry: fit cells, split columns into blocks, or allow overflow (default `fit`).

- Cell breaks still follow `--wrap`.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="table-wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-borders">

### `--table-borders`

<p class="cli-syntax"><code>-B, --table-borders</code></p>

Restores full rounded table borders.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="table-borders"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-smart-indent">

### `--table-smart-indent`

<p class="cli-syntax"><code>-S, --table-smart-indent</code></p>

Automatic table indent adjustment based on available width.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="table-smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--heading-layout">

### `--heading-layout`

<p class="cli-syntax"><code>-H, --heading-layout &lt;level|center|flat|none&gt;</code></p>

Controls heading indentation (default `level`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="heading-layout"></div>

</details>

</section>

<section class="cli-option" aria-label="--show-heading-markers">

### `--show-heading-markers`

Prefixes headings with markdown-style markers matching their level.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="show-heading-markers"></div>

</details>

</section>

<section class="cli-option" aria-label="--smart-indent">

### `--smart-indent`

<p class="cli-syntax"><code>-I, --smart-indent</code></p>

Smooths indentation jumps between heading levels in `level` mode.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-wrap-indent">

### `--code-wrap-indent`

<p class="cli-syntax"><code>--code-wrap-indent &lt;none|base|double&gt;</code></p>

Sets the hanging indent applied to wrapped code block lines (default `double`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-wrap-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--block-spacing">

### `--block-spacing`

<p class="cli-syntax"><code>--block-spacing &lt;spec&gt;</code></p>

Overrides top and bottom blank lines per block.

- Omitted sides keep their defaults, and adjacent block gaps collapse to the larger value.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="block-spacing"></div>

</details>

</section>

<section class="cli-option" aria-label="--horizontal-rule-style">

### `--horizontal-rule-style`

<p class="cli-syntax"><code>--horizontal-rule-style &lt;pretty|simple&gt;</code></p>

Draws Markdown rules and HTML `<hr>` with decorative endpoints or as a continuous line.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="horizontal-rule-style"></div>

</details>

</section>

## Content visibility

<section class="cli-option" aria-label="--show-empty-elements">

### `--show-empty-elements`

Keeps normally hidden empty headings, lists, block quotes, code blocks, and tables in the output.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="show-empty-elements"></div>

</details>

</section>

<section class="cli-option" aria-label="--no-code-guessing">

### `--no-code-guessing`

Disables heuristic detection of code block languages (unknown blocks remain plain text).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="code-guessing"></div>

</details>

</section>

## Links

<section class="cli-option" aria-label="--link-style">

### `--link-style`

<p class="cli-syntax"><code>-u, --link-style &lt;clickable|fclickable|inline|inlinetable|endtable|hide&gt;</code></p>

Changes how links are displayed (default `clickable`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="link-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--link-overflow">

### `--link-overflow`

<p class="cli-syntax"><code>-l, --link-overflow &lt;wrap|cut|tablecut|none&gt;</code></p>

Controls whether links wrap, are shortened, or overflow the available width (default `wrap`).

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="link-overflow"></div>

</details>

</section>

## Footnotes

<section class="cli-option" aria-label="--footnote-style">

### `--footnote-style`

<p class="cli-syntax"><code>--footnote-style &lt;endnotes|attached&gt;</code></p>

Places footnotes at the end of the document or immediately after the block that references them, including paragraphs, tables, and list items.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="footnote-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--missing-footnote-style">

### `--missing-footnote-style`

<p class="cli-syntax"><code>--missing-footnote-style &lt;show|hide&gt;</code></p>

Controls placeholder entries for missing, invalid, or empty footnote definitions.

- `show` renders a placeholder message in the footnote block `hide` omits those entries entirely.

<details class="mdv-reference-preview">
<summary>Output preview</summary>

<div data-mdv-preview="missing-footnote-style"></div>

</details>

</section>

## Information

<section class="cli-option" aria-label="mdv help">

### `mdv help`

Opens the full `mdv --help` output in the selected pager.

- It uses `MDV_PAGER` when set and the built-in pager otherwise; when input or output is redirected, it prints the same help directly.

</section>

<section class="cli-option" aria-label="-h">

### `-h`

Shows a concise option summary.

</section>

<section class="cli-option" aria-label="--help">

### `--help`

Shows full help with value descriptions, formats, and examples.

</section>

<section class="cli-option" aria-label="--version">

### `--version`

<p class="cli-syntax"><code>-V, --version</code></p>

Prints the current version.

</section>
