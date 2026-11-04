---
title: "Introduction"
description: "What mdv renders and where to start."
slug: ""
group: "Getting started"
order: 0
---

## What is mdv?

mdv is a Markdown viewer for ANSI-compatible terminals, written in Rust. Read a README, follow a guide or keep a rendered document next to your editor without leaving the command line.

It works with files, directories and standard input. Choose a theme, adjust the layout and use the built-in document browser or pager when you need an interactive view.

## Key features

- **Readable Markdown:** headings, syntax-highlighted code, tables, callouts, task lists and links.
- **Your own layout:** control width, wrapping, margins, heading alignment and table behavior.
- **Themes and presets:** use built-in palettes or save a custom configuration in YAML.
- **Interactive reading:** browse Markdown files, search a document and open its source in your editor.
- **Live monitoring:** refresh the rendered view when a file changes.
- **Command-line workflows:** read pipes, control color output and export HTML.

## Quick start

Install from crates.io with Rust 1.95 or newer:

```sh
cargo install mdv
mdv README.md
```

Try a theme, watch a file or browse the current directory:

```sh
mdv --theme catppuccin README.md
mdv --monitor notes.md
mdv -i
```

## Where next?

- [Installation](./installation/) covers installers, Cargo, RPM, Arch, Nix, source builds and updates.
- [Usage](./usage/), [document browser](./browser/), [pager](./pager/) and [monitoring](./monitoring/) explain reading workflows and controls.
- [Layout](./rendering/), [code](./code-blocks/), [math](./math/), [callouts](./callouts/), [lists](./lists/), [links](./links/), [front matter](./front-matter/) and [HTML](./html/) cover rendering in detail.
- [Themes](./themes/), [theme fields](./theme-reference/), [presets](./presets/) and [syntax definitions](./syntaxes/) explain customization.
- [Configuration](./configuration/), [all YAML settings](./configuration-reference/), [environment variables](./environment/) and the [complete CLI reference](./cli/) document every available setting.

[Development](./development/) covers building and testing the project. The [GitHub repository](https://github.com/WhoSowSee/mdv) contains its source and examples.
