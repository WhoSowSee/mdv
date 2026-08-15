use super::*;

pub(super) fn is_html_checkbox(element: &ElementRef<'_>) -> bool {
    element.value().name().eq_ignore_ascii_case("input")
        && element
            .attr("type")
            .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("checkbox"))
}

pub(super) fn html_list_item_starts_with_checkbox(element: &ElementRef<'_>) -> bool {
    for child in element.children() {
        if let HtmlNode::Text(text) = child.value() {
            if text.trim().is_empty() {
                continue;
            }
            return false;
        }
        if matches!(child.value(), HtmlNode::Comment(_)) {
            continue;
        }
        return ElementRef::wrap(child).is_some_and(|child| is_html_checkbox(&child));
    }

    false
}

pub(super) fn html_list_marker_state(
    element: &ElementRef<'_>,
    ordered: bool,
) -> HtmlListMarkerState {
    if ordered {
        let reversed = element.attr("reversed").is_some();
        let default_start = if reversed {
            html_direct_list_item_count(element).max(1) as i64
        } else {
            1
        };
        return HtmlListMarkerState::Ordered {
            current: parse_html_integer_attr(element, "start").unwrap_or(default_start),
            step: if reversed { -1 } else { 1 },
            kind: html_ordered_list_marker_kind(element)
                .unwrap_or(HtmlOrderedListMarkerKind::Decimal),
        };
    }

    HtmlListMarkerState::Unordered {
        marker: html_unordered_list_marker(element),
    }
}

pub(super) fn html_direct_list_item_count(element: &ElementRef<'_>) -> usize {
    element
        .child_elements()
        .filter(|child| child.value().name().eq_ignore_ascii_case("li"))
        .count()
}

pub(super) fn parse_html_integer_attr(element: &ElementRef<'_>, attr: &str) -> Option<i64> {
    element.attr(attr)?.trim().parse::<i64>().ok()
}

pub(super) fn html_ordered_list_marker_kind(
    element: &ElementRef<'_>,
) -> Option<HtmlOrderedListMarkerKind> {
    match element.attr("type")?.trim() {
        "1" => Some(HtmlOrderedListMarkerKind::Decimal),
        "a" => Some(HtmlOrderedListMarkerKind::LowerAlpha),
        "A" => Some(HtmlOrderedListMarkerKind::UpperAlpha),
        "i" => Some(HtmlOrderedListMarkerKind::LowerRoman),
        "I" => Some(HtmlOrderedListMarkerKind::UpperRoman),
        _ => None,
    }
}

pub(super) fn html_unordered_list_marker(element: &ElementRef<'_>) -> &'static str {
    match element.attr("type").map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case("disc") => "• ",
        Some(value) if value.eq_ignore_ascii_case("circle") => "◦ ",
        Some(value) if value.eq_ignore_ascii_case("square") => "▪ ",
        _ => "- ",
    }
}

pub(super) fn format_html_ordered_marker(value: i64, kind: HtmlOrderedListMarkerKind) -> String {
    match kind {
        HtmlOrderedListMarkerKind::Decimal => value.to_string(),
        HtmlOrderedListMarkerKind::LowerAlpha => format_alpha_marker(value, false),
        HtmlOrderedListMarkerKind::UpperAlpha => format_alpha_marker(value, true),
        HtmlOrderedListMarkerKind::LowerRoman => format_roman_marker(value, false),
        HtmlOrderedListMarkerKind::UpperRoman => format_roman_marker(value, true),
    }
}

pub(super) fn format_alpha_marker(value: i64, uppercase: bool) -> String {
    if value <= 0 {
        return value.to_string();
    }

    let mut remaining = value;
    let mut marker = Vec::new();
    while remaining > 0 {
        remaining -= 1;
        let base = if uppercase { b'A' } else { b'a' };
        marker.push((base + (remaining % 26) as u8) as char);
        remaining /= 26;
    }

    marker.iter().rev().collect()
}

pub(super) fn format_roman_marker(value: i64, uppercase: bool) -> String {
    if !(1..=3999).contains(&value) {
        return value.to_string();
    }

    let mut remaining = value;
    let mut marker = String::new();
    for (amount, symbol) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while remaining >= amount {
            marker.push_str(symbol);
            remaining -= amount;
        }
    }

    if uppercase {
        marker
    } else {
        marker.to_ascii_lowercase()
    }
}
