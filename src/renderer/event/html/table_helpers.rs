use super::*;

pub(super) fn parse_html_alignment(value: &str) -> Option<HtmlAlignment> {
    match value.trim().to_ascii_lowercase().as_str() {
        "center" | "middle" => Some(HtmlAlignment::Center),
        "right" => Some(HtmlAlignment::Right),
        "left" => Some(HtmlAlignment::Left),
        _ => None,
    }
}

pub(super) fn table_alignment_from_html(alignment: HtmlAlignment) -> Alignment {
    match alignment {
        HtmlAlignment::Left => Alignment::Left,
        HtmlAlignment::Center => Alignment::Center,
        HtmlAlignment::Right => Alignment::Right,
    }
}

pub(super) fn html_text_align_from_style(style: &str) -> Option<HtmlAlignment> {
    for declaration in style.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        if property.trim().eq_ignore_ascii_case("text-align")
            && let Some(alignment) = parse_html_alignment(value)
        {
            return Some(alignment);
        }
    }

    None
}

pub(super) fn normalize_html_table(table: &mut TableState) {
    if table.headers.is_empty() && !table.rows.is_empty() {
        table.headers = table.rows.remove(0);
    }

    let column_count = table
        .headers
        .len()
        .max(table.rows.iter().map(Vec::len).max().unwrap_or(0));
    if column_count == 0 {
        return;
    }

    if table.headers.len() < column_count {
        table.headers.extend(std::iter::repeat_n(
            String::new(),
            column_count - table.headers.len(),
        ));
    }
    if table.alignments.len() < column_count {
        table.alignments.extend(std::iter::repeat_n(
            Alignment::None,
            column_count - table.alignments.len(),
        ));
    }
    for row in &mut table.rows {
        if row.len() < column_count {
            row.extend(std::iter::repeat_n(String::new(), column_count - row.len()));
        }
    }
}
