# Renderer Code and Math

Code rendering combines a language hint, optional heuristic detection, `syntect`, the terminal palette, and one of three layout styles.

See the [file index](file-index.md#srcrenderereventcode) for code handlers and
[math modules](file-index.md#srcmath).

## Code-block rendering

Code content and its language hint are buffered until the block closes. The handler selects
math, Markdown/plaintext, or syntax highlighting, then passes prepared text, label, wrapping
mode, width, and original source to the chosen layout.

The Markdown/plaintext path parses content with a nested processor and renderer. Its depth
guard prevents repeated recursive interpretation. Definitions and reference blocks are
reconciled with the enclosing callout, table, or document so they retain their scope.

Footnote markers use the link style. After each marker, the complete source SGR
state is restored so following code retains its colors and font attributes.

## Code-block styles

| Style | Behavior |
|---|---|
| `basic` | Small indentation without a frame; the default. |
| `simple` | Lightweight visual emphasis with an optional label. |
| `pretty` | Top and bottom borders, optional name/icon, and aligned content. |

`show-name` and `show-icon` are independent visibility options. A custom code-block definition can override the label, icon, and aliases for a language without changing its syntax definition. `code_block.label` colors the combined language name and icon in every layout. Plain labels are truncated and wrapped before their common color is applied.

The optional `code_block.border` theme color applies to both vertical and horizontal frame pieces. Without it, both styles use the terminal text color.

## Code wrapping

Code wraps as highlighted segments, preserving color on continuation lines. `CodeWrapIndent` controls hanging indentation, while available width accounts for:

- document margins;
- blockquote and list context;
- code borders and inner padding;
- the document line-number gutter;
- the optional code-block line-number gutter.

The first row uses the full available width. Continuation rows reserve the selected hanging indent before wrapping by words or characters, so a full row does not lose its indent.

Shared ANSI wrapping stores a bounded set of SGR attributes for continuation prefixes. Superseded colors do not accumulate, keeping the output size linear in the source length. Reset detection distinguishes zero-valued RGB components and color indexes from reset commands. Wrap-boundary ordering is preserved.

## Code line numbers

`--code-line-numbers` and the `code_line_numbers` YAML key reset numbering for each block and apply to `basic`, `simple`, and `pretty` layouts. The default rendered target numbers every wrapped terminal row. The `source` target numbers the first segment of each physical code line and leaves wrapped continuations blank. The `separator` modifier uses the same independently themed number and separator colors as document line numbers.

The gutter width participates in wrapping before content is laid out. The renderer repeats layout until every block uses the document-wide maximum digit width and the wrapped-row counts are stable. Pretty frames include the shared gutter inside their aligned content width.

## Syntax resources

Embedded `syntect` themes provide standalone code palettes. A terminal-derived code theme
maps `Theme.syntax` roles to syntax scopes.

The syntax set is loaded from `assets/syntaxes.bin` through a shared fallible cache.
User `.sublime-syntax` definitions under `syntaxes_dir` take priority over the embedded set.
A missing directory, invalid definition, or invalid embedded dump fails renderer construction.

## Language resolution

Syntax selection follows this order:

1. an explicit normalized language token;
2. custom aliases;
3. embedded aliases such as `sh`, `shell-session`, and Objective-C variants;
4. heuristic detection, only when `code_guessing=true`;
5. plaintext, when the language is explicitly plain/Markdown or no syntax matches.

## Math

Math rendering does not depend on `syntect`, although fenced blocks with `math`, `latex`, `tex`, `katex`, or `mathjax` hints enter through the code-block handler.

The parser builds a math tree before rendering. Inline math becomes one compact Unicode sequence; display math becomes a `MathBox` with a line buffer, width, and baseline (its height is the line count), allowing fractions and matrix cells to compose without losing alignment. Ordinary source newlines collapse to spaces and an explicit TeX `\\` creates a mathematical row.

The implementation covers fractions, roots with explicit parenthesized scope, compact scripts, common symbols and operators, `operatorname*` limits, Unicode math alphabets, vertical binomials, accents, `overset`/`underset`, braces, `align`, `gather`, matrices, `cases`, and basic `array` column specifications. Unsupported commands and environments retain their original TeX. Structural errors retain the entire original formula and produce a deduplicated warning.

Fonts apply to supported AST text and symbol nodes before composition; unsupported source remains literal, including physical-line whitespace. Mathematical scripts convert atomically and use a parenthesized form when a complete expression cannot become Unicode scripts. HTML `sup`/`sub` uses a separate conversion that retains spaces. Inline and display accents share the single-grapheme, single-column predicate.

Ordinary-line alphabet glyphs, relation symbols, arrows, and fraction slashes cannot qualify as Unicode script conversions. Punctuation can accompany converted characters, but a punctuation-only script keeps its explicit marker. Fraction arguments with nested fractions retain an outer group unless one balanced pair encloses the entire rendered argument.

Named operators retain an AST body inside a `Compact` node, which keeps their original inline representation in display layout. A `mathbb` argument containing unsupported source also retains a `Compact` AST body; only fully supported arguments may be reduced to a symbol or plain text. This preserves unsupported descendants through nested wrappers and outer fonts.

Flat display expressions can wrap by words or grapheme clusters. Structured layouts remain indivisible even when their compact result has one line; if they exceed the available width, mdv preserves their geometry and reports the overflow. Math lines inside pretty callouts carry temporary metadata at the boundary between the container prefix and mathematical content. Callout frames containing these lines preserve protection at every nesting level. Event-renderer finalization removes the metadata after all framing completes. `math_block_style` selects the math-only `basic`, `simple`, or `pretty` container and is unrelated to code block styles and code-line numbering.

Fenced math receives clean TeX and separate source-line metadata. Its first content row receives the formula's source marker; pretty borders and subsequent decorative rows remain unnumbered. `TerminalRenderer::render` creates a fresh warning set, while `render_for_pager` shares one set across all three layouts and repeated gutter calculations. Nested code, HTML-table, and footnote renderers share the same set. No global warning cache survives a document refresh.

TeX operators remain distinct from ordinary text in the tree. This lets display layout center limits under `operatorname*`, `sum`, `prod`, `lim`, `min`, and `max`, while an ordinary subscript stays compact or beside its base. Operator boundaries also supply readable spacing before fractions and neighboring operands. Markdown table cells keep inline formulas compact, but `\displaystyle` and display events use the same structured layout as other display math.

## Invariants

- Raw code remains unchanged until the syntax/plaintext/math path is selected.
- ANSI spans do not contribute to visible width.
- Every rendered code row resets syntax styles before padding and container borders, including wrapped continuations.
- Labels and icons do not affect syntax lookup.
- References found in a plaintext Markdown code block must not escape their callout or table boundary.
- A pretty border is at least as wide as every visible content line after wrapping.
