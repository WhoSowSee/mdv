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
    contains_html_tag_outside_comments(html, tag, closing, &mut false)
}

pub(super) fn contains_html_tag_outside_comments(
    html: &str,
    tag: &str,
    closing: bool,
    comment_open: &mut bool,
) -> bool {
    let lower = html.to_ascii_lowercase();
    let needle = if closing {
        format!("</{tag}")
    } else {
        format!("<{tag}")
    };
    let mut offset = 0;
    let mut found = false;

    while offset < lower.len() {
        if *comment_open {
            let Some(end) = lower[offset..].find("-->") else {
                break;
            };
            offset += end + 3;
            *comment_open = false;
            continue;
        }

        let Some(index) = lower[offset..].find('<') else {
            break;
        };
        offset += index;
        if lower[offset..].starts_with("<!--") {
            *comment_open = true;
            offset += 4;
            continue;
        }
        if lower[offset..].starts_with(&needle)
            && lower[offset + needle.len()..]
                .chars()
                .next()
                .is_some_and(|ch| ch == '>' || ch == '/' || ch.is_ascii_whitespace())
        {
            found = true;
        }
        offset += 1;
    }

    found
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
