use super::*;

pub(super) fn html_alignment(element: &ElementRef<'_>) -> Option<HtmlAlignment> {
    if element.value().name().eq_ignore_ascii_case("center") {
        return Some(HtmlAlignment::Center);
    }

    if let Some(alignment) = element.attr("align").and_then(parse_html_alignment) {
        return Some(alignment);
    }

    element.attr("style").and_then(html_text_align_from_style)
}

pub(super) fn html_inline_style_elements(element: &ElementRef<'_>) -> Vec<ThemeElement> {
    let Some(style) = element.attr("style") else {
        return Vec::new();
    };

    let mut elements = Vec::new();
    for declaration in style.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property.trim();
        let value = value.trim();

        if property.eq_ignore_ascii_case("font-weight") && is_bold_font_weight(value) {
            push_unique_theme_element(&mut elements, ThemeElement::Strong);
        } else if property.eq_ignore_ascii_case("font-style")
            && matches_ignore_ascii_case_any(value, &["italic", "oblique"])
        {
            push_unique_theme_element(&mut elements, ThemeElement::Emphasis);
        } else if matches_ignore_ascii_case_any(
            property,
            &["text-decoration", "text-decoration-line"],
        ) {
            for token in value.split_whitespace() {
                if token.eq_ignore_ascii_case("line-through") {
                    push_unique_theme_element(&mut elements, ThemeElement::Strikethrough);
                } else if token.eq_ignore_ascii_case("underline") {
                    push_unique_theme_element(&mut elements, ThemeElement::Underline);
                }
            }
        }
    }

    elements
}

pub(super) fn push_unique_theme_element(elements: &mut Vec<ThemeElement>, element: ThemeElement) {
    if !elements.contains(&element) {
        elements.push(element);
    }
}

pub(super) fn is_bold_font_weight(value: &str) -> bool {
    if matches_ignore_ascii_case_any(value, &["bold", "bolder"]) {
        return true;
    }

    value
        .parse::<u16>()
        .map(|weight| weight >= 600)
        .unwrap_or(false)
}

pub(super) fn matches_ignore_ascii_case_any(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}
