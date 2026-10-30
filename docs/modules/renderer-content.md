# Renderer Text and Block Elements

This section covers ordinary text, inline formatting, wrapping, headings, lists, blockquotes, callouts, definition lists, and media markers.

Use the [file index](file-index.md#srcrenderereventtext-and-formatting) to locate handlers.

## Text pipeline

Text is accumulated by its active code, link, table, HTML, or callout container before ordinary
paragraph rendering. Plain and styled text share wrapping rules; underlines and strikethroughs
remain continuous across fragments.

## Inline formatting

The formatting stack stores semantic `ThemeElement` values. Styles are resolved from the
active theme and inline attributes when visible text is emitted.

## Wrapping

`text/wrapping.rs` operates on fragments containing both text and style. This prevents two classes of defects:

- an escape sequence cannot be split across lines;
- a continuation line retains the correct list, blockquote, or code indentation.

Word mode extracts words and oversized units; character mode splits on extended grapheme clusters. Oversized words use the same grapheme boundaries, preserving compound emoji and combining marks. Visible width is measured with `unicode-width` after ANSI and OSC sequences are removed.

Wrapping units borrow slices from their text event; only oversized words that need rewriting allocate new strings. ANSI removal preserves its two-stage contract (SGR first, then OSC 8), and visible width uses the complete resulting Unicode string so emoji sequences remain intact across style boundaries.

Inline heading content is collected without wrapping. Heading finalization wraps it once, after adding optional Markdown markers and subtracting the actual heading prefix width; continuation lines receive that prefix exactly once.

Heading finalization replaces only the current heading span in the output buffer. It does not copy the preceding document for each heading.

Pretty-callout text reserves the frame overhead and the outer indentation saved at each active callout level. Nested frame rendering uses the remaining width of its parent, so framing does not wrap already fitted text again.

## Callout pipeline

Callouts use blockquote state. The Markdown pipeline normalizes alternative dialects into
markers; the text handler collects the type, fold state, and label before body rendering.
Private presentation events carry per-block title and icon options. The theme supplies the
type palette, while label formatting resolves custom icons and case options.

A pretty callout first accumulates logical content and is framed afterward. Handlers must not print border segments directly in the middle of the block.

The optional `callout.border` color applies to simple markers and pretty frames; titles and
icons share `callout.label`. Other containers use independent theme roles. See
[presentation overrides](themes-and-styling.md#callouts-lists-checkboxes-and-code-labels)
for color precedence.

Links, inline math, and inline code inside a pending label contribute visible text in source
order without emitting body content or URL references.

Hidden titles omit the simple header or the pretty-frame label without consuming body lines.
Per-block icon suppression leaves the independently configured fold indicator available.
Fold states never hide terminal content.

A list starting inside a pending ordinary blockquote ends callout detection for that quote, so a list item's literal `[!note]` cannot become a callout header. A new blockquote inside the list establishes its own pending state. Block-spacing classification follows the same distinction.

`show-icons` selects the Nerd Font icon map, while `show-simple-icons` selects bracketed ASCII markers. The options are mutually exclusive; custom callout icons continue to take precedence over either built-in map.

## Headings and spacing

[Heading layout](../../src/renderer/event/headings.rs) supports level-based, centered, flat,
and unindented headings, optional Markdown markers, and smart content indentation. Empty
heading placeholders are retained only when visibility settings or following content require
them. Block spacing is calculated from semantic element boundaries before rendering.

## Lists, task markers, and definition lists

List nesting owns marker positions and counters. Task checkboxes replace the list bullet;
empty items are removed before spacing and numbering are finalized. Marker configuration is
compiled in `Config`, with per-depth icons and independent ordered/unordered and task-state
colors. Definition descriptions maintain their own nesting stack and marker style.

## Soft breaks, hard breaks, and rules

Source soft breaks become spaces only when `reflow` and text wrapping are enabled; otherwise
they remain newlines. Hard breaks retain the active container prefix. Horizontal rules use
the available nested width, `horizontal_rule_style`, and the optional `horizontal_rule` color.

Explicit blank-line markers are separate from soft and hard breaks and must not accumulate with block spacing.

## Images and media

[event/images.rs](../../src/renderer/event/images.rs) represents images, video, audio, SVG,
and GIF sources as typed text markers with their label and destination. Classification uses
extensions and data URIs; media is not downloaded or displayed inline.

HTML media elements such as `img`, `video`, `audio`, and `source` are handled separately in `event/html/media.rs`, but share the same marker helpers.

## Invariants

- Inline styles apply to visible content, not structural indentation or prefixes without an explicit reason.
- Pretty frames preserve inner padding after wrapping.
- Smart heading indentation affects the content scope, not global document width.
- A checkbox replaces only the marker of a task-list item; regular and ordered items remain intact.
- Empty elements are suppressed before spacing is added, preventing orphan blank lines.
