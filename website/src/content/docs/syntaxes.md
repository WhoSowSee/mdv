---
title: "Custom syntaxes"
description: "Load Sublime Syntax definitions and connect their language aliases."
slug: "syntaxes"
group: "Customization"
order: 33
---

## Load definitions

`--syntaxes-dir DIR` recursively loads `.sublime-syntax` files on top of the embedded syntax set. Custom definitions take precedence over matching embedded syntaxes. A missing directory or invalid definition is an error, rather than a silent switch to plain text.

```sh
mdv --syntaxes-dir ./syntaxes README.md
```

## Minimal example

Save this as `syntaxes/Acme.sublime-syntax`:

```yaml
%YAML 1.2
---
name: Acme
file_extensions: [acme]
scope: source.acme
contexts:
  main:
    - match: '\b(let|print)\b'
      scope: keyword.control.acme
```

Use `acme` as the Markdown code fence's language hint. Define icons, display names and additional aliases separately:

```sh
mdv --syntaxes-dir ./syntaxes --custom-code-block 'acme:label=Acme,icon=*,aliases=ac' README.md
```

## Configuration paths and detection

`syntaxes_dir` saves the directory in YAML.

- Relative YAML paths start from the configuration directory; a CLI relative path is relative to the current working directory.
- Custom aliases affect lookup, while custom code labels alone do not add a grammar.
- `--no-code-guessing` disables heuristic detection without disabling explicit syntax selection.
