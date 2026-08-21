# Renderer Text and Block Elements

This section covers ordinary text, inline formatting, wrapping, headings, lists, blockquotes, callouts, definition lists, and media markers.

## Text pipeline

| File | Responsibility |
|---|---|
| [event/text.rs](../../src/renderer/event/text.rs) | Shared highlight/callout parsing types and text-submodule registration. |
| [event/text/handling.rs](../../src/renderer/event/text/handling.rs) | Main `handle_text` entry point and state-dependent routing. |
| [event/text/segments.rs](../../src/renderer/event/text/segments.rs) | Split text into highlighted and non-highlighted segments. |
| [event/text/styled.rs](../../src/renderer/event/text/styled.rs) | Regular, underlined, and strikethrough paths. |
| [event/text/wrapping.rs](../../src/renderer/event/text/wrapping.rs) | Word and character wrapping with styles and hanging indentation. |
| [event/text/callouts.rs](../../src/renderer/event/text/callouts.rs) | Buffer and parse `[!kind]`, a fold marker, and a custom title. |

`handle_text` first checks active containers: code block, link, table cell, HTML buffer, footnote scan, and callout marker. Only ordinary text enters the shared styled-wrapping pipeline.

## Inline formatting

| File | Responsibility |
|---|---|
| [event/formatting.rs](../../src/renderer/event/formatting.rs) | Shared imports, the default callout icon, and layout metadata helpers. |
| [event/formatting/inline.rs](../../src/renderer/event/formatting/inline.rs) | Semantic style stack, highlighting, and inline-backtick synchronization. |
| [event/formatting/spacing.rs](../../src/renderer/event/formatting/spacing.rs) | Contextual newlines, prefixes, blank-line normalization, and effective width. |
| [event/formatting/blockquotes.rs](../../src/renderer/event/formatting/blockquotes.rs) | Quote prefixes and available width for nested blockquotes. |
| [event/formatting/borders.rs](../../src/renderer/event/formatting/borders.rs) | Code/callout border pieces and display-width prefix trimming. |

`formatting_stack` stores `ThemeElement` values rather than prepared ANSI fragments. At emission time, `create_style(theme, element)` combines the semantic theme with inline-style attributes.

## Wrapping

`text/wrapping.rs` operates on fragments containing both text and style. This prevents two classes of defects:

- an escape sequence cannot be split across lines;
- a continuation line retains the correct list, blockquote, or code indentation.

Word mode extracts words and oversized units; character mode splits on Unicode characters. Visible width is measured with `unicode-width` after ANSI and OSC sequences are removed.

## Callout pipeline

A callout passes through several stages:

1. `markdown/admonitions.rs` converts alternative syntax into a blockquote marker.
2. `text/callouts.rs` buffers initial characters and parses marker, type, fold state, and title.
3. `core/callouts.rs` selects the semantic kind and color.
4. `formatting/callout_label.rs` builds the label and selected Nerd Font or portable ASCII icon and applies case options.
5. Ordinary text renders inside callout state.
6. On close, pretty style passes through `callout_render.rs` and `callout_frame.rs`.

| File | Responsibility |
|---|---|
| [formatting/callout_label.rs](../../src/renderer/event/formatting/callout_label.rs) | Label override, default/custom icon, fold icon, and header. |
| [formatting/callout_render.rs](../../src/renderer/event/formatting/callout_render.rs) | Assemble the complete pretty-callout block. |
| [formatting/callout_frame.rs](../../src/renderer/event/formatting/callout_frame.rs) | Frame, padding, in-frame wrapping, and single-character-tail normalization. |

A pretty callout first accumulates logical content and is framed afterward. Handlers must not print border segments directly in the middle of the block.

`show-icons` selects the Nerd Font icon map, while `show-simple-icons` selects bracketed ASCII markers. The options are mutually exclusive; custom callout icons continue to take precedence over either built-in map.

## Headings and spacing

[event/headings.rs](../../src/renderer/event/headings.rs) owns:

- heading-level mapping to `ThemeElement` and `BlockElement`;
- `level`, `center`, `flat`, and `none` layouts;
- smart heading and content indentation;
- optional Markdown markers;
- placeholders for empty headings.

[event/spacing.rs](../../src/renderer/event/spacing.rs) maps the event sequence to `BlockElement` values in advance, keeping per-element spacing independent of incidental intermediate events.

## Lists, task markers, and definition lists

- List starts, ends, and item boundaries are handled by `core/start_tags.rs` and `core/end_lists.rs`.
- [event/misc.rs](../../src/renderer/event/misc.rs) selects a regular, pretty, or custom marker and removes the bullet before a checkbox.
- [event/definition_lists.rs](../../src/renderer/event/definition_lists.rs) maintains a separate description stack and applies `PrettyDefinitionStyle`.
- `list_marker::ListMarkerConfig` is already compiled in `Config`; the renderer only resolves the marker for the current depth.

## Soft breaks, hard breaks, and rules

| File | Behavior |
|---|---|
| [event/soft_breaks.rs](../../src/renderer/event/soft_breaks.rs) | Preserve the source break or collapse it during reflow. |
| `core/end_tags.rs` | Render a hard break with the active container prefix. |
| [event/misc.rs](../../src/renderer/event/misc.rs) | Render a horizontal rule at the available width and nesting level. |

Explicit blank-line markers are separate from soft and hard breaks and must not accumulate with block spacing.

## Images and media

[event/images.rs](../../src/renderer/event/images.rs) classifies extensions and data URIs, then builds a terminal marker for an image, video, audio file, SVG, or GIF. Destination and alternative text accumulate between `handle_image_start` and `handle_image_end`.

HTML media elements such as `img`, `video`, `audio`, and `source` are handled separately in `event/html/media.rs`, but share the same marker helpers.

## Invariants

- Inline styles apply to visible content, not structural indentation or prefixes without an explicit reason.
- Pretty frames preserve inner padding after wrapping.
- Smart heading indentation affects the content scope, not global document width.
- A checkbox replaces only the marker of a task-list item; regular and ordered items remain intact.
- Empty elements are suppressed before spacing is added, preventing orphan blank lines.
