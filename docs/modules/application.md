# Application Startup and Routing

## Entry points

| File | Responsibility |
|---|---|
| [src/main.rs](../../src/main.rs) | Minimal binary wrapper: logging, Clap parsing, and the call to `mdv::run`. |
| [src/lib.rs](../../src/lib.rs) | Crate root, module declarations, and routing for every execution mode. |
| [src/error.rs](../../src/error.rs) | Typed `MdvError` variants for configuration, themes, Markdown, rendering, monitoring, I/O, and syntax highlighting. |
| [src/monitor.rs](../../src/monitor.rs) | Standalone file-monitoring mode used after ordinary output. |

## `main.rs`

`main` intentionally contains no application logic:

1. Initialize `env_logger`.
2. Build `clap::ArgMatches` through `Cli::command()`.
3. Construct `Cli` with `Cli::from_arg_matches`.
4. Pass both values to `mdv::run`.

`ArgMatches` remains available because configuration assembly must distinguish explicit user input from Clap-provided defaults.

## `lib.rs`

### Public surface

The crate publicly exposes reusable modules such as `cli`, `config`, `markdown`, `renderer`, `table`, `terminal`, `theme`, and `utils`, together with `run`. Interactive-mode and pager implementation details remain crate-private.

### `run`

`run(mut cli, matches)` evaluates branches in a fixed order:

1. `mdv help` builds the extended help document.
2. `--init-config` writes the reference configuration into the selected configuration directory.
3. The effective `Config` is assembled.
4. `--preset-info` without a file prints the preset catalog.
5. `--theme-info` without a file prints active theme information.
6. `interactive::select_interactive_target` decides whether to open the document browser or page a specific file or standard input.
7. The ordinary path reads input and calls `render_document`.
8. With `--pager` and terminal output, the result is wrapped in `PagerDocument`.
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
| `RenderedOutput` | Carries rendered text and pager status-bar transparency. |

## Input handling

The application accepts three input sources:

- a path supplied as positional `FILE`;
- `-` or non-terminal standard input;
- a Markdown file selected by the interactive browser.

Input is converted to one UTF-8 `String` before the Markdown pipeline starts, so files and pipes receive identical preprocessing.

## ANSI output and HTML

`--html` selects `TerminalRenderer::to_html` and emits an HTML document. `--render-html` serves a different purpose: it allows HTML embedded in Markdown to become terminal elements. The options are not interchangeable.

## Ordinary monitor mode

[src/monitor.rs](../../src/monitor.rs) uses `notify` and observes `Modify` and `Create` events for one file.

- The initial render completes before event waiting begins.
- Events use a 100 ms debounce interval.
- A new `MarkdownProcessor` is created for each refresh.
- The `TerminalRenderer` is reused because configuration and themes do not change.
- An individual refresh error is written to standard error without terminating the watcher loop.

The pager has a separate watcher in `src/pager/watcher.rs`. These mechanisms intentionally remain separate: ordinary monitor mode prints successive snapshots, while the pager replaces the current document in place.

## Errors

`MdvError` supplies user-facing error categories, while public application boundaries return `anyhow::Result`. File, theme, or operation context is attached where that information becomes available. Low-level helpers must propagate failures instead of substituting plausible-looking output.
