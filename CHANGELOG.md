# Changelog

## [6.0.2] - 2026-10-09

### Bug Fixes

- Fixed: pager help descriptions keep their muted color through wrapping and column changes, while shortcut keys retain the theme color ([a472d4c](https://github.com/WhoSowSee/mdv/commit/a472d4ce347e270edad71d54852166fe87ecd260))
- Fixed: `--render-html` preserves comments unless `--hide-comments` is set, including multiline text and source numbering; borderless header-only tables retain all lines ([1814e47](https://github.com/WhoSowSee/mdv/commit/1814e473f5493685457f9ed44ffecb05bb25c9a4))

### Documentation

- Added: light and dark star history charts stored in the repository and refreshed automatically for both README versions ([0ee95fb](https://github.com/WhoSowSee/mdv/commit/0ee95fb9c1a7091a68d1fd02e26fc66cf2ee4744))
- Documented: the corrected argument order for macOS `lipo` in the 6.0.1 changelog, making the already shipped build fix explicit ([2f91bf4](https://github.com/WhoSowSee/mdv/commit/2f91bf473dac5ccdafc0b0344e72d6bea8076295))
- Fixed: README star history charts now display correctly on crates.io through absolute image URLs for light and dark variants ([dcf5b92](https://github.com/WhoSowSee/mdv/commit/dcf5b92c864777afc0e3a0e5ebc5cb8188316d09))
- Localized: star history titles, star counts and month labels in README-RU, and removed headings already provided by the charts ([6a5bc6e](https://github.com/WhoSowSee/mdv/commit/6a5bc6e7a084bc5ff12646a49e242b9b15f02e9f))
- Updated: all four star history charts with current data, covering English and Russian labels in light and dark themes ([007a7f2](https://github.com/WhoSowSee/mdv/commit/007a7f282509b164007624945a962c6a68f28054))
- Updated: the overview and theme screenshots in both README versions, and shortened the sample SQL query for the new captures ([d41edb1](https://github.com/WhoSowSee/mdv/commit/d41edb1594cfcce61528b544e7370bf31a8d42d0))

### Maintenance

- Removed: the special merging of 6.0.1 and 6.0.0 release notes; each release now uses only its own changelog section ([3a6e0a2](https://github.com/WhoSowSee/mdv/commit/3a6e0a29ed5370aa7802beed29b7673293cda8c3))
- Added: contribution and AI guidelines, issue/PR templates and Rust settings; CI checks formatting and Clippy, runs tests on Linux, Windows and macOS, and reuses build workflows ([6c2bbbb](https://github.com/WhoSowSee/mdv/commit/6c2bbbbaae06743c0f6663e6dad5d67251990738))
- Fixed: automatic star history updates can publish on protected `main` through a dedicated GitHub App, with commits restricted to the four chart files ([82f3d35](https://github.com/WhoSowSee/mdv/commit/82f3d357b073ddbc3a4def76eb3acd0760f328c4))
- Added: `.editorconfig` to standardize UTF-8, LF and indentation while preserving Markdown trailing spaces used for hard line breaks ([77575d2](https://github.com/WhoSowSee/mdv/commit/77575d2c1ceeffbe6374247b190b52a15ab46eed))
- Added: `.gitattributes` to normalize text to LF, show Rust context in diffs and preserve screenshots and bundled syntax data as binary files ([aaec029](https://github.com/WhoSowSee/mdv/commit/aaec029df15ab6dc0be20c3b355d332aad5eb139))
- Added: RPM packages for x86_64/ARM64 and Arch Linux packages for x86_64; CI checks dependencies, SHA-256 and licenses, then installs and runs each package ([e477ebb](https://github.com/WhoSowSee/mdv/commit/e477ebb378224ee689853c1266f210e66f1cd06f))

## [6.0.1] - 2026-10-02

### Bug Fixes

- Fixed: lipo argument order in macOS builds
- Fixed: rendering test timeouts on Linux and macOS
- Fixed: crates.io package verification on Rust 1.99.0

### Maintenance

- Updated: mdv-minus to v6.0.1

## [6.0.0] - 2026-10-02

### Breaking Changes

- Reworked: theme structure and extended color customization
- Renamed: styling options and standardized CLI help
- Changed: color controls to `auto`, `always` and `never` modes

### Features

- Added: interactive table of contents to the built-in pager
- Added: HTML `details` styling and block spacing support
- Added: `code_block_border`, `callout_border` and `footnote_separator` line colors
- Added: `--horizontal-rule-style` and `horizontal_rule` theme color
- Expanded: callout syntax and nesting support
- Added: automatic color depth detection and palette adaptation
- Added: built-in `pretty` preset
- Added: detailed build information to `--version` output
- Added: external pager selection via `MDV_PAGER` and `--pager`
- Added: extended terminal math support
- Added: source-line navigation to the pager
- Added: Git-ignored file toggling to interactive mode

### Bug Fixes

- Fixed: monitor rendering and simplified internal modules
- Fixed: heading and callout wrapping in narrow terminals
- Fixed: cursor appearing during transitions between the browser and pager
- Fixed: code highlighting styles leaking into block borders
- Fixed: automatic interactive mode selection for redirected output
- Fixed: callout rendering in the pager

### Changes

- Improved: Markdown rendering performance and built-in pager startup
- Improved: copying and added select-all support in the pager
- Improved: pager line numbering and help

### Documentation

- Improved: test data and standardized documentation formatting

### Internal

- Added: justfile for development and project checks
- Simplified: internal modules and removed Rust antipatterns
- Removed: unused imports from the pager search module

### Maintenance

- Updated: mdv-minus to v6.0.0
- Updated: release build and packaging pipeline
- Updated: Cargo dependencies

## [5.1.0] - 2026-08-22

### Features

- Added: structured YAML support for complex configuration settings
- Added: YAML front matter support with display modes
- Added: simple icon mode for callouts
- Added: configurable line numbers to code blocks
- Added: incremental discovery and loading indicator to interactive mode

### Bug Fixes

- Fixed: pager position after cancelling search
- Fixed: file path parsing after line number flags
- Fixed: width recalculation for character wrapping in tables
- Fixed: link reference wrapping in inline tables

### Changes

- Improved: pager search and text selection
- Applied: text wrapping mode to table cells

### Internal

- Added: buffered rendering to interactive mode

### Maintenance

- Updated: mdv-minus to v5.7.3

## [5.0.0] - 2026-08-16

### Breaking Changes

- Simplified: renderer internals and removed dead `code_block` theme option
- Added: configurable pretty-list styles and uniform markers
- Reworked: code block styles and options
- Removed: code block label suppression option
- Reworked: CLI short aliases and help text

### Features

- Added: built-in and user configuration presets
- Added: custom syntax support
- Expanded: interactive pager with status bar, help, and selection
- Added: symmetric spacing for block elements
- Added: Markdown heading marker display
- Added: configurable horizontal margins
- Added: definition list support
- Expanded: pager navigation and search shortcuts
- Added: configurable line numbering
- Added: configurable table border style
- Added: configurable spacing for individual block elements
- Added: interactive document browser
- Added: scrolling past the last line in the pager
- Expanded: HTML content rendering inside tables
- Added: help subcommand using the built-in pager
- Added: pager status bar transparency setting and fixed scrolling artifacts
- Added: configurable Markdown inline element styles

### Bug Fixes

- Fixed: ANSI styles leaking into plain text in tables
- Fixed: long unbroken text overflowing in word-wrap mode
- Fixed: rendering of non-standard and empty checkboxes
- Fixed: spacing after link reference blocks in inlinetable mode
- Fixed: spacing after links from callouts and code blocks
- Fixed: no-match search handling in the pager
- Fixed: semantic HTML tag rendering inside paragraphs
- Fixed: HTML superscript and subscript rendering
- Fixed: clickable link wrapping in the pager
- Fixed: extra blank lines before tables with HTML
- Fixed: pager search and help behavior
- Fixed: theme application in the help pager
- Fixed: ARM64 deb and core24 Snap package builds

### Changes

- Changed: paragraph line break and spacing behavior

### Internal

- Consolidated: YAML assets for themes, presets, and configuration
- Split: large modules and added developer documentation
- Added: separately publishable `mdv-minus` crate for mdv-specific pager extensions

### Maintenance

- Optimized: local and CI builds
- Updated: dependencies and adapted code to new APIs

## [4.2.1] - 2026-07-15

### Bug Fixes

- Fixed: sequential link numbering across blocks in inlinetable mode
- Fixed: white code blocks on light terminal backgrounds
- Fixed: keyword color and font style in code highlighting
- Fixed: phantom empty line at the bottom of code blocks

## [4.2.0] - 2026-07-13

### Features

- Added: separate `[SVG]` marker for SVG images

### Bug Fixes

- Fixed: duplicate list marker when combining `--pretty-list` and `--pretty-checkbox`
- Fixed: leading gap in markdown code block starting with a list
- Fixed: gap before link index in HTML block with nested images
- Fixed: missing spacing after link references in HTML block

### Documentation

- Added: short aliases and `--render-html` to READMEs

## [4.1.0] - 2026-07-13

### Features

- Added: HTML rendering via `--render-html`
- Added: Nerd Font checkbox icons via `--pretty-checkbox` and `--custom-checkbox`
- Added: user theme loading from `<config_dir>/themes`
- Added: `--pretty-list` and `--custom-list` for Nerd Font list markers
- Added: Nix support via `flake.nix`

### Bug Fixes

- Fixed: silent stdin read when no file is provided
- Fixed: extra space after non-standard checkbox icons

### Documentation

- Updated: `--config-file`, `--reflow` and `--custom-code-block` help text

### Internal

- Simplified: lazy caches and table width calculation
- Removed: redundant aliases and dead code
- Updated: CI to Node 24 actions, native snap builds and artifact guards

## [4.0.0] - 2026-06-26

### Breaking Changes

- Changed: `--config-file` and `MDV_CONFIG_PATH` now accept directories containing `config.yaml` or `config.yml`

### Features

- Added: short `-S` alias for `--table-smart-indent`
- Added: `-G`/`--init-config` for creating a default configuration file
- Added: `-p` pager output flag
- Added: built-in pager and editor support
- Added: code block icons and per-language customization
- Added: `-R`/`--reflow` flag and named soft-break thresholds

### Bug Fixes

- Fixed: GIF media links rendered as `[IMAGE]`
- Fixed: long Markdown comments wrapping
- Fixed: space-indented text rendered as code block
- Fixed: parsing headings after spaces and tabs
- Fixed: soft break handling

### Documentation

- Updated: README badge markup
- Updated: icon in READMEs
- Updated: README badges to embedded SVG logos and pipeline examples

## [3.0.0] - 2026-05-16

### Breaking Changes

- Renamed: `catppucin` theme to `catppuccin`
- Renamed: `--style-code-block` to `--code-block-style`

### Bug Fixes

- Fixed: incorrect links inside nested lists
- Fixed: footnote rendering inside tables
- Fixed: indentation for code blocks before lists and inside code blocks
- Fixed: table link underline leakage
- Fixed: extra blank lines around display math blocks
- Fixed: rendering of tables and inline table links for callouts, code blocks, and blockquotes
- Fixed: tab-prefixed lines incorrectly rendered as indented code blocks
- Fixed: pretty callout wrapping that splits trailing single letters
- Fixed: callout rendering with word wrap for long unbroken lines
- Fixed: header-only table rendering

### Features

- Added: new parameter `--table-smart-indent`
- Added: `tablecut` link truncation for inline table links
- Added: support for OSC8 hyperlinks within tables
- Added: support for additional media tags beyond `[image]`

### Refactoring

- Refactored: split tests into multiple files

### Maintenance

- Updated: `syntaxes.bin`

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
