# File Index

This index reflects the current `src/` and `tests/` structure. Topic documents describe contracts in depth; use this page to locate the owner of a specific file.

## `src/` root

| File | Responsibility | Details |
|---|---|---|
| [main.rs](../../src/main.rs) | Binary entry point and Clap bootstrap. | [application](application.md) |
| [lib.rs](../../src/lib.rs) | Root API and execution-mode routing. | [application](application.md) |
| [error.rs](../../src/error.rs) | `MdvError` categories. | [application](application.md) |
| [cli.rs](../../src/cli.rs) | `Cli` structure and module facade. | [CLI/config](cli-configuration.md) |
| [config.rs](../../src/config.rs) | Effective `Config`, defaults, and helpers. | [CLI/config](cli-configuration.md) |
| [preset.rs](../../src/preset.rs) | Embedded and user presets. | [CLI/config](cli-configuration.md) |
| [block_spacing.rs](../../src/block_spacing.rs) | Per-element blank-line settings. | [CLI/config](cli-configuration.md) |
| [callout.rs](../../src/callout.rs) | Custom callout definitions. | [themes](themes-and-styling.md) |
| [checkbox.rs](../../src/checkbox.rs) | Standard checkbox icons. | [themes](themes-and-styling.md) |
| [checkbox_override.rs](../../src/checkbox_override.rs) | Custom checkbox states. | [themes](themes-and-styling.md) |
| [custom_code_block.rs](../../src/custom_code_block.rs) | Custom code labels, icons, and aliases. | [renderer code](renderer-code.md) |
| [inline_style.rs](../../src/inline_style.rs) | Semantic inline styles and overrides. | [themes](themes-and-styling.md) |
| [list_marker.rs](../../src/list_marker.rs) | Pretty and custom list markers. | [renderer content](renderer-content.md) |
| [markdown.rs](../../src/markdown.rs) | Markdown processor facade. | [Markdown](markdown.md) |
| [math.rs](../../src/math.rs) | Math parser facade. | [renderer code](renderer-code.md) |
| [monitor.rs](../../src/monitor.rs) | Ordinary `--monitor` watcher. | [application](application.md) |
| [pager.rs](../../src/pager.rs) | Pager facade. | [interactive/pager](interactive-and-pager.md) |
| [table.rs](../../src/table.rs) | Low-level table facade. | [links/tables](links-footnotes-tables.md) |
| [terminal.rs](../../src/terminal.rs) | ANSI styling and color conversion. | [themes](themes-and-styling.md) |
| [theme.rs](../../src/theme.rs) | Theme facade and public re-exports. | [themes](themes-and-styling.md) |
| [user_themes.rs](../../src/user_themes.rs) | User-theme facade. | [themes](themes-and-styling.md) |
| [utils.rs](../../src/utils.rs) | Display width, ANSI stripping, and text wrapping. | [architecture](architecture.md) |
| [editor.rs](../../src/editor.rs) | Editor discovery and launch. | [interactive/pager](interactive-and-pager.md) |

Top-level companion unit tests: [editor/tests.rs](../../src/editor/tests.rs), [list_marker/tests.rs](../../src/list_marker/tests.rs), and [utils/tests.rs](../../src/utils/tests.rs).

## `src/cli/`

| File | Responsibility |
|---|---|
| [callouts.rs](../../src/cli/callouts.rs) | Callout, checkbox, and definition-list CLI types. |
| [code_blocks.rs](../../src/cli/code_blocks.rs) | Code-block style and wrap-indent types. |
| [commands.rs](../../src/cli/commands.rs) | CLI subcommands. |
| [help.rs](../../src/cli/help.rs) | Long-help constants. |
| [layout.rs](../../src/cli/layout.rs) | Text/table wrapping and heading-layout enums. |
| [line_numbers.rs](../../src/cli/line_numbers.rs) | Line-number targets and options. |
| [links.rs](../../src/cli/links.rs) | Link and footnote enums. |
| [margins.rs](../../src/cli/margins.rs) | Horizontal-margin parser and serde support. |

