# mdv-minus

`mdv-minus` 5.7.2 is the mdv-maintained fork of `minus` 5.7.2 from the upstream `v5.7.2` tag. It remains available under the original MIT/Apache-2.0 license, and its library target intentionally keeps the `minus` name.

Depend on the fork under that library name:

```toml
minus = { package = "mdv-minus", version = "5.7.2" }
```

mdv adds a typed prompt-rendering API:

- `Pager::set_prompt_renderer` installs a renderer receiving a stable, read-only `PromptContext`.
- `Pager::clear_prompt_renderer` restores the built-in prompt without recreating the pager.
- `PromptLine`, `PromptSpan`, `PromptStyle`, `PromptColor`, and `PromptAttribute` provide left/right alignment, Unicode-aware truncation, padding, colors, attributes, and a final style reset without exposing ANSI construction to callers.
- `Pager::set_prompt_panel` and `Pager::clear_prompt_panel` manage styled lines below the status prompt; panel rows reduce the content viewport and preserve bottom anchoring when toggled.
- `Pager::set_search_prompt` replaces the `/` or `?` search prefix with validated single-line text, while `Pager::clear_search_prompt` restores the directional default. Search input is drawn on the reserved status row even when a prompt panel is visible.
- `Pager::send_message_for` displays a message for a fixed duration and uses a generation ID so an older timer cannot clear a newer message.
- `PagerState::selected_text` returns the active visible selection without ANSI or OSC control sequences, allowing custom input classifiers to choose between selection-aware and whole-document actions.
- `PromptContext::content_rows` reports the usable content height, `PromptContext::panel_rows` exposes the currently reserved panel height, and `PromptContext::max_scroll_offset` shares the pager's canonical scroll bound.
- `PromptSpan` rejects line breaks and terminal control characters. Base-prompt and message setters now report line breaks through `Result` instead of panicking while preserving their legacy ANSI-capable surface; the search-prefix setter follows the same single-line contract.
- Changing the base prompt while a renderer is active updates `PromptContext::prompt`; clearing the renderer later reveals that latest base prompt.
- The prompt is regenerated after vertical or horizontal scrolling, appends, resize/reformat operations, temporary messages, and renderer lifecycle changes.
- Prompt-panel scrolling uses synchronized full redraws so terminal scroll commands cannot move panel fragments into the document viewport.
- Entering search pauses the general event reader before the search command is queued, keeping search cancellation keys inside the search input loop.
- Search input restores and selects the last query draft. Escape closes the prompt while preserving its current text even without confirmation; Backspace or Delete removes the selected query, cursor movement collapses the selection for ordinary editing, and a manually cleared query is not restored later. Shifted characters are accepted, and editing uses Unicode character positions instead of UTF-8 byte offsets. Outside the input prompt, Escape clears active highlights while retaining the query for reuse; only the next Escape exits.
- Mouse selection maps terminal display cells to ANSI-free Unicode text coordinates, converts horizontal byte offsets back to character offsets, keeps complete grapheme clusters intact, redraws only changed rows inside a synchronized terminal update, and remains highlighted after mouse release.
- The fixed mdv selection palette (`#8F93A2` on `#1F2233`) overrides embedded SGR colors and attributes while selected, then restores the original style at the selection boundary.
- `dynamic_paging_in_place` runs the same dynamic pager without entering or leaving a terminal screen buffer, allowing callers that already own an alternate screen to hand it over without exposing the underlying terminal.
- Default navigation adds `b`/`f` for full-page movement and state-aware `Esc`, uses the panel-aware content height for full- and half-page movement, maps Space to single-line down, and maps `Ctrl+F` to forward search. The Space and `Ctrl+F` aliases stay out of mdv's help panel.

The renderer runs synchronously while the pager state is locked. Implementations must remain fast, non-blocking, and free of terminal I/O.

The fork adds direct `unicode-segmentation` and `unicode-width` dependencies for grapheme-safe prompt layout and selection geometry.

The extension exists because upstream `minus` 5.7.2 hardcodes its prompt colors and does not expose a dynamic status-line renderer.
