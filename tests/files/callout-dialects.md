!!! warning "MkDocs title"

    MkDocs first paragraph.

    MkDocs second paragraph.

    ???+ tip "Nested MkDocs"
        Nested MkDocs body.

After MkDocs.

::::warning[Docusaurus **title**]{#warning-id .custom-class}
Docusaurus body.

:::tip[Nested Docusaurus]
Nested Docusaurus body.
:::
::::

After Docusaurus.

::: warning VitePress title
VitePress body.
:::

::: details VitePress details
Details body.
:::

:::{admonition} MyST title
:class: tip dropdown
:name: myst-anchor

MyST body.
:::

```{warning}
:name: fenced-myst

MyST backtick body.
```

::: {#tip-quarto .callout-tip collapse="true" icon=false}
## Quarto title

Quarto body.
:::

/// admonition | PyMdown title
    type: danger
    attrs: {id: pymdown-anchor, class: custom-class}

PyMdown body.
///

/// details | PyMdown details
    open: True

PyMdown details body.
///

{% hint style="success" %}
GitBook body.

{% hint style="warning" %}
Nested GitBook body.
{% endhint %}
{% endhint %}

After GitBook.