Unit tests: [tests.rs](../../src/cli/tests.rs), [contract.rs](../../src/cli/tests/contract.rs), [parsing.rs](../../src/cli/tests/parsing.rs), and [styles.rs](../../src/cli/tests/styles.rs).

## `src/config/`

| File | Responsibility |
|---|---|
| [files.rs](../../src/config/files.rs) | Configuration paths, loading, and init-config writing. |
| [from_cli.rs](../../src/config/from_cli.rs) | Assemble the effective `Config`. |
| [merge.rs](../../src/config/merge.rs) | Field-aware merging. |
| [runtime.rs](../../src/config/runtime.rs) | Derived widths and compiled overrides. |

Unit tests: [tests.rs](../../src/config/tests.rs), [environment.rs](../../src/config/tests/environment.rs), [loading.rs](../../src/config/tests/loading.rs), and [writing.rs](../../src/config/tests/writing.rs).

## `src/markdown/`

| File | Responsibility |
|---|---|
| [admonitions.rs](../../src/markdown/admonitions.rs) | Admonition-to-callout conversion. |
| [blockquotes.rs](../../src/markdown/blockquotes.rs) | Blockquote preprocessing. |
| [conversion.rs](../../src/markdown/conversion.rs) | Owned events, tab expansion, and reverse mode. |
| [detection.rs](../../src/markdown/detection.rs) | Code-language extraction and detection. |
| [events.rs](../../src/markdown/events.rs) | Event postprocessing. |
| [fences.rs](../../src/markdown/fences.rs) | Tab-indented fence normalization. |
| [parsing.rs](../../src/markdown/parsing.rs) | Constructor, parsing, and preprocessing order. |
| [raw_html.rs](../../src/markdown/raw_html.rs) | Raw-text HTML event coalescing. |
| [source_lines.rs](../../src/markdown/source_lines.rs) | Source-line maps and markers. |
| [structure.rs](../../src/markdown/structure.rs) | Structural-line predicates. |
| [task_lists.rs](../../src/markdown/task_lists.rs) | Task-list normalization. |
| [tests.rs](../../src/markdown/tests.rs) | Processor regression tests. |

## `src/math/`

| File | Responsibility |
|---|---|
| [parser.rs](../../src/math/parser.rs) | Recursive math parser. |
| [rendering.rs](../../src/math/rendering.rs) | Fractions, roots, matrices, and alignment. |
| [scripts.rs](../../src/math/scripts.rs) | Superscripts, subscripts, delimiters, and literal commands. |
| [symbols.rs](../../src/math/symbols.rs) | Command-to-symbol table. |

## `src/interactive/`

| File | Responsibility |
|---|---|
| [mod.rs](../../src/interactive/mod.rs) | Target selection and terminal event loop. |
| [app.rs](../../src/interactive/app.rs) | UI actions and state transitions. |
| [browser.rs](../../src/interactive/browser.rs) | Browser, filter, and page state. |
| [discovery.rs](../../src/interactive/discovery.rs) | File discovery and fuzzy matching. |
| [screen.rs](../../src/interactive/screen.rs) | Screen facade and constants. |
| [screen/session.rs](../../src/interactive/screen/session.rs) | Raw/alternate-terminal lifecycle. |
| [screen/draw.rs](../../src/interactive/screen/draw.rs) | Main and error views. |
| [screen/header.rs](../../src/interactive/screen/header.rs) | Header, filter, and pagination. |
| [screen/help.rs](../../src/interactive/screen/help.rs) | Mini and full help. |
| [screen/style.rs](../../src/interactive/screen/style.rs) | Styling, sanitization, and truncation. |
| [screen/time.rs](../../src/interactive/screen/time.rs) | Document timestamps. |
| [screen/tests.rs](../../src/interactive/screen/tests.rs) | Screen-layout tests. |

Additional tests: [interactive_tests.rs](../../src/interactive_tests.rs).

## `src/pager/`

