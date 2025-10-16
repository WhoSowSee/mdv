# Markdown Feature Showcase

<!-- This file demonstrates a wide variety of Markdown capabilities. -->

## Table of Contents

- [Markdown Feature Showcase](#markdown-feature-showcase)
  - [Table of Contents](#table-of-contents)
  - [Basic Formatting](#basic-formatting)
  - [Headings](#headings)
- [Level 1 Heading](#level-1-heading)
  - [Level 2 Heading](#level-2-heading)
    - [Level 3 Heading](#level-3-heading)
      - [Level 4 Heading](#level-4-heading)
        - [Level 5 Heading](#level-5-heading)
          - [Level 6 Heading](#level-6-heading)
  - [Paragraph Variations](#paragraph-variations)
  - [Lists](#lists)
  - [Task Lists](#task-lists)
  - [Blockquotes](#blockquotes)
  - [Code Blocks](#code-blocks)
  - [Tables](#tables)
  - [Links and Images](#links-and-images)
  - [Footnotes and References](#footnotes-and-references)
  - [Inline HTML and Details](#inline-html-and-details)
  - [Definition List](#definition-list)
  - [Mathematics and Escapes](#mathematics-and-escapes)
  - [Horizontal Rules](#horizontal-rules)
  - [Miscellaneous Syntax](#miscellaneous-syntax)

## Basic Formatting

Regular text can be combined with **bold**, *italic*, and ***bold italic*** emphasis.
GitHub Flavored Markdown also supports ~~strikethrough~~ for marking removed content.
Inline code such as `let sample = true;` is handy for short snippets.

Block-level emphasis:

> **Note:** Quotes can contain `inline code`, *emphasis*, and other Markdown.

## Headings

Markdown offers six levels of headings:

# Level 1 Heading
## Level 2 Heading
### Level 3 Heading
#### Level 4 Heading
##### Level 5 Heading
###### Level 6 Heading

Return to regular text after heading demonstrations.

## Paragraph Variations

Line breaks can be forced with two spaces at the end of the line.
For example, this sentence ends with two spaces.
This line appears immediately below without an empty line between them.

Paragraphs can include emphasis, links like [Example Domain](https://example.com), and emojis such as :sparkles: when rendered on platforms that support them.

## Lists

Unordered list with nesting:

- Item one
  - Nested item A
    - Deeply nested item alpha
  - Nested item B
- Item two
- Item three

Ordered list with custom numbering:

1. First ordered item
2. Second ordered item
3. Third ordered item
   1. Nested ordered sub-item
   2. Another ordered sub-item

Mixed list:

1. Start with a numbered item
   - Switch to an unordered entry
   - Another unordered entry
2. Resume ordered items

## Task Lists

GitHub Flavored Markdown supports checkboxes:

- [x] Completed task
- [ ] Pending task
- [ ] Task with **bold** text
  - [x] Nested completed task
  - [ ] Nested pending task

## Blockquotes

Simple blockquote:

> Markdown enables quoting other sources.

Multi-paragraph blockquote:

> First paragraph in the quote.
>
> Second paragraph continues the quoted text.

Nested blockquote:

> Top-level quote
>> Nested quote
>>> Third level quote

## Code Blocks

Indented code block:

    {
        "name": "Indented JSON",
        "purpose": "Demonstrate four-space indentation"
    }

Fenced code block without language:

```
Plain fenced block with backticks.
Use this when syntax highlighting is not required.
```

Fenced block with language identifier:

```rust
fn main() {
    println!("Hello from Rust!");
}
```

```python
def greet(name: str) -> str:
    """Return a friendly greeting."""
    return f"Hello, {name}!"

if __name__ == "__main__":
    print(greet("Markdown Reader"))
```

Alternative fence characters:

~~~sql
SELECT
    id,
    username,
    created_at
FROM users
WHERE active = TRUE
ORDER BY created_at DESC;
~~~

Diff code block:

```diff
- println!("Debug output");
+ println!("Release output");
```

HTML comment inside Markdown (will not render):

<!-- Debug code block removed -->

## Tables

Basic table:

| Syntax | Description |
|--------|-------------|
| Header | Title text  |
| Paragraph | Text under the header |

Table with alignment:

| Left Aligned | Center Aligned | Right Aligned |
|:-------------|:--------------:|--------------:|
| Item A       |    10 units    |         9.99 |
| Item B       |     5 units    |        19.95 |
| Item C       |    42 units    |         0.42 |

Complex cell content:

| Feature | Supports |
|---------|----------|
| Lists   | - Bullet one<br>- Bullet two |
| Code    | `inline()` and<br>```block``` |

## Links and Images

Inline link: [OpenAI](https://openai.com)

Reference-style link: [Rust Lang][rust-home]

Auto-detected URL: <https://www.rust-lang.org>

Relative link example: [Renderer module](src/renderer/terminal.rs)

Image link:

![Colorful nebula](https://images.unsplash.com/photo-1582719478250-c89cae4dc85b?auto=format&fit=crop&w=800&q=80 "Unsplash sample")

Image with reference:

![Rust logo][rust-logo]

## Footnotes and References

This sentence includes a footnote reference.[^1] You can add multiple footnotes within the same paragraph[^2] if needed.

Quoted text with a citation[^citation]:

> “Markdown allows you to write using an easy-to-read, easy-to-write plain text format.” — *Anonymous*

[^1]: Footnotes appear at the end of the document or section.
[^2]: Multiple footnotes showcase repeated usage.
[^citation]: Citation footnote for the blockquote example.

[rust-home]: https://www.rust-lang.org/
[rust-logo]: https://static.rust-lang.org/logos/rust-logo-512x512.png

## Inline HTML and Details

Markdown can include HTML blocks for richer layouts:

<details>
  <summary>Expandable Section</summary>
  <p>This content is hidden until the user expands the details element.</p>
  <ul>
    <li>Supports nested HTML</li>
    <li>Works well for FAQs</li>
  </ul>
</details>

Block-level HTML:

<div style="border: 1px solid #ccc; padding: 8px;">
  <p>Custom HTML containers can highlight important information.</p>
</div>

Inline `<abbr title="Application Programming Interface">API</abbr>` tags provide abbreviations.

## Definition List

Term One
: First definition paragraph that explains the concept in more detail.
: Second definition line for additional context.

Term Two
: Separate definition entry with its own description.

## Mathematics and Escapes

Inline math style using delimiters: $E = mc^2$.

Fenced math block:

```math
\int_a^b f(x)\,dx = F(b) - F(a)
```

Escaping special characters: \*escaped asterisk\*, \_escaped underscore\_, and \\ backslash.

## Horizontal Rules

Three different horizontal separators:

---

***

___

## Miscellaneous Syntax

Combined emphasis: **_bold italic text_** and __*alternate bold italic*__ demonstrate redundancy.

Superscripts and subscripts (GitHub extension): X^2^ and H~2~O.

Emoji shortcodes: :rocket:, :tada:, and :memo:.

Keyboard keys: <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd>.

Tables combined with footnotes:

| Shortcut | Description |
|----------|-------------|
| `Ctrl+C` | Copy command[^copy] |
| `Ctrl+V` | Paste command[^paste] |

[^copy]: Common keyboard shortcut for copying content.
[^paste]: Common keyboard shortcut for pasting content.

Callouts using blockquotes:

> **Warning:** Ensure the renderer supports extended syntax before relying on platform-specific features.

Metadata style block:

```yaml
title: Sample Markdown Metadata
author: Example Author
tags:
  - markdown
  - reference
```

Escaped HTML entities: &copy; &reg; &hearts;

Final paragraph reminding readers that this file intentionally covers numerous Markdown cases to help test rendering, wrapping, theming, and hyperlink strategies.
