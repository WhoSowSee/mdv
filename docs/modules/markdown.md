# Markdown Pipeline

The `markdown` module separates optional YAML front matter and converts the Markdown body into a stable stream of `pulldown_cmark::Event<'static>`. It neither chooses colors nor constructs terminal output.

## Facade

[src/markdown.rs](../../src/markdown.rs) contains:

- `MarkdownProcessor { config, options }`;
- `ParsedDocument { events, front_matter }` and `FrontMatter { raw, properties }`;
- the internal `BLANK_LINE_MARKER`;
- declarations for specialized submodules;
- re-exports for code-language detection.

The public contract remains `MarkdownProcessor::new(config)` and `parse(markdown)`. Application rendering uses the crate-internal `parse_document(markdown)` to retain front matter alongside body events.

## `parse` stages

```mermaid
flowchart LR
    A["Document string"] --> B["Extract first-line YAML front matter"]
    B --> C["preprocess_content on Markdown body"]
    C --> D["Parser::new_ext"]
    D --> E["Events and byte ranges"]
    E --> F["postprocess_events"]
    F --> G["reverse_events when enabled"]
    G --> H["ParsedDocument"]
```

`pulldown-cmark` runs through `into_offset_iter`, so every event carries a byte range in the transformed source. Those ranges support source-line numbering and restoration of omitted blank lines.

## Exact preprocessing order

[src/markdown/parsing.rs](../../src/markdown/parsing.rs) applies transformations sequentially:

1. Unless `front_matter: source` is selected, an exact `---` block is extracted only when its opening delimiter is the first line, it has an exact closing delimiter, and its YAML root is a property mapping.
2. `--from` restricts the Markdown body source-line range.
3. Tab-indented fenced code blocks are normalized.
4. Explicit `\` lines become blank-line markers.
5. Task-list termination is repaired.
6. Pretty-checkbox mode normalizes a backslash before a checkbox marker.
7. Blockquote prefixes and blank lines inside quotes are normalized.
8. Admonition syntax becomes a callout blockquote, with an explicit source-line map for retained lines and generated separators.
9. Callout markers are separated from setext headings.
10. For terminal rendering, `\(…\)` and `\[…\]` are normalized into protected math ranges after code, HTML, and link destinations are excluded.

Most transformations go through `source_lines::apply_transform`. Admonition conversion carries source positions directly through nested blocks, removed metadata, and generated separators; it does not infer those changes from a text diff.

## Module files

| File | Responsibility |
|---|---|
| [src/markdown/parsing.rs](../../src/markdown/parsing.rs) | Constructor, front matter extraction, `parse`, preprocessing, and `--from` filtering. |
| [src/markdown/admonitions.rs](../../src/markdown/admonitions.rs) | Normalize callout dialects, retain source positions, and restore per-block presentation events. |
| [src/markdown/admonitions/syntax.rs](../../src/markdown/admonitions/syntax.rs) | Dialect headers, fence terminators, and type/title parsing. |
| [src/markdown/admonitions/attributes.rs](../../src/markdown/admonitions/attributes.rs) | Balanced brackets and quoted attribute values. |
| [src/markdown/admonitions/scanner.rs](../../src/markdown/admonitions/scanner.rs) | Block conversion, indentation, and source positions. |
| [src/markdown/admonitions/boundaries.rs](../../src/markdown/admonitions/boundaries.rs) | Precomputed block endpoints, including unmatched openers. |
| [src/markdown/admonitions/protected.rs](../../src/markdown/admonitions/protected.rs) | Protected code/HTML lines, including multiline inline code. |
| [src/markdown/admonitions/containers.rs](../../src/markdown/admonitions/containers.rs) | Callouts inside Markdown blockquotes and list items. |
| [src/markdown/admonitions/metadata.rs](../../src/markdown/admonitions/metadata.rs) | Directive options, Quarto headings, and internal presentation events. |
| [src/markdown/blockquotes.rs](../../src/markdown/blockquotes.rs) | Parse `>` prefixes, nesting, and explicit blank lines inside blockquotes. |
| [src/markdown/fences.rs](../../src/markdown/fences.rs) | Find fence markers and normalize tab-indented fences without losing inner indentation. |
| [src/markdown/math.rs](../../src/markdown/math.rs) | Recognize extended TeX delimiters outside protected Markdown ranges. |
| [src/markdown/math/dollars.rs](../../src/markdown/math/dollars.rs) | Protect table separators inside dollar-delimited formulas. |
| [src/markdown/math/protected.rs](../../src/markdown/math/protected.rs) | Build protected ranges from Markdown offset events. |
| [src/markdown/task_lists.rs](../../src/markdown/task_lists.rs) | Terminate task-list blocks and normalize alternative checkbox spelling. |
| [src/markdown/structure.rs](../../src/markdown/structure.rs) | Recognize list, callout, and setext structural lines. |
| [src/markdown/events.rs](../../src/markdown/events.rs) | Postprocess offset events, source markers, and special-case indented code. |
| [src/markdown/conversion.rs](../../src/markdown/conversion.rs) | Convert borrowed events and tags to `'static`, expand tabs, and reverse events. |
| [src/markdown/detection.rs](../../src/markdown/detection.rs) | Extract explicit language hints and heuristically detect source languages. |
| [src/markdown/raw_html.rs](../../src/markdown/raw_html.rs) | Merge raw-text HTML containers such as `pre` and `textarea` into one event. |
| [src/markdown/source_lines.rs](../../src/markdown/source_lines.rs) | Encode and decode the internal source-line map. |

