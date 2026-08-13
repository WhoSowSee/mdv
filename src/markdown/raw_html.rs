use pulldown_cmark::Event;
use std::ops::Range;

const RAW_TEXT_CONTAINER_TAGS: &[&str] = &[
    "code", "noscript", "pre", "script", "style", "template", "textarea", "title",
];

pub(super) fn coalesce_raw_text_container(
    content: &str,
    events: &[(Event<'_>, Range<usize>)],
    start_idx: usize,
) -> Option<(Event<'static>, Range<usize>, usize)> {
    let (start_event, start_range) = events.get(start_idx)?;
    let start_html = html_event_content(start_event)?;
    let tag = opening_raw_text_container(start_html)?;
    let mut depth = 0usize;

    for (idx, (event, range)) in events.iter().enumerate().skip(start_idx) {
        let Some(html) = html_event_content(event) else {
            continue;
        };

        depth = depth.saturating_add(count_html_tags(html, tag, false));
        depth = depth.saturating_sub(count_html_tags(html, tag, true));
        if depth != 0 || idx == start_idx {
            continue;
        }

        let combined_range = start_range.start..range.end;
        let raw = content.get(combined_range.clone())?.to_string();
        let combined = match start_event {
            Event::Html(_) => Event::Html(raw.into()),
            Event::InlineHtml(_) => Event::InlineHtml(raw.into()),
            _ => return None,
        };
        return Some((combined, combined_range, idx));
    }

    None
}

fn html_event_content<'event>(event: &'event Event<'_>) -> Option<&'event str> {
    match event {
        Event::Html(html) | Event::InlineHtml(html) => Some(html.as_ref()),
        _ => None,
    }
}

fn opening_raw_text_container(html: &str) -> Option<&'static str> {
    let trimmed = html.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    let tag = RAW_TEXT_CONTAINER_TAGS
        .iter()
        .copied()
        .find(|tag| starts_with_html_tag(&lower, tag, false))?;
    let opening_end = lower.find('>')?;
    if lower[..opening_end].trim_end().ends_with('/') || count_html_tags(&lower, tag, true) > 0 {
        return None;
    }
    Some(tag)
}

fn count_html_tags(html: &str, tag: &str, closing: bool) -> usize {
    let lower = html.to_ascii_lowercase();
    let needle = if closing {
        format!("</{tag}")
    } else {
        format!("<{tag}")
    };
    let mut count = 0usize;
    let mut offset = 0usize;

    while let Some(relative) = lower[offset..].find(&needle) {
        let start = offset + relative;
        let boundary = start + needle.len();
        let valid_boundary = lower[boundary..]
            .chars()
            .next()
            .is_some_and(|ch| ch == '>' || ch == '/' || ch.is_ascii_whitespace());
        if valid_boundary {
            count += 1;
        }
        offset = boundary;
    }

    count
}

fn starts_with_html_tag(html: &str, tag: &str, closing: bool) -> bool {
    let prefix = if closing {
        format!("</{tag}")
    } else {
        format!("<{tag}")
    };
    html.strip_prefix(&prefix)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|ch| ch == '>' || ch == '/' || ch.is_ascii_whitespace())
}