| File | Responsibility |
|---|---|
| [document.rs](../../src/pager/document.rs) | Pager document, screen, and callback types. |
| [page.rs](../../src/pager/page.rs) | `minus` pager setup and event loop. |
| [input.rs](../../src/pager/input.rs) | Custom keys and classifier. |
| [operations.rs](../../src/pager/operations.rs) | Refresh, clipboard, and messages. |
| [watcher.rs](../../src/pager/watcher.rs) | Targeted file watcher. |
| [footer.rs](../../src/pager/footer.rs) | Footer renderer and tests. |
| [help.rs](../../src/pager/help.rs) | Help panel and tests. |
| [tests.rs](../../src/pager/tests.rs) | Pager behavior tests. |

## `src/theme/` and `src/user_themes/`

| File | Responsibility |
|---|---|
| [theme/builtin.rs](../../src/theme/builtin.rs) | Embedded themes. |
| [theme/color_parse.rs](../../src/theme/color_parse.rs) | Color parsing and luminosity. |
| [theme/colors.rs](../../src/theme/colors.rs) | `Color`. |
| [theme/display.rs](../../src/theme/display.rs) | Theme listing and style creation. |
| [theme/element.rs](../../src/theme/element.rs) | `ThemeElement`. |
| [theme/manager.rs](../../src/theme/manager.rs) | Theme registry. |
| [theme/overrides.rs](../../src/theme/overrides.rs) | Terminal and code overrides. |
| [theme/types.rs](../../src/theme/types.rs) | `Theme` and `SyntaxTheme`. |
| [theme/tests.rs](../../src/theme/tests.rs) | Theme tests. |
| [user_themes/loading.rs](../../src/user_themes/loading.rs) | User-theme discovery and inheritance. |
| [user_themes/schema.rs](../../src/user_themes/schema.rs) | Partial YAML schema. |
| [user_themes/tests.rs](../../src/user_themes/tests.rs) | User-theme tests. |

## `src/table/`

| File | Responsibility |
|---|---|
| [layout.rs](../../src/table/layout.rs) | Cells, widths, and column blocks. |
| [rendering.rs](../../src/table/rendering.rs) | Wrapping modes and borders. |
| [whitespace.rs](../../src/table/whitespace.rs) | Boundary-space normalization using arranged column widths. |
| [links.rs](../../src/table/links.rs) | Fragmented ANSI/OSC replacements. |
| [tests.rs](../../src/table/tests.rs) | Test facade. |
| [tests/rendering.rs](../../src/table/tests/rendering.rs) | Table-layout tests. |
| [tests/styles.rs](../../src/table/tests/styles.rs) | ANSI and inline-style tests. |
| [tests/links.rs](../../src/table/tests/links.rs) | OSC and reference-replacement tests. |

## `src/renderer/`

| File | Responsibility |
|---|---|
| [mod.rs](../../src/renderer/mod.rs) | Renderer module facade. |
| [terminal.rs](../../src/renderer/terminal.rs) | Document-level renderer. |
| [line_numbers.rs](../../src/renderer/line_numbers.rs) | Number gutters and internal markers. |
| [syntax_set.rs](../../src/renderer/syntax_set.rs) | Syntax cache and loader. |
| [syntax_theme.rs](../../src/renderer/syntax_theme.rs) | Code-theme facade. |
| [syntax_theme/builder.rs](../../src/renderer/syntax_theme/builder.rs) | Terminal palette to `syntect` theme. |
| [syntax_theme/terminal.rs](../../src/renderer/syntax_theme/terminal.rs) | `syntect` spans to ANSI. |
| [syntax_theme/tests.rs](../../src/renderer/syntax_theme/tests.rs) | Theme-adapter tests. |
| [tests.rs](../../src/renderer/tests.rs) | Renderer-facade tests. |

## `src/renderer/event/core/`

