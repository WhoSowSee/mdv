# Developer Documentation

These documents describe mdv's architecture, configuration contracts, rendering rules,
and validation procedures. Use the [file index](file-index.md) to locate implementation
and test files.

| Topic | Read when working on |
|---|---|
| [Architecture](architecture.md) | Component boundaries, data flow, and state ownership. |
| [Application](application.md) | Input, execution modes, output, monitoring, and errors. |
| [CLI and configuration](cli-configuration.md) | Option parsing, precedence, discovery, presets, and YAML. |
| [Markdown pipeline](markdown.md) | Preprocessing, supported dialects, and source-line mapping. |
| [Renderer core](renderer-core.md) | Rendering resources, event state, and document finalization. |
| [Text and blocks](renderer-content.md) | Wrapping, headings, lists, callouts, and media. |
| [Code and math](renderer-code.md) | Syntax highlighting, code layout, and mathematical expressions. |
| [Embedded HTML](renderer-html.md) | HTML fragments rendered as terminal content. |
| [Links, footnotes, and tables](links-footnotes-tables.md) | Reference scopes, table geometry, and link styling. |
| [Themes and styling](themes-and-styling.md) | Theme schema, inheritance, colors, and terminal styling. |
| [Interactive mode and pager](interactive-and-pager.md) | Browser, paging, selection, refresh, and editor integration. |
| [Testing](testing.md) | Checks, fixtures, test isolation, and regression coverage. |
| [CI and releases](ci-release.md) | Build matrix, package validation, and publication. |

Start with the architecture for changes spanning several subsystems. For configuration
examples, use [config.yaml](../examples/config.yaml); source code and tests define the
behavior when documentation differs.

Update the relevant topic when a contract or component boundary changes. Keep file
locations in the file index and place API details beside their declarations in rustdoc.
