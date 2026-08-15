pub(super) fn buffering_html_container_tag(html: &str) -> Option<&'static str> {
    BUFFERED_HTML_CONTAINER_TAGS
        .iter()
        .copied()
        .find(|tag| contains_html_tag(html, tag, false))
}

pub(super) fn buffering_inline_html_container_tag(html: &str) -> Option<&'static str> {
    BUFFERED_INLINE_HTML_CONTAINER_TAGS
        .iter()
        .copied()
        .find(|tag| contains_html_tag(html, tag, false))
}

pub(super) fn contains_html_tag(html: &str, tag: &str, closing: bool) -> bool {
    let lower = html.to_ascii_lowercase();
    let needle = if closing {
        format!("</{tag}")
    } else {
        format!("<{tag}")
    };
    let mut offset = 0;

    while let Some(index) = lower[offset..].find(&needle) {
        let after = offset + index + needle.len();
        let has_tag_boundary = lower[after..]
            .chars()
            .next()
            .map(|ch| ch == '>' || ch == '/' || ch.is_ascii_whitespace())
            .unwrap_or(false);

        if has_tag_boundary {
            return true;
        }

        offset = after;
    }

    false
}

pub(super) fn is_html_block_element(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "dd"
            | "details"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "header"
            | "main"
            | "nav"
            | "p"
            | "section"
            | "summary"
            | "center"
    )
}

pub(super) fn is_definition_description_inline_block(name: &str) -> bool {
    matches!(name, "p" | "div" | "section" | "article" | "span")
}

pub(super) const BUFFERED_HTML_CONTAINER_TAGS: &[&str] = &[
    "table",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "p",
    "div",
    "address",
    "center",
    "section",
    "figure",
    "figcaption",
    "header",
    "footer",
    "main",
    "article",
    "aside",
    "nav",
    "dialog",
    "fieldset",
    "form",
    "details",
    "summary",
    "blockquote",
    "dl",
    "ol",
    "pre",
    "textarea",
    "ul",
];

pub(super) const BUFFERED_INLINE_HTML_CONTAINER_TAGS: &[&str] = &[
    "a", "abbr", "b", "button", "cite", "code", "del", "em", "i", "kbd", "mark", "s", "samp",
    "select", "small", "span", "strike", "strong", "sub", "sup",
];