| File | Responsibility |
|---|---|
| [core.rs](../../src/renderer/event/core.rs) | `EventRenderer` state owner. |
| [core/state.rs](../../src/renderer/event/core/state.rs) | Internal state types. |
| [core/constructor.rs](../../src/renderer/event/core/constructor.rs) | Constructor. |
| [core/render.rs](../../src/renderer/event/core/render.rs) | Document lifecycle. |
| [core/process.rs](../../src/renderer/event/core/process.rs) | Event dispatcher. |
| [core/start_tags.rs](../../src/renderer/event/core/start_tags.rs) | Start tags. |
| [core/end_tags.rs](../../src/renderer/event/core/end_tags.rs) | End tags and hard breaks. |
| [core/end_paragraph.rs](../../src/renderer/event/core/end_paragraph.rs) | Paragraph finalization. |
| [core/end_blockquote.rs](../../src/renderer/event/core/end_blockquote.rs) | Blockquote and callout finalization. |
| [core/end_lists.rs](../../src/renderer/event/core/end_lists.rs) | List and item finalization. |
| [core/callouts.rs](../../src/renderer/event/core/callouts.rs) | Callout kind and palette. |

## `src/renderer/event/code/`

Facade: [code.rs](../../src/renderer/event/code.rs).

| File | Responsibility |
|---|---|
| [aliases.rs](../../src/renderer/event/code/aliases.rs) | Language aliases. |
| [block.rs](../../src/renderer/event/code/block.rs) | Code-block finalization. |
| [highlighting.rs](../../src/renderer/event/code/highlighting.rs) | `syntect` highlighting. |
| [hint.rs](../../src/renderer/event/code/hint.rs) | Language-hint tokens. |
| [inline.rs](../../src/renderer/event/code/inline.rs) | Inline code. |
| [labels.rs](../../src/renderer/event/code/labels.rs) | Labels, icons, and custom definitions. |
| [plaintext.rs](../../src/renderer/event/code/plaintext.rs) | Plain and Markdown code path. |
| [pretty.rs](../../src/renderer/event/code/pretty.rs) | Pretty layout. |
| [rendering.rs](../../src/renderer/event/code/rendering.rs) | Basic and simple layouts. |
| [syntax.rs](../../src/renderer/event/code/syntax.rs) | Syntax lookup. |
| [tests.rs](../../src/renderer/event/code/tests.rs) | Code-renderer tests. |

## `src/renderer/event/text/` and `formatting/`

Facades: [text.rs](../../src/renderer/event/text.rs) and [formatting.rs](../../src/renderer/event/formatting.rs).

Text files: [callouts.rs](../../src/renderer/event/text/callouts.rs), [handling.rs](../../src/renderer/event/text/handling.rs), [segments.rs](../../src/renderer/event/text/segments.rs), [styled.rs](../../src/renderer/event/text/styled.rs), and [wrapping.rs](../../src/renderer/event/text/wrapping.rs).

Formatting files: [blockquotes.rs](../../src/renderer/event/formatting/blockquotes.rs), [borders.rs](../../src/renderer/event/formatting/borders.rs), [callout_frame.rs](../../src/renderer/event/formatting/callout_frame.rs), [callout_label.rs](../../src/renderer/event/formatting/callout_label.rs), [callout_render.rs](../../src/renderer/event/formatting/callout_render.rs), [inline.rs](../../src/renderer/event/formatting/inline.rs), and [spacing.rs](../../src/renderer/event/formatting/spacing.rs).

## `src/renderer/event/links/`, `footnotes/`, and `tables/`

Link facade: [links.rs](../../src/renderer/event/links.rs). Files: [clickable.rs](../../src/renderer/event/links/clickable.rs), [end.rs](../../src/renderer/event/links/end.rs), [inline.rs](../../src/renderer/event/links/inline.rs), [references.rs](../../src/renderer/event/links/references.rs), [start.rs](../../src/renderer/event/links/start.rs), and [wrapping.rs](../../src/renderer/event/links/wrapping.rs).

Footnote facade: [footnotes.rs](../../src/renderer/event/footnotes.rs). Files: [extraction.rs](../../src/renderer/event/footnotes/extraction.rs), [markdown.rs](../../src/renderer/event/footnotes/markdown.rs), [rendering.rs](../../src/renderer/event/footnotes/rendering.rs), and [scanning.rs](../../src/renderer/event/footnotes/scanning.rs).

