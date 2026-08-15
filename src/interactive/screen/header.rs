use super::*;

pub(super) fn browser_header(browser: &BrowserState, no_colors: bool) -> String {
    if browser.filter_state() == FilterState::Editing {
        return styled(
            &format!("{} local", browser.documents().len()),
            Some(rgb(98, 98, 98)),
            None,
            false,
            no_colors,
        );
    }

    let documents = format!("{} documents", browser.documents().len());
    if browser.filter_state() != FilterState::Applied {
        return styled(&documents, Some(rgb(98, 98, 98)), None, false, no_colors);
    }
    let filtered = format!("{} “{}”", browser.filtered_count(), browser.query());
    let documents_color = if browser.section() == BrowserSection::Documents {
        rgb(151, 151, 151)
    } else {
        rgb(98, 98, 98)
    };
    let filtered_color = if browser.section() == BrowserSection::Filter {
        rgb(151, 151, 151)
    } else {
        rgb(98, 98, 98)
    };
    format!(
        "{} {} {}",
        styled(&documents, Some(documents_color), None, false, no_colors),
        styled("│", Some(rgb(60, 60, 60)), None, false, no_colors),
        styled(&filtered, Some(filtered_color), None, false, no_colors)
    )
}

pub(super) fn pagination(browser: &BrowserState, width: usize, no_colors: bool) -> String {
    let pages = browser.page_count();
    let dots_width = pages;
    if dots_width + 6 > width {
        return styled(
            &format!("{}/{}", browser.page() + 1, pages),
            Some(rgb(92, 92, 92)),
            None,
            false,
            no_colors,
        );
    }
    (0..pages)
        .map(|page| {
            let color = if page == browser.page() {
                rgb(132, 122, 133)
            } else {
                rgb(60, 60, 60)
            };
            styled("•", Some(color), None, false, no_colors)
        })
        .collect::<Vec<_>>()
        .join("")
}

pub(super) fn browser_filter_cursor_x(filter_text: &str, width: u16) -> u16 {
    display_width(filter_text)
        .saturating_add(3)
        .min(width.saturating_sub(1) as usize) as u16
}

pub(super) fn browser_item_selected(browser: &BrowserState, row: usize) -> bool {
    browser.filter_state() != FilterState::Editing && row == browser.selected_index_on_page()
}

pub(super) fn browser_filter_prompt_text(text: &str, no_colors: bool) -> String {
    let (label, query) = text
        .split_once(' ')
        .map_or((text, None), |(label, query)| (label, Some(query)));
    let mut prompt = format!(
        "   {}",
        styled(label, Some(BROWSER_ACCENT), None, false, no_colors)
    );
    if let Some(query) = query {
        prompt.push(' ');
        prompt.push_str(&styled(
            query,
            Some(BROWSER_FILTER_INPUT),
            None,
            false,
            no_colors,
        ));
    }
    prompt
}

pub(super) fn filtered_title(
    document: &crate::interactive::discovery::DocumentEntry,
    title: &str,
    query: &str,
    color: Color,
    no_colors: bool,
) -> String {
    if query.is_empty() || no_colors {
        return styled(title, Some(color), None, false, no_colors);
    }

    let indices = document.match_indices(query);
    let mut output = String::new();
    for (index, character) in title.chars().enumerate() {
        let mut style = AnsiStyle::new().fg(color);
        if indices.binary_search(&index).is_ok() {
            style = style.underline();
        }
        output.push_str(&style.apply(&character.to_string(), false));
    }
    output
}

pub(super) fn browser_logo_line(no_colors: bool) -> String {
    let logo = styled(
        " MDV ",
        Some(BROWSER_LOGO_FOREGROUND),
        Some(BROWSER_ACCENT),
        true,
        no_colors,
    );
    format!("   {logo}")
}
