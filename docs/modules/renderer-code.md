# Renderer Code and Math

Code rendering combines a language hint, optional heuristic detection, `syntect`, the terminal palette, and one of three layout styles.

## Code-renderer files

| File | Responsibility |
|---|---|
| [event/code.rs](../../src/renderer/event/code.rs) | Shared constants and types such as `CodeBlockRenderInput`, plaintext results, and submodule registration. |
| [event/code/block.rs](../../src/renderer/event/code/block.rs) | Finish a fenced code block and select the math, plaintext, or highlighted path. |
| [event/code/inline.rs](../../src/renderer/event/code/inline.rs) | Inline code with backticks, semantic styling, and wrapping. |
| [event/code/hint.rs](../../src/renderer/event/code/hint.rs) | Separate the language hint from additional tokens. |
| [event/code/aliases.rs](../../src/renderer/event/code/aliases.rs) | Normalize and expand language aliases for syntax lookup. |
| [event/code/syntax.rs](../../src/renderer/event/code/syntax.rs) | Find a `SyntaxReference` in the loaded `SyntaxSet`. |
| [event/code/labels.rs](../../src/renderer/event/code/labels.rs) | Human-readable labels, custom icons/labels/aliases, and icon width. |
| [event/code/highlighting.rs](../../src/renderer/event/code/highlighting.rs) | Run `syntect`, emit terminal escapes, and highlight footnote markers specially. |
| [event/code/line_numbers.rs](../../src/renderer/event/code/line_numbers.rs) | Build per-block source/rendered gutters and reserve their width before wrapping. |
| [event/code/plaintext.rs](../../src/renderer/event/code/plaintext.rs) | Markdown/plaintext blocks, embedded link-reference blocks, and width estimation. |
| [event/code/rendering.rs](../../src/renderer/event/code/rendering.rs) | `basic` and `simple` layouts. |
| [event/code/pretty.rs](../../src/renderer/event/code/pretty.rs) | Pretty frames, labels, segment wrapping, and border rendering. |

## Code-block lifecycle

1. `core/start_tags.rs` enables `in_code_block` and stores the language hint.
2. Text and code events accumulate in `code_block_content`.
3. `handle_code_block_end` chooses a specialized math/plaintext path or a syntax.
4. Without an explicit syntax, `detect_source_code` may run when `code_guessing` is enabled.
5. `highlight_code` converts `syntect` spans to terminal escape sequences.
6. The `basic`, `simple`, or `pretty` renderer builds the block layout.
7. Captured or deferred reference blocks return to the owner of the current container.

`CodeBlockRenderInput` supplies layout functions with prepared values only: highlighted text, label, wrapping mode, terminal width, and original code body.

## Code-block styles

| Style | Behavior |
|---|---|
| `basic` | Small indentation without a frame; the default. |
| `simple` | Lightweight visual emphasis with an optional label. |
| `pretty` | Top and bottom borders, optional name/icon, and aligned content. |

`show-name` and `show-icon` are independent. A custom code-block definition can override the label, icon, and aliases for a language without changing its syntax definition.

## Code wrapping

Code wraps as highlighted segments, preserving color on continuation lines. `CodeWrapIndent` controls hanging indentation, while available width accounts for:

- document margins;
- blockquote and list context;
- code borders and inner padding;
- the document line-number gutter;
- the optional code-block line-number gutter.

## Code line numbers

`--code-line-numbers` and the `code_line_numbers` YAML key reset numbering for each block and apply to `basic`, `simple`, and `pretty` layouts. The default rendered target numbers every wrapped terminal row. The `source` target numbers the first segment of each physical code line and leaves wrapped continuations blank. The `separator` modifier uses the same independently themed number and separator colors as document line numbers.

The gutter width participates in wrapping before content is laid out. The renderer repeats layout until every block uses the document-wide maximum digit width and the wrapped-row counts are stable. Pretty frames include the shared gutter inside their aligned content width.

## Syntax resources

| File | Responsibility |
|---|---|
| [renderer/syntax_set.rs](../../src/renderer/syntax_set.rs) | Lazy cache of embedded syntaxes and loading of custom `.sublime-syntax` files. |
| [renderer/syntax_theme.rs](../../src/renderer/syntax_theme.rs) | `CodeHighlightTheme`: a native `syntect` theme or a palette derived from the terminal theme. |
| [renderer/syntax_theme/builder.rs](../../src/renderer/syntax_theme/builder.rs) | Build a `syntect::Theme` from semantic `SyntaxTheme` values. |
| [renderer/syntax_theme/terminal.rs](../../src/renderer/syntax_theme/terminal.rs) | Convert `syntect` spans and font-style differences to ANSI foreground/style sequences. |

`DEFAULT_THEME_SET` stores embedded `syntect` themes. For a terminal-derived code theme, the builder creates scope rules from `Theme.syntax`, and the terminal adapter serializes the resulting spans.

## Language resolution

Syntax selection follows this order:

1. an explicit normalized language token;
2. custom aliases;
3. embedded aliases such as `sh`, `shell-session`, and Objective-C variants;
4. heuristic detection, only when `code_guessing=true`;
5. plaintext, when the language is explicitly plain/Markdown or no syntax matches.

An unsupported user `.sublime-syntax` file produces a diagnosable loading error.

## Math

Math rendering does not depend on `syntect`, although fenced blocks with `math`, `latex`, `tex`, `katex`, or `mathjax` hints enter through the code-block handler.

| File | Responsibility |
|---|---|
| [src/math.rs](../../src/math.rs) | `MathMode`, `render_math`, language-hint detection, and parser facade. |
| [src/math/parser.rs](../../src/math/parser.rs) | Recursive parser for commands, groups, scripts, delimiters, and environments. |
| [src/math/rendering.rs](../../src/math/rendering.rs) | Fractions, roots, binomials, align/matrix environments, and output normalization. |
| [src/math/scripts.rs](../../src/math/scripts.rs) | Unicode superscript/subscript, spacing, delimiters, and `mathbb`. |
| [src/math/symbols.rs](../../src/math/symbols.rs) | LaTeX-command-to-Unicode-symbol table. |
| [event/math.rs](../../src/renderer/event/math.rs) | Inline/display events and fenced math blocks in the current renderer context. |

The math parser intentionally produces a terminal text approximation rather than a full TeX layout. Display math receives block spacing; inline math continues the current line.

## Invariants

- Raw code remains unchanged until the syntax/plaintext/math path is selected.
- ANSI spans do not contribute to visible width.
- Labels and icons do not affect syntax lookup.
- References found in a plaintext Markdown code block must not escape their callout or table boundary.
- A pretty border is at least as wide as every visible content line after wrapping.
