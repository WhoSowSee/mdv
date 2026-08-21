pub(super) fn normalize_preformatted_html_text(text: &str) -> String {
    let mut normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.starts_with('\n') {
        normalized.remove(0);
    }
    if normalized.ends_with('\n') {
        normalized.pop();
    }
    normalized
}

pub(super) fn is_void_html_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
