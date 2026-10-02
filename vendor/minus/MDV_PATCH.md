# mdv-minus

`mdv-minus` 6.0.1 is the mdv-maintained fork of `minus` 5.7.2 from the upstream `v5.7.2` tag. It remains available under the original MIT/Apache-2.0 license, and its library target intentionally keeps the `minus` name. The fork's version is independent of the upstream version.

Depend on the fork under that library name:

```toml
minus = { package = "mdv-minus", version = "6.0.1" }
```

## Changes in 6.0.1

This patch fixes redundant mutable borrows in the pager's I/O handler so the
crate passes Clippy 1.99.0 checks with warnings denied. The public API is unchanged
from 6.0.0.

## Migrating from 5.7.3

Version 6.0.0 added source-line navigation, an interactive document outline,
deferred source-view rendering, whole-document selection with `Ctrl+A`, and
configurable color depth and highlight palettes. It also fixes cursor visibility
when handing an existing screen back to the caller and restores the previous
process panic hook after each pager session.

The major version reflects these public API changes:

- `SearchOpts::cursor_position` is now `usize`, and `SearchOpts::word_index` is
  `Vec<usize>`. Use character indices rather than `u16` terminal coordinates.
- Exhaustive matches on `PromptColor` must handle the new `Reset` variant.
- Exhaustive matches on `PromptError` must handle the new `Layout(String)` variant.

## Pager extensions

mdv adds a typed prompt-rendering API:

- `Pager::set_layout_renderer` supplies width-aware mapped content for sidebar toggles and terminal resizes. `Pager::refresh_layout` explicitly requests an update after application state changes; `set_mapped_text` remains a direct content replacement.

- `LineNavigation::with_toc` attaches source-mapped outline entries to document updates. `PagerState::toc_input` routes outline controls; the panel docks beside content in wide terminals, overlays narrow content, follows scrolling, and preserves the current numbering view when jumping.

- `Pager::set_mapped_text` atomically replaces text and source navigation, anchors the viewport to the source position, and clears stale selection and navigation highlights.
- `Pager::set_output_styling` disables decorative ANSI in custom/default prompts, prompt panels, search matches, selections, and source-line navigation while preserving pager terminal-control commands.

- `Pager::set_prompt_renderer` installs a renderer receiving a stable, read-only `PromptContext`.
- `Pager::clear_prompt_renderer` restores the built-in prompt without recreating the pager.
- `PromptLine`, `PromptSpan`, `PromptStyle`, `PromptColor`, and `PromptAttribute` provide left/right alignment, Unicode-aware truncation, padding, colors, attributes, and a final style reset without exposing ANSI construction to callers.
- `Pager::set_prompt_panel` and `Pager::clear_prompt_panel` manage styled lines below the status prompt; panel rows reduce the content viewport and preserve bottom anchoring when toggled.
- `Pager::set_search_prompt` replaces the `/` or `?` search prefix with validated single-line text, while `Pager::clear_search_prompt` restores the directional default. Search input is drawn on the reserved status row even when a prompt panel is visible.
- `Pager::send_message_for` displays a message for a fixed duration and uses a generation ID so an older timer cannot clear a newer message.
- `PagerState::selected_text` returns the active visible selection without ANSI or OSC control sequences, allowing custom input classifiers to choose between selection-aware and whole-document actions.
- `LineNavigation` and `Pager::set_line_navigation` provide a source-numbered `:` navigation mode with a persistent target indicator and fixed whole-line highlight; `Esc` restores the caller's normal content and line-number mode.
- `LineNavigation::deferred` accepts a source-view renderer that runs on first navigation. `prepare_source_view` preserves the current text and navigation state when that renderer returns an error.
- `PromptContext::content_rows` reports the usable content height, `PromptContext::panel_rows` exposes the currently reserved panel height, `PromptContext::max_scroll_offset` shares the pager's canonical scroll bound, and `PromptContext::line_navigation_position` exposes the active source-line target.
- `PromptSpan` rejects line breaks and terminal control characters. Base-prompt and message setters now report line breaks through `Result` instead of panicking while preserving their legacy ANSI-capable surface; the search-prefix setter follows the same single-line contract.
- Changing the base prompt while a renderer is active updates `PromptContext::prompt`; clearing the renderer later reveals that latest base prompt.
- The prompt is regenerated after vertical or horizontal scrolling, appends, resize/reformat operations, temporary messages, and renderer lifecycle changes.
- Prompt-panel scrolling uses synchronized full redraws so terminal scroll commands cannot move panel fragments into the document viewport.
- Entering search pauses the general event reader before the search command is queued, keeping search cancellation keys inside the search input loop.
- Search input restores and selects the last query draft. Escape closes the prompt while preserving its current text and the incrementally previewed viewport even without confirmation; Backspace or Delete removes the selected query, cursor movement collapses the selection for ordinary editing, and a manually cleared query is not restored later. Shifted characters are accepted, and editing uses Unicode character positions instead of UTF-8 byte offsets. Outside the input prompt, Escape clears active highlights while retaining the query for reuse; only the next Escape exits.
- Mouse selection maps terminal display cells to ANSI-free Unicode text coordinates, converts horizontal byte offsets back to character offsets, keeps complete grapheme clusters intact, redraws only changed rows inside a synchronized terminal update, and remains highlighted after mouse release.
- The default selection background (`#2E313B`) preserves the original foreground and restores SGR styles at the selection boundary. `Pager::set_highlight_styles(HighlightStyles)` overrides selection, ordinary search, and current-search foreground/background colors; omitted colors retain their previous defaults, including adaptive search backgrounds. Incremental previews share the same palette. `PromptColor::Reset` supports terminal foreground/background resets.
- `dynamic_paging_in_place` runs the same dynamic pager without entering or leaving a terminal screen buffer, allowing callers that already own an alternate screen to hand it over without exposing the underlying terminal.
- Default navigation adds `b`/`f` for full-page movement and state-aware `Esc`, uses the panel-aware content height for full- and half-page movement, maps Space to single-line down, and maps `Ctrl+F` to forward search. The Space and `Ctrl+F` aliases stay out of mdv's help panel.

The renderer runs synchronously while the pager state is locked. Implementations must remain fast, non-blocking, and free of terminal I/O.

`Pager::set_color_depth(ColorDepth::{Ansi16, Ansi256, TrueColor})` limits document
rows and pager-generated colors, including prompts, panels, incremental search,
selection, and line navigation. True Color preserves existing output by default.
ANSI 16 uses contrasting highlight pairs. `ColorDepth::adapt` exposes the same
SGR conversion for application rendering while retaining text and OSC/control
payloads. Terminal capability detection remains the application's responsibility.

The hue conversion uses Björn Ottosson's public-domain
[sRGB-to-Oklab transform](https://bottosson.github.io/posts/oklab/).

The fork adds direct `unicode-segmentation` and `unicode-width` dependencies for grapheme-safe prompt layout and selection geometry.

The extension exists because upstream `minus` 5.7.2 hardcodes its prompt colors and does not expose a dynamic status-line renderer.

Pager sessions restore the preceding process panic hook on normal completion and
after unwinding. Search cursor positions and word indices use `usize`; screen
coordinates are bounded when drawing. Code using `SearchOpts` positions should
use text indices rather than `u16` terminal coordinates.

Runtime state is split into `state/{layout,prompt,selection}.rs`; search uses
`search/{input,incremental,highlighting}.rs`; command handling uses
`core/ev_handler/{dispatch,io,search_actions}.rs`. Companion tests preserve the
existing behavior coverage. The root package CI checks this fork independently
with `dynamic_output,search`.
