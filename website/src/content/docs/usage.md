---
title: "Usage"
description: "Files, directories, pipelines and partial-document reading."
slug: "usage"
group: "Getting started"
order: 2
---

## Read a file

Pass a Markdown file to render it in the terminal:

```sh
mdv README.md
mdv --theme nord --cols 100 README.md
```

Choose a [preset](../presets/) when you want to reuse a set of options:

```sh
mdv --preset reader README.md
```

## Read from a pipe

mdv reads standard input when used in a pipeline. A single dash selects standard input explicitly:

```sh
cat notes.md | mdv
mdv -
```

## Browse a directory

Pass a directory, or use interactive mode for the current directory:

```sh
mdv docs/
mdv -i
```

Running `mdv` without arguments in a terminal also opens the browser. See [Document browser](../browser/) for discovery rules.

## Use a pager

```sh
mdv --pager README.md
```

The built-in pager adds search, copying, source-line navigation and editor integration. Learn the controls in [Pager and editor](../pager/).

## Start at a section

```sh
mdv --from 'Installation' README.md
mdv --from 'Installation:20' README.md
```

`--from` begins at the first matching text. Adding `:20` limits the selected source range to 20 lines. `--reverse` starts from the end while keeping block formatting intact.

<div data-mdv-preview="reverse"></div>

## Parameter reference

The [complete CLI reference](../cli/) describes every option. Use [layout](../rendering/), [code blocks](../code-blocks/), [callouts](../callouts/), [lists](../lists/) and [links](../links/) for formats and examples. [Configuration](../configuration/) explains how to save the same settings in YAML.
