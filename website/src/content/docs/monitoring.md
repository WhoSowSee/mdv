---
title: "Live monitoring"
description: "Watch a source file and preserve its theme and layout across updates."
slug: "monitoring"
group: "Reading"
order: 12
---

## Watch a file

```sh
mdv --monitor notes.md
```

Edit and save `notes.md` in another pane. mdv refreshes the rendered document when the source file changes.

## Keep your reading style

Combine monitoring with your usual rendering options:

```sh
mdv --monitor --theme catppuccin --cols 100 notes.md
mdv --monitor --preset reader notes.md
```

The same theme and layout settings apply when the document is rendered again.

## Work with an editor

Use a terminal split to keep the editor and rendered output visible together. To open the source directly from an interactive reading view, see [Pager and editor](../pager/).

Monitoring is a file workflow. For Markdown produced by another command, use the [standard-input workflow](../usage/#read-from-a-pipe).
