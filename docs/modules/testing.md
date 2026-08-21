# Testing

The project combines unit tests beside implementation modules with one integration harness that runs the compiled binary through `assert_cmd`.

## Commands

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --no-fail-fast
cargo build --release
```

Use `cargo test <name> -- --nocapture` to inspect a specific test's standard output.

## Unit tests

After large source files were split, large inline `mod tests` blocks moved into companion modules. The primary groups are:

| Implementation | Tests |
|---|---|
| `src/cli.rs` | `src/cli/tests.rs` and `src/cli/tests/{contract,parsing,styles}.rs` |
| `src/config.rs` | `src/config/tests.rs` and `src/config/tests/{loading,environment,writing}.rs` |
| `src/markdown.rs` | `src/markdown/tests.rs`, with additional tests in `raw_html.rs` and `source_lines.rs` |
| `src/table.rs` | `src/table/tests.rs` and `src/table/tests/{rendering,styles,links}.rs` |
| `src/theme.rs` | `src/theme/tests.rs` |
| `src/user_themes.rs` | `src/user_themes/tests.rs` |
| `src/pager.rs` | `src/pager/tests.rs`, plus tests in `footer.rs` and `help.rs` |
| `src/interactive/browser.rs` | `src/interactive/browser/tests.rs`, plus screen tests in `src/interactive/screen/tests.rs` |
| `src/editor.rs` | `src/editor/tests.rs` |
| `src/list_marker.rs` | `src/list_marker/tests.rs` |
| `src/utils.rs` | `src/utils/tests.rs` |
| Renderer | `renderer/tests.rs`, `event/code/tests.rs`, `syntax_theme/tests.rs`, and local modules |

Unit tests cover parsers, semantic defaults, width helpers, state transitions, and internal invariants that are difficult to observe through the CLI alone.

## Integration harness

[tests/integration.rs](../../tests/integration.rs) explicitly includes topic modules because `Cargo.toml` defines one `integration` target and disables automatic creation of separate test crates.

| Group | Primary coverage |
|---|---|
| `tests/cli_basic*` | Help/version, input modes, embedded HTML, themes, presets, configuration, and pager routing. |
| `tests/callouts*` | Syntax, custom styles, wrapping, frames, headings, tables, and links. |
| `tests/checkboxes*` | Shapes, custom states/colors, lists, and nested indentation. |
| `tests/code_blocks*` | Styles, labels/icons, line-number gutters, blockquotes, spacing, wrapping, and tab-indented fences. |
| `tests/layout*` | Headings, blockquotes, margins, blank lines, reflow, and block spacing. |
| `tests/links_tables*` | Link styles, truncation, references, smart indentation, and code-block interaction. |
| `tests/footnotes*` | Attached/endnote modes, ordering, placement, and invalid or missing definitions. |
| `tests/front_matter.rs` | Strict first-line YAML recognition, display modes, HTML, line numbers, and reverse mode. |
| `tests/definition_lists.rs` | Markdown definition-list rendering. |
| `tests/html_table_content.rs` | Block and inline content inside HTML table cells. |
| `tests/inline_styles.rs` | Semantic attributes and theme overrides. |
| `tests/line_numbers.rs` | Source/rendered targets and gutters. |
| `tests/math.rs` | Inline, display, and fenced math. |
| `tests/media.rs` | Image, video, and audio markers. |
| `tests/syntax_palette.rs` | Code-theme palette and ANSI output. |
| `tests/visibility.rs` | Empty elements, comments, and visibility options. |

## Nested integration modules

Large topic files act as facades with explicit `#[path = "..."]` declarations:

- `tests/cli_basic/`: general behavior, HTML content/semantics/lists, rendering options, configuration, and pager;
- `tests/callouts/formatting/`: wrapping, rules, and headings;
- `tests/callouts/tables_links/`: tables, references, and inline links;
- `tests/checkboxes/`: basic behavior, colors, custom states, and list layout;
- `tests/code_blocks/tab_indent/`: fences, deep indentation, and paragraphs;
- `tests/layout/spacing/`: backslashes, paragraphs, block spacing, and inline HTML;
- `tests/links_tables/basic/` and `references/`: core table/link behavior and reference scopes.

Shared helpers remain in the facade module and are imported by child tests through `use super::*`.

## Fixtures

- `tests/files/` contains reusable Markdown fixtures.
- A small, scenario-specific document is created with `NamedTempFile` directly in its test.
- `docs/examples/config.yaml` is the reference for the complete configuration schema.
- Root `test_*.md` files support manual visual checks and do not replace assertions.

## Minimum checks by change

| Change | Minimum validation |
|---|---|
| CLI or configuration | Unit parsing/merge tests and the corresponding `cli_basic` scenario. |
| Markdown preprocessing | Unit event test plus an end-to-end rendering case. |
| Wrapping or layout | Unicode width, ANSI-stripped width, and a narrow-terminal case. |
| Links or tables | Plain text, ANSI colors, OSC 8, and fragmented wrapping. |
| Themes | YAML parsing, overrides, and `--no-colors`. |
| Pager or interactive mode | Pure key/action helpers; manually exercise terminal integration when necessary. |

## Test invariants

- A test asserts a user-visible effect or a subtle internal contract rather than duplicating implementation.
- ANSI assertions distinguish visible content from escape sequences.
- Temporary environment variables are protected by the shared mutex and restored by a guard.
- Add a regression to an existing topic group unless a separate module materially improves navigation.
