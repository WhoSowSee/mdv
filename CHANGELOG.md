# Changelog

## [2.2.0] - 2026-02-02

### Bug Fixes

- Fixed: keep code blocks looking pretty in narrow terminals
- Fixed: backslashes, blank lines, and task list spacing
- Fixed: text disappearing inside lists
- Fixed: backslash/blank-line handling in callouts and blockquotes

### Features

- Added: support for background text highlighting syntax
- Added: footnote support
- Added: basic LaTeX support
- Added: support for additional TODO marker types
- Added: callout support
- Added: new parameter `--missing-footnote-style`
- Added: short flags for callout and footnote options


## [2.1.0] - 2025-11-03

### Bux Fixes

- Fixed: made the `--theme` option case-insensitive
- Fixed: `--theme` can now accept an empty string `""`
- Fixed: missing `---` separator after text

### Refactoring

- Refactored: split tests into multiple files

### Features

- Added: new link output type `--link-style endtable`
- Added: new parameter `--code-wrap-indent`

## [2.0.0] - 2025-10-18

### Bux Fixes

- Fixed a bug for comments with an extra blank line before elements that were already preceded by an empty line
- Fixed: extra blank line before tables at the beginning of the file

### Features

- Added: new parameter `--reverse`
- Added: indentation for comments, matching regular text

### Changes

- Changed: default code block style to `pretty`
- Changed: code block language is now shown by default

### Breaking Changes

- Renamed: `--show-block-language` to `--no-code-language`

## [1.0.0] - 2024-10-16

### Added

- Terminal Markdown viewer with ANSI-aware layout, HTML export, and syntax highlighting
- CLI options for layout, link styles, themes, configuration files, and monitoring mode
- YAML-based configuration loading with environment overrides
- Integration and unit test suites covering rendering, wrapping, and link handling
