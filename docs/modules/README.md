# mdv Module Documentation

This section documents the internal architecture of `mdv` after large source files were split into focused modules. It is written for contributors and describes state ownership, data flow, contracts, and the responsibility of each file.

## Documentation map

| Document | Contents |
|---|---|
| [architecture.md](architecture.md) | Application layers, primary data flow, and architectural invariants. |
| [application.md](application.md) | Entry points, mode routing, input handling, monitoring, and errors. |
| [cli-configuration.md](cli-configuration.md) | CLI types, final `Config`, precedence, presets, and custom settings. |
| [markdown.md](markdown.md) | Markdown preprocessing, `pulldown-cmark`, source-line markers, and event normalization. |
| [renderer-core.md](renderer-core.md) | `TerminalRenderer`, `EventRenderer`, render state, and event dispatch. |
| [renderer-content.md](renderer-content.md) | Text, formatting, headings, lists, callouts, images, and definition lists. |
| [renderer-code.md](renderer-code.md) | Code blocks, `syntect` highlighting, labels, aliases, and math rendering. |
| [renderer-html.md](renderer-html.md) | Terminal rendering of embedded HTML and HTML tables. |
| [links-footnotes-tables.md](links-footnotes-tables.md) | Links, footnotes, and the two table-rendering layers. |
| [themes-and-styling.md](themes-and-styling.md) | Themes, ANSI styles, inline styles, user themes, and visual overrides. |
| [interactive-and-pager.md](interactive-and-pager.md) | Interactive document browser, pager, editor integration, and file watching. |
| [testing.md](testing.md) | Unit and integration test organization, fixtures, and validation commands. |
| [file-index.md](file-index.md) | Complete index of Rust source and test files with links to topic documents. |

## Suggested reading paths

- Application startup: `architecture.md` → `application.md` → `cli-configuration.md`.
- Markdown syntax changes: `markdown.md` → `renderer-core.md` → the relevant renderer document.
- Link or table changes: `links-footnotes-tables.md` → `testing.md`.
- Theme or visual override changes: `themes-and-styling.md` → `../examples/config.yaml`.
- Ownership of a specific file: `file-index.md`.

## Scope

- `src/` is the source of truth for internal contracts.
- `../examples/config.yaml` is the reference for user-facing YAML configuration.
- `tests/` defines observable CLI behavior and takes precedence if documentation differs.
- This documentation complements the user-facing root `README.md`; it focuses on code organization.

## Maintenance rule

When moving a file, changing a public type, or transferring state ownership, update the relevant topic document and [file-index.md](file-index.md). Internal helper changes normally require no documentation update when their contract and data flow remain unchanged.