## Admonitions and callouts

`admonitions.rs` does not render a frame. It emits Markdown blockquotes with `[!kind]` markers. Material for MkDocs, Docusaurus, VitePress, MyST/Sphinx, Quarto, PyMdown Blocks, and GitBook headers share this path; native GitHub/Obsidian markers retain their existing parser. Reusable dialect inputs live in [the test fixture](../../tests/files/callout-dialects.md).

Fenced dialects require a matching closing delimiter. MkDocs bodies follow four-column indentation and retain blank paragraphs; the existing unindented `!!! type Title` paragraph form is still accepted. A containing fenced callout's closing delimiter also bounds the legacy paragraph form. Block endpoints are computed from the end of each scope and reused for nested lookups, including unmatched openers; boundary discovery does not recurse or retry failed nested searches. Parsed headers are passed to conversion alongside their endpoints rather than parsed again. Openers without any later closing marker of the same family are rejected without scanning their body. List and quote contexts are normalized recursively. Indentation uses the Markdown pipeline's shared tab-stop calculation, and type-name validation is shared with native callouts and custom-callout configuration.

Fenced code, indented code, HTML blocks, and multiline inline code are protected before either opening or closing callout markers are matched. Boundary protection includes code nested inside lists and quotes, while conversion traverses those containers to reach their actual callouts. Inline code within a callout title does not protect the preceding opener. Quarto promotes an initial ATX heading only at an indentation of zero to three columns; indented code remains in the body. Quarto and PyMdown ATX titles remove a closing hash sequence only when separated from the title by a space or tab, preserving text such as `C#` and escaped hashes.

Each generated header includes a document-unique placeholder. After `pulldown-cmark` parsing, the placeholder becomes a private `InlineHtml` presentation event, before ordinary marker/title events. `CalloutOptions` carries hidden-title and hidden-icon flags; fold state remains the existing `+`/`-` marker. The renderer consumes the options only for the pending blockquote, and HTML export filters the private event. Placeholders never reach visible output. Inline title code and links contribute their visible text to the label.

CSS classes and IDs are recognized as metadata; CSS, site configuration, JavaScript, and Quarto cross-reference numbering are outside the terminal renderer's scope. Native `[!kind]` still requires whitespace before a custom title.

## Code fences

`fences.rs` distinguishes container indentation from content indentation. This prevents a tab-indented fence from being parsed as an ordinary indented code block while preserving additional tabs inside the code.

`events.rs` can also demote a plain indented block to text when the original structure identifies it as a paragraph. The decision uses source byte ranges, not only the event kind.

## Extended math delimiters

Terminal parsing accepts the dollar delimiters implemented by `pulldown-cmark` plus `\(…\)` and `\[…\]`. The scanner uses offset events to exclude inline/fenced code, raw HTML, heading attributes, inline link destinations, autolinks, and reference destinations. An unclosed delimiter remains ordinary Markdown. CLI HTML export disables this normalization so its existing output contract remains unchanged.

Library callers opt into the same terminal-specific normalization with `MarkdownProcessor::with_extended_math(true)`; `MarkdownProcessor::new` alone keeps the original delimiter behavior because the processor does not know which output backend will consume its events.

Collision-checked internal placeholders protect dollar signs, table pipes, and delimiter-edge whitespace until parsing completes. They are restored in math events and in text/code/HTML events when Markdown does not recognize a formula. The math parser therefore receives the original TeX rather than Markdown escape characters. Math pipes are protected with table parsing disabled so they cannot prevent header recognition. Display delimiter lines are protected while explicit blank-line preprocessing runs, so TeX `\\` row separators remain part of the formula. Every transformation still passes through the source-line remapper.

Protected and container ranges come from one offset event stream. Reference-definition spans come from that same parser, protecting only definitions that it actually recognized and leaving following prose available for math normalization.

## Source-line markers

Source numbering is enabled only for `LineNumberTarget::Source`. In this mode:

- the initial map contains line numbers `1..=N`;
- after front matter extraction, the body map starts at its original source line;
- a line inserted during preprocessing receives `None`;
- an invisible internal marker is inserted before an event;
- skipped blank source lines receive their own marker;
- fenced math keeps its first source-line marker separate from the TeX buffer;
- `renderer/line_numbers.rs` decodes markers and removes them from output.

Markers must never reach ANSI or HTML output or contribute to visible width.

## Event postprocessing

`postprocess_events` performs four important operations:

- restore source blank markers between non-overlapping byte ranges;
- turn a synthetic blank paragraph into `Event::Html(BLANK_LINE_MARKER)`;
- merge content inside raw-text HTML containers;
- convert every event, tag, text value, and code value to owned `'static` data.

Only then does `reverse_events` run when reverse mode is enabled.

## Invariants

- Preprocessing must not change the visible meaning of valid Markdown unless the responsible option is enabled.
- Front matter is recognized only at byte zero, apart from the BOM removed by application input handling, with exact `---` delimiter lines and a top-level YAML mapping.
- Every line transformation must preserve the source-line map.
- `MarkdownProcessor` contains no terminal-specific ANSI logic.
- `code_guessing` disables language heuristics; an explicit language hint always wins.
- New syntax normalization requires a unit test in `src/markdown/tests.rs` or an integration test in the relevant `tests/` group.