Table facade: [tables.rs](../../src/renderer/event/tables.rs). Files: [layout.rs](../../src/renderer/event/tables/layout.rs) and [rendering.rs](../../src/renderer/event/tables/rendering.rs).

## `src/renderer/event/html/`

Facade: [html.rs](../../src/renderer/event/html.rs).

Files: [blockquotes.rs](../../src/renderer/event/html/blockquotes.rs), [blocks.rs](../../src/renderer/event/html/blocks.rs), [buffer.rs](../../src/renderer/event/html/buffer.rs), [buffer_helpers.rs](../../src/renderer/event/html/buffer_helpers.rs), [definitions.rs](../../src/renderer/event/html/definitions.rs), [dispatch.rs](../../src/renderer/event/html/dispatch.rs), [forms.rs](../../src/renderer/event/html/forms.rs), [layout.rs](../../src/renderer/event/html/layout.rs), [list_helpers.rs](../../src/renderer/event/html/list_helpers.rs), [lists.rs](../../src/renderer/event/html/lists.rs), [media.rs](../../src/renderer/event/html/media.rs), [media_helpers.rs](../../src/renderer/event/html/media_helpers.rs), [styles.rs](../../src/renderer/event/html/styles.rs), [table_cells.rs](../../src/renderer/event/html/table_cells.rs), [table_helpers.rs](../../src/renderer/event/html/table_helpers.rs), [tables.rs](../../src/renderer/event/html/tables.rs), [text.rs](../../src/renderer/event/html/text.rs), and [text_helpers.rs](../../src/renderer/event/html/text_helpers.rs).

## Other event handlers

| File | Responsibility |
|---|---|
| [event/mod.rs](../../src/renderer/event/mod.rs) | Event-module facade. |
| [event/definition_lists.rs](../../src/renderer/event/definition_lists.rs) | Definition-list state and handlers. |
| [event/headings.rs](../../src/renderer/event/headings.rs) | Heading layouts and smart indentation. |
| [event/images.rs](../../src/renderer/event/images.rs) | Markdown media markers. |
| [event/math.rs](../../src/renderer/event/math.rs) | Math events and blocks. |
| [event/misc.rs](../../src/renderer/event/misc.rs) | HTML bridge, rules, footnote references, and task markers. |
| [event/soft_breaks.rs](../../src/renderer/event/soft_breaks.rs) | Soft-break and reflow behavior. |
| [event/spacing.rs](../../src/renderer/event/spacing.rs) | Prepared block-spacing sequence. |

## Integration tests

Harness: [tests/integration.rs](../../tests/integration.rs).

- Callouts: [callouts.rs](../../tests/callouts.rs), [basic.rs](../../tests/callouts/basic.rs), [customization.rs](../../tests/callouts/customization.rs), [formatting.rs](../../tests/callouts/formatting.rs), [heading_layout.rs](../../tests/callouts/heading_layout.rs), and [tables_links.rs](../../tests/callouts/tables_links.rs).
- Checkboxes: [checkboxes.rs](../../tests/checkboxes.rs) and files under [tests/checkboxes/](../../tests/checkboxes/basic.rs).
- CLI: [cli_basic.rs](../../tests/cli_basic.rs) and files under [tests/cli_basic/](../../tests/cli_basic/general.rs).
- Code blocks: [code_blocks.rs](../../tests/code_blocks.rs) and files under [tests/code_blocks/](../../tests/code_blocks/basic.rs).
- Footnotes: [footnotes.rs](../../tests/footnotes.rs) and files under [tests/footnotes/](../../tests/footnotes/attached.rs).
- Layout: [layout.rs](../../tests/layout.rs) and files under [tests/layout/](../../tests/layout/headings.rs).
- Links and tables: [links_tables.rs](../../tests/links_tables.rs) and files under [tests/links_tables/](../../tests/links_tables/basic.rs).
- Standalone groups: [definition_lists.rs](../../tests/definition_lists.rs), [html_table_content.rs](../../tests/html_table_content.rs), [inline_styles.rs](../../tests/inline_styles.rs), [line_numbers.rs](../../tests/line_numbers.rs), [math.rs](../../tests/math.rs), [media.rs](../../tests/media.rs), [syntax_palette.rs](../../tests/syntax_palette.rs), and [visibility.rs](../../tests/visibility.rs).

