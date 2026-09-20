# Application Startup and Routing

## Entry points

| File | Responsibility |
|---|---|
| [src/main.rs](../../src/main.rs) | Minimal binary wrapper: logging, Clap parsing, and the call to `mdv::run`. |
| [src/version.rs](../../src/version.rs) | Plain build-information output for `--version` and `-V`. |
| [build/version.rs](../../build/version.rs) | Compile-time package revision, target triple, and Rust compiler version. |
| [src/lib.rs](../../src/lib.rs) | Crate root, module declarations, and routing for every execution mode. |
| [src/document.rs](../../src/document.rs) | Shared document rendering options, metadata prefixes, and refresh rendering. |
| [src/error.rs](../../src/error.rs) | Typed `MdvError` variants for configuration, themes, Markdown, rendering, monitoring, I/O, and syntax highlighting. |
| [src/monitor.rs](../../src/monitor.rs) | Standalone file-monitoring mode used after ordinary output. |

## `main.rs`

`main` intentionally contains no application logic:

1. Initialize `env_logger` with plain diagnostics.
2. Build `clap::ArgMatches` through `Cli::command()`. Handle Clap's `DisplayVersion` by printing build information; other help/error exits retain Clap behavior.
3. Construct `Cli` with `Cli::from_arg_matches`.
4. Pass both values to `mdv::run`.

`ArgMatches` remains available because configuration assembly must distinguish explicit user input from Clap-provided defaults.

Both version flags print `mdv` followed by aligned `Version`, `Debug`, `Triple`,
and `Rustc` fields before configuration or input is loaded. `Debug` reflects the
binary's `debug_assertions`; the target triple comes from Cargo's `TARGET`, with
OS/architecture from the compiled target. Version requests launch no external commands.

`build/version.rs`, selected by `package.build` in `Cargo.toml`, embeds the package
version and compiler's `--version` output. A local
`.git` directory or worktree file adds the abbreviated HEAD hash and committer
date to `Version`. Git HEAD, refs, and existing packed refs are tracked so a
revision change updates an incremental build. Source archives, crates.io
packages, and Nix sources without `.git` print the package version without Git
metadata. Failures to read an existing checkout or query the compiler fail the
build rather than embedding placeholder values.

## `lib.rs`

### Public surface

The crate publicly exposes reusable modules such as `cli`, `config`, `markdown`, `renderer`, `table`, `terminal`, `theme`, and `utils`, together with `run`. Interactive-mode and pager implementation details remain crate-private.

### `run`

`run(mut cli, matches)` evaluates branches in a fixed order:

1. `--init-config` writes the reference configuration without loading color settings.
2. The effective `Config` and pager backend are resolved, then `OutputStyle` is resolved from stdout TTY.
3. `mdv help` builds the extended help document using the effective configuration and styling policy.
4. `--preset-info` without a file prints the preset catalog.
5. `--theme-info` without a file prints active theme information.
6. `interactive::select_interactive_target` decides whether to open the document browser or page a specific file or standard input.
7. The ordinary path reads input and calls `render_document`.
8. With `--pager[=<COMMAND>]` and terminal output, the result is wrapped in `PagerDocument` and sent to the selected backend.
9. Otherwise, the result is written directly.
10. `--monitor` starts only for ordinary file output without an active pager.

This ordering prevents metadata and setup commands from opening input or initializing a renderer unnecessarily.

### Main symbols

| Symbol | Role |
|---|---|
| `show_help` | Selects regular Clap help or the full-screen help view. |
| `build_help_document` | Builds themed Markdown/ANSI content for pager help. |
| `render_document` | Runs the shared `MarkdownProcessor` → `TerminalRenderer` → ANSI/HTML pipeline. |
| `render_document_file` | Re-reads a file for the pager refresh callback. |
| `format_current_themes` | Formats the active terminal and code themes. |
| `get_input_content` | Selects a file, `-`, piped standard input, or `--from` and returns its text. |
| `strip_leading_bom` | Removes a UTF-8 BOM only when it occurs at the beginning of input. |
| `RenderedOutput` | Carries rendered text, optional line-navigation data, styling policy, and pager status-bar transparency. |

`render_document`, `render_document_file`, `RenderOptions`, and
`format_current_themes` live in `document.rs`. Input selection and mode routing
remain in `lib.rs`.

## Input handling

The application accepts three input sources:

- a path supplied as positional `FILE`;
- `-` or non-terminal standard input;
- a Markdown file selected by the interactive browser.

Input is converted to one UTF-8 `String` before the Markdown pipeline starts, so files and pipes receive identical preprocessing.

Pager rendering retains source-line metadata and prepares unnumbered, rendered-numbered, and source-numbered views. The configured mode selects the initial view; ordinary output still renders only that one configured line-number mode.

## ANSI output and HTML

`--color` and `MDV_COLOR` choose `auto`, `always`, or `never`, with CLI precedence
over environment, presets, and configuration. `auto` enables styling only when
stdout is a terminal. Piped stdin does not disable styling in a terminal.
The resolved `OutputStyle` is passed to document renderers, browser, pager,
monitor, and refresh callbacks without mutating `Config.color`.

Pager backend selection is resolved independently as explicit
`--pager=<COMMAND>` over `MDV_PAGER` over the default backend, currently
`builtin`. The special value `default` selects that default explicitly, while a
bare `--pager` enables paging without overriding the environment-selected
backend. Because color is resolved before pager dispatch, built-in and external
backends receive the same styled or plain rendering. External commands are not
started when stdout is redirected. External paging renders one selected view;
the built-in backend prepares the additional line-number and navigation views.

Disabled styling suppresses generated SGR and OSC 8, preserving visible text
and layout. Full-screen terminal-control commands remain necessary in `never`.
Fast Clap help/version/errors and stderr diagnostics are always plain.

`--html` selects `TerminalRenderer::to_html` and emits an HTML document. `--render-html` serves a different purpose: it allows HTML embedded in Markdown to become terminal elements. The options are not interchangeable.

## Ordinary monitor mode

[src/monitor.rs](../../src/monitor.rs) uses `notify` and observes `Modify` and `Create` events for one file.

- The initial render completes before event waiting begins.
- Events use a 100 ms debounce interval.
- A new `MarkdownProcessor` is created for each refresh.
- The `TerminalRenderer` is reused because configuration and themes do not change.
- Initial and repeated rendering share the same resolved `OutputStyle`.
- An individual refresh error is written to standard error without terminating the watcher loop.

The pager has a separate watcher in `src/pager/watcher.rs`. These mechanisms intentionally remain separate: ordinary monitor mode prints successive snapshots, while the pager replaces the current document in place.

## Errors

`MdvError` supplies user-facing error categories, while public application boundaries return `anyhow::Result`. File, theme, or operation context is attached where that information becomes available. Low-level helpers must propagate failures instead of substituting plausible-looking output.
