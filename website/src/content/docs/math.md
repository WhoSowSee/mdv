---
title: "Math"
description: "Inline and display formulas, math containers and supported TeX notation."
slug: "math"
group: "Formatting"
order: 22
---

## Inline and display formulas

Use `$...$` for inline formulas and `$$...$$` for display math. Fenced blocks with `math`, `latex`, `tex`, `katex` or `mathjax` enter the same math renderer.

```markdown
Inline: $x^2 + y^2 = z^2$.

$$
\frac{a+b}{c} = \sqrt{x}
$$
```

Inline formulas become a compact Unicode sequence. Display math can use multiple rows for fractions, matrices, roots and aligned expressions. Ordinary source newlines become spaces; TeX `\\` creates a mathematical row.

## Math block styles

`--math-block-style` accepts `basic` (indented without a border, the default), `simple` (a left border), or `pretty` (a frame). It is independent from `--code-block-style` and code-line numbering. Theme fields `math.text` and `math.border` control colors.

<div data-mdv-preview="math-block-style"></div>

## Supported notation and limits

The renderer supports fractions, roots, scripts, common symbols and operators, Unicode math alphabets, binomials, accents, `overset`/`underset`, braces, `align`, `gather`, matrices, `cases` and basic `array` column specifications. This is terminal Unicode output, not browser TeX typesetting.

Unsupported commands and environments retain their TeX source.

- A structural error preserves the entire original formula and produces a warning.
- Flat expressions can wrap; a structured formula keeps its geometry when wider than the available space and reports overflow.
- Markdown table cells keep inline formulas compact, while display math and `\displaystyle` use structured layout.
