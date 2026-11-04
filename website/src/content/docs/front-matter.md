---
title: "Front matter"
description: "YAML document properties and all eight presentation modes."
slug: "front-matter"
group: "Formatting"
order: 26
---

## Recognized input

Front matter must begin on the first line with an exact `---`, end with an exact `---`, and contain a YAML property mapping. `source` disables extraction. A later horizontal rule does not start front matter.

```markdown
---
title: Example
author: Alice
tags: [guide, terminal]
---

# Document

The body starts here.
```

## Display modes

| `--front-matter` | Result |
| --- | --- |
| `hidden` | Omit front matter; the default. |
| `panel` | A properties panel using the callout renderer. |
| `table` | A two-column Property/Value table. |
| `plain` | One normalized key/value pair per line. |
| `inline` | One flowing paragraph of properties. |
| `blocks` | Each value below its key, as a definition list. |
| `code` | A syntax-highlighted YAML block. |
| `source` | Parse the entire source as ordinary Markdown. |

<div data-mdv-preview="front-matter"></div>

## Configuration and styling

Save the mode in `front_matter`. Theme fields `front_matter.title`, `key`, `value` and `border` control the properties panel. `--from` selects a range of the Markdown body after front matter extraction; source line numbers retain their original positions.
