# Changelog

## [2.0.0] - 2025-10-18

### Bux Fixes

- Fixed a bug for comments with an extra blank line before elements that were already preceded by an empty line.
- Fixed: extra blank line before tables at the beginning of the file

### Features

- Added: new parameter --reverse
- Added: indentation for comments, matching regular text

### Changes

- Changed: default code block style to `pretty`
- Changed: code block language is now shown by default
- Renamed: --show-block-language to --no-code-language

## [1.0.0] - 2024-10-16

### Added

- Terminal Markdown viewer with ANSI-aware layout, HTML export, and syntax highlighting.
- CLI options for layout, link styles, themes, configuration files, and monitoring mode.
- YAML-based configuration loading with environment overrides.
- Integration and unit test suites covering rendering, wrapping, and link handling.
