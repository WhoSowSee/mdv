---
title: "Environment variables"
description: "Configuration, pager, editor and terminal color environment settings."
slug: "environment"
group: "Reference"
order: 42
---

## Variables

### `MDV_CONFIG_PATH`

Custom path to a configuration directory; also used by `mdv --init-config` when no directory is provided.

### `MDV_EDITOR`

Editor command used by the document browser and built-in pager; takes priority over `EDITOR`.

- Known GUI editors launch asynchronously while terminal editors block until exit; Emacs and Vim modes are selected from their CLI arguments.
- Unknown commands are treated as terminal editors.
- Malformed commands and non-Unicode editor settings are reported as errors.

### `MDV_EDITOR_MODE`

Optional editor launch mode: `tui` waits for the editor to exit, while `gui` launches it asynchronously.

- When unset, the mode is detected automatically.
- Explicit `tui` may be used with GUI launchers to pause the pager; explicit `gui` overrides unknown commands but is rejected for known terminal editors to prevent both processes from controlling the same terminal.
- Invalid values and conflicts are reported in the pager without launching the editor.

### `MDV_PAGER`

Pager command, optionally with quoted paths and arguments (for example, `less -R`).

- It selects the backend for `--pager`, `mdv help`, and documents opened from the interactive browser, but does not enable paging for ordinary output by itself.
- An explicit `--pager=<COMMAND>` has higher priority; `default` selects the default backend (currently built-in `minus`), while `builtin` explicitly selects `minus`.
- Empty, malformed, non-Unicode, and recursive `mdv` commands are errors.
- Commands are split into a program and arguments and run directly without shell interpretation.

### `MDV_COLOR`

Sets `auto`, `always`, or `never`; an explicit `--color` has higher priority.
