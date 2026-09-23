# Callout syntax

All supported callout dialects use mdv's `--callout-style` and `--custom-callout` settings. Their bodies support ordinary Markdown, including paragraphs, lists, tables, links, code, and nested callouts. Callouts may also appear inside Markdown blockquotes and list items.

## Material for MkDocs

```markdown
!!! warning "Before updating"

    Save your changes.

    This is another paragraph in the same block.

    ???+ tip "Optional steps"
        This nested block is initially expanded.

This paragraph is outside the callout.
```

Bodies use four columns of indentation relative to the opening marker. `???` marks a collapsed block; `???+` marks an expanded block. `!!! note ""` hides the title and type icon. Empty titles on collapsible blocks retain the default title. `inline` and `inline end` are consumed as layout metadata; terminal blocks remain in document order. Quoted titles can contain Markdown. The earlier mdv form `!!! note Title` followed immediately by an unindented paragraph remains supported until a blank line, another callout opener, or the containing callout's closing delimiter.

## Docusaurus

```markdown
::::warning[Before **updating**]{#update-warning .custom-class}
Save your changes.

:::tip[Read [the guide](https://example.com)]
Nested content.
:::
::::
```

Square-bracket titles accept nested Markdown brackets and escaped delimiters. Attribute blocks are consumed as metadata. Use longer fences around nested containers.

## VitePress

```markdown
::: warning Before updating
Save your changes.
:::

::: details Optional steps {open}
This block is initially expanded.
:::
```

The title follows the type as plain Markdown. Trailing attributes are consumed; `{open}` sets the initial state of a `details` block. Without it, `details` is initially collapsed.

## MyST / Sphinx

````markdown
:::{admonition} Before updating
:class: warning dropdown
:name: update-warning

Save your changes.
:::

```{note}
---
class: dropdown toggle-shown
name: optional-steps
---
Optional content.
```
````

Colon and backtick/tilde directive fences are recognized. Options can use `:key: value` lines or a YAML block delimited by `---`. The `admonition` directive accepts a custom title; a known type in `class` supplies its style. `dropdown` marks a collapsed block, and `toggle-shown` makes it expanded. `name` and other nonvisual options are metadata. `versionadded`, `versionchanged`, and `deprecated` directives include their version argument in the generated title. Existing `:::{note} Title` input also remains accepted.

## Quarto

```markdown
::: {#wrn-update .callout-warning collapse="true" icon=false}
## Before updating

Save your changes.
:::

::: {.callout-tip title="Optional steps" appearance="minimal"}
Optional content.
:::
```

The `.callout-TYPE` class supplies the type. A heading at the beginning of the body becomes the title and is removed from the body; an explicit `title` attribute takes precedence. A line indented by four or more columns remains body content, even if it contains heading markers. `collapse=true/false` records the initial collapsed/expanded state. `icon=false` and `appearance="minimal"` suppress the type icon. The other appearance values use the configured terminal frame.

## PyMdown Blocks

```markdown
/// admonition | Before updating
    type: warning
    attrs: {id: update-warning, class: custom-class}

Save your changes.
///

/// details | Optional steps
    open: True

Optional content.
///
```

Type shortcuts such as `/// note | Title` are supported. Options are a YAML mapping indented by at least four spaces immediately after the opening line; a blank line ends the option section. `type` selects the type, `open` sets the state of `details`, and `attrs` is consumed as metadata. Titles may begin with a Markdown heading marker. Nested blocks can use longer outer fences.

## GitBook

```markdown
{% hint style="warning" %}
Save your changes.

{% hint style="info" %}
Additional context.
{% endhint %}
{% endhint %}
```

The supported styles are `info`, `success`, `warning`, and `danger`. Both quote styles work for the attribute value. Hint blocks support multiple paragraphs and nesting.

## Terminal and export behavior

- The full body of a foldable block is always visible. Enable `--callout-style 'pretty:show-icons;fold-icons'` to display its declared initial state; this requires a Nerd Font.
- Empty MkDocs titles suppress the header in both `simple` and `pretty` layouts. Icon suppression is per block and does not suppress a separately enabled fold indicator.
- CSS classes, IDs, and site-specific options are consumed as metadata. mdv does not execute CSS, JavaScript, MDX components, site configuration, or Quarto cross-reference numbering.
- Fenced callouts require a matching closing delimiter. Malformed headers remain literal Markdown. Ordinary fenced code, indented code, multiline inline code, and HTML blocks are protected during syntax normalization. mdv's existing Markdown rendering for `text`/`markdown` code blocks is unchanged.
- Source numbering follows retained body lines even when option lines or delimiters are removed. Generated separators have no source number.
- `--html` keeps mdv's existing canonical blockquote export; it does not reproduce the source site's callout theme or interactivity. Internal presentation metadata is excluded from HTML output.
