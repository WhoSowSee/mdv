use super::core::CalloutState;
use super::{
    CapturedReferenceBlock, CowStr, DeferredLinkReferenceBlock, EventRenderer, LinkStyle,
    LinkTruncationStyle, Result, TableInlineUrlSegment, TableInlineUrlTarget, TableState,
    ThemeElement, create_style, wrap_text_with_mode,
};
use crate::block_spacing::BlockElement;

const TABLE_REFERENCE_WRAP_DELIMITER: char = '\u{200B}';

fn style_underlined_table_link(link_text: &str, no_colors: bool) -> String {
    if no_colors {
        link_text.to_string()
    } else {
        format!("\x1b[4m{}\x1b[24m", link_text)
    }
}

fn build_clickable_underlined_table_link_replacement(
    link_text: &str,
    url: &str,
    no_colors: bool,
) -> Option<String> {
    if no_colors || link_text.is_empty() {
        None
    } else {
        Some(format!(
            "\x1b]8;;{}\x1b\\\x1b[4m{}\x1b[24m\x1b]8;;\x1b\\",
            url, link_text
        ))
    }
}

fn push_clickable_table_link(
    table: &mut TableState,
    link_text: &str,
    url: Option<&str>,
    no_colors: bool,
) {
    if link_text.is_empty() {
        return;
    }

    if let Some(styled) = url.and_then(|url| {
        build_clickable_underlined_table_link_replacement(link_text, url, no_colors)
    }) {
        table
            .clickable_link_replacements
            .push((link_text.to_string(), styled));
        table.current_cell.push_str(link_text);
    } else {
        push_underlined_table_link(table, link_text, no_colors);
    }
}

fn push_underlined_table_link(table: &mut TableState, link_text: &str, no_colors: bool) {
    table
        .current_cell
        .push_str(&style_underlined_table_link(link_text, no_colors));
}

fn push_wrappable_table_reference(cell: &mut String, rendered_reference: &str) {
    if rendered_reference.is_empty() {
        return;
    }

    // Insert a zero-width delimiter so comfy-table can wrap before `[N]` when needed
    // without changing visible content width.
    let needs_separator = cell
        .chars()
        .last()
        .map(|ch| !ch.is_whitespace() && ch != TABLE_REFERENCE_WRAP_DELIMITER)
        .unwrap_or(false);

    if needs_separator {
        cell.push(TABLE_REFERENCE_WRAP_DELIMITER);
    }

    cell.push_str(rendered_reference);
}

mod clickable;
mod end;
mod inline;
mod references;
mod start;
mod wrapping;