### Complete nested integration-file list

- Callout formatting: [headings.rs](../../tests/callouts/formatting/headings.rs), [rules.rs](../../tests/callouts/formatting/rules.rs), and [wrapping.rs](../../tests/callouts/formatting/wrapping.rs).
- Callout tables and links: [inline_links.rs](../../tests/callouts/tables_links/inline_links.rs), [references.rs](../../tests/callouts/tables_links/references.rs), and [tables.rs](../../tests/callouts/tables_links/tables.rs).
- Checkboxes: [basic.rs](../../tests/checkboxes/basic.rs), [colors.rs](../../tests/checkboxes/colors.rs), [custom_states.rs](../../tests/checkboxes/custom_states.rs), and [layout_and_lists.rs](../../tests/checkboxes/layout_and_lists.rs).
- CLI: [general.rs](../../tests/cli_basic/general.rs), [config_and_pager.rs](../../tests/cli_basic/config_and_pager.rs), [html_content.rs](../../tests/cli_basic/html_content.rs), [html_lists_tables.rs](../../tests/cli_basic/html_lists_tables.rs), [html_semantics.rs](../../tests/cli_basic/html_semantics.rs), and [rendering_options.rs](../../tests/cli_basic/rendering_options.rs).
- Code blocks: [basic.rs](../../tests/code_blocks/basic.rs), [blockquotes.rs](../../tests/code_blocks/blockquotes.rs), [icons.rs](../../tests/code_blocks/icons.rs), [pretty.rs](../../tests/code_blocks/pretty.rs), [spacing.rs](../../tests/code_blocks/spacing.rs), [tab_indent.rs](../../tests/code_blocks/tab_indent.rs), and [wrap_indent.rs](../../tests/code_blocks/wrap_indent.rs).
- Tab-indented code blocks: [deep_indent.rs](../../tests/code_blocks/tab_indent/deep_indent.rs), [fences.rs](../../tests/code_blocks/tab_indent/fences.rs), and [paragraphs.rs](../../tests/code_blocks/tab_indent/paragraphs.rs).
- Footnotes: [attached.rs](../../tests/footnotes/attached.rs), [ordering.rs](../../tests/footnotes/ordering.rs), [placement.rs](../../tests/footnotes/placement.rs), and [validation.rs](../../tests/footnotes/validation.rs).
- Layout: [blockquotes.rs](../../tests/layout/blockquotes.rs), [headings.rs](../../tests/layout/headings.rs), [margins.rs](../../tests/layout/margins.rs), and [spacing.rs](../../tests/layout/spacing.rs).
- Layout spacing: [backslash_basic.rs](../../tests/layout/spacing/backslash_basic.rs), [backslash_blocks.rs](../../tests/layout/spacing/backslash_blocks.rs), [block_spacing.rs](../../tests/layout/spacing/block_spacing.rs), [inline_html.rs](../../tests/layout/spacing/inline_html.rs), and [paragraphs.rs](../../tests/layout/spacing/paragraphs.rs).
- Links and tables: [basic.rs](../../tests/links_tables/basic.rs), [code_blocks.rs](../../tests/links_tables/code_blocks.rs), [references.rs](../../tests/links_tables/references.rs), [smart_indent.rs](../../tests/links_tables/smart_indent.rs), and [truncation.rs](../../tests/links_tables/truncation.rs).
- Basic links and tables: [links.rs](../../tests/links_tables/basic/links.rs) and [tables.rs](../../tests/links_tables/basic/tables.rs).
- Link references: [blockquotes.rs](../../tests/links_tables/references/blockquotes.rs), [collections.rs](../../tests/links_tables/references/collections.rs), and [spacing_and_wrapping.rs](../../tests/links_tables/references/spacing_and_wrapping.rs).

See [testing.md](testing.md) for the complete test-organization guide.
