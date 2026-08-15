use super::*;

pub(super) fn html_media_source(element: &ElementRef<'_>) -> Option<String> {
    for attr in ["src", "data", "href"] {
        if let Some(value) = element.attr(attr).map(str::trim)
            && !value.is_empty()
        {
            return Some(value.to_string());
        }
    }

    element
        .attr("srcset")
        .and_then(first_srcset_candidate)
        .map(str::to_string)
}

pub(super) fn first_srcset_candidate(srcset: &str) -> Option<&str> {
    srcset
        .split(',')
        .filter_map(|candidate| candidate.split_whitespace().next())
        .find(|candidate| !candidate.is_empty())
}

pub(super) fn html_media_label(element: &ElementRef<'_>, source: &str) -> String {
    for attr in ["alt", "title", "aria-label"] {
        if let Some(value) = element.attr(attr).map(str::trim)
            && !value.is_empty()
        {
            return value.to_string();
        }
    }

    media_filename(source).unwrap_or(source).to_string()
}

pub(super) fn media_filename(source: &str) -> Option<&str> {
    let path = source.split(['?', '#']).next().unwrap_or(source);
    let filename = path.rsplit(['/', '\\']).next().unwrap_or(path).trim();
    if filename.is_empty() {
        None
    } else {
        Some(filename)
    }
}
