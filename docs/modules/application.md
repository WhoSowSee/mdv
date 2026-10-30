# Application Startup and Routing

[src/main.rs](../../src/main.rs) parses arguments and calls `mdv::run` in
[src/lib.rs](../../src/lib.rs). The application retains `ArgMatches` so configuration
assembly can distinguish explicit options from Clap defaults.

## Mode selection

Help, version, configuration generation, and palette/preset listings can complete without
reading Markdown. `--init-config` writes the reference template before color settings are
loaded. Other rendering modes resolve configuration, pager selection, and output styling
before opening a document.

| Input or option | Behavior |
|---|---|
| `--preset-info` or `--theme-info` without a file | Print metadata and return. |
| No arguments with terminal stdin and stdout | Browse the current directory. |
| A directory, or explicit `--interactive` | Select an interactive target. |
| `--pager` with terminal stdout | Open the selected pager, bypassing the browser. |
| A file with `--monitor`, without active paging | Print the initial document and watch for changes. |
| A file, `-`, or piped stdin | Render once; redirected stdout suppresses paging. |

`mdv help` uses the configured pager when both input and output are terminals and prints
directly otherwise. Pager command precedence and validation are defined in
[Pager policy](cli-configuration.md#pager-policy).

## Input and document rendering

[src/document.rs](../../src/document.rs) provides the shared parsing and rendering path.
Files and stdin are read as UTF-8, with a leading BOM removed before front matter recognition.
The same normalization applies when a file is refreshed.

Ordinary output renders the configured line-number view. The built-in pager retains source
metadata and prepares additional views on demand; see [PagerDocument](interactive-and-pager.md#pagerdocument).
The monitor supplies a prepared renderer so refreshes reuse theme and syntax resources.

## Output

Application routing resolves `OutputStyle` from the destination and color settings once,
then passes it to rendering, the browser, the pager, and refresh callbacks. Piped stdin
does not disable colors when stdout is a terminal. Styling settings and depth detection
are described in [Color policy](cli-configuration.md#color-policy).

Disabled styling suppresses generated SGR and OSC 8 sequences while retaining layout and
visible symbols. Full-screen modes still need cursor, screen-buffer, and mouse controls.
Clap diagnostics, version output, and stderr diagnostics are plain.

`--html` exports a document through `TerminalRenderer::to_html`. `--render-html` enables
[terminal rendering of embedded HTML](renderer-html.md).

## Version information

[src/version.rs](../../src/version.rs) prints `Version`, `Debug`, `Triple`, and `Rustc`
for `--version` and `-V`, before configuration or input is loaded. `Debug` reflects
`debug_assertions`; `Triple` identifies the compiled target.

[build/version.rs](../../build/version.rs) embeds the package version and compiler version.
Git checkouts also include the abbreviated HEAD and committer date. HEAD, refs, and packed
refs are tracked for incremental builds. Sources without `.git`, including crates.io
packages, print the package version alone. Version requests run no external commands;
failures to read an existing checkout or query the compiler fail the build.

## Ordinary monitor mode

[src/monitor.rs](../../src/monitor.rs) watches one file for `Modify` and `Create` events.
The watcher is registered before the initial snapshot is printed; the readiness message
follows that snapshot. Updates are coalesced until 100 ms after the last relevant event.

Each refresh parses the new source with the existing renderer and the original output
options, including front matter, HTML export, and styling. Refresh errors go to stderr
without terminating the watch loop. The public `monitor::watch_file` entry point uses
default terminal-output options; application routing supplies CLI options through
`watch_file_with_options`.

The pager's [watcher](interactive-and-pager.md#watcher) replaces the displayed document
in place, while ordinary monitor mode prints successive snapshots.

## Errors

`MdvError` identifies configuration, theme, parsing, rendering, monitoring, I/O, and syntax
errors. Application boundaries return `anyhow::Result` with file or operation context.
Low-level errors propagate to the boundary that can explain them.

[src/output.rs](../../src/output.rs) uses fallible writes and flushes. A `BrokenPipe`
caused by a downstream reader exiting early is successful completion; other output
errors remain failures.
