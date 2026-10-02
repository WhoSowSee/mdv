use regex::Regex;
use std::sync::LazyLock;

use super::{ANSI_REGEX, SearchRange};

const RESET_STYLE: &str = "\x1b[0m";
static WHOLE_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(".+").unwrap());
mod colors;
use colors::{LINE_NAVIGATION_BACKGROUND, Rgb, SgrState, highlight_background, track_sgr};

fn highlight_matches(
    line: &str,
    query: &Regex,
    current_range: Option<SearchRange>,
    content_start_chars: usize,
    fixed_background: Option<Rgb>,
    depth: crate::ColorDepth,
    styles: crate::HighlightStyles,
) -> String {
    let stripped = ANSI_REGEX.replace_all(line, "");
    let content_start = stripped
        .char_indices()
        .nth(content_start_chars)
        .map_or(stripped.len(), |(index, _)| index);
    let matches = query
        .find_iter(&stripped[content_start..])
        .map(|matched| {
            let byte_start = content_start + matched.start();
            let byte_end = content_start + matched.end();
            let start = stripped[content_start..byte_start].chars().count();
            let end = start + stripped[byte_start..byte_end].chars().count();
            (byte_start, byte_end, SearchRange { start, end })
        })
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return line.to_string();
    }

    let mut removed_bytes = 0;
    let escapes = ANSI_REGEX
        .find_iter(line)
        .map(|escape| {
            let visible_offset = escape.start() - removed_bytes;
            removed_bytes += escape.len();
            (visible_offset, escape.as_str())
        })
        .collect::<Vec<_>>();
    let mut positions = matches
        .iter()
        .flat_map(|(start, end, _)| [*start, *end])
        .chain(escapes.iter().map(|(offset, _)| *offset))
        .collect::<Vec<_>>();
    positions.sort_unstable();
    positions.dedup();

    let mut output = String::with_capacity(line.len() + matches.len() * 32);
    let mut sgr_history = String::new();
    let mut sgr_state = SgrState::default();
    let mut cursor = 0;
    let mut escape_index = 0;
    let mut match_index = 0;
    let mut active_match: Option<usize> = None;

    for position in positions {
        output.push_str(&stripped[cursor..position]);
        cursor = position;

        if active_match.is_some_and(|index| matches[index].1 == position) {
            output.push_str(RESET_STYLE);
            output.push_str(&sgr_history);
            active_match = None;
            match_index += 1;
        }

        let first_escape = escape_index;
        while escapes
            .get(escape_index)
            .is_some_and(|(offset, _)| *offset == position)
        {
            let escape = escapes[escape_index].1;
            output.push_str(escape);
            track_sgr(&mut sgr_history, &mut sgr_state, escape);
            escape_index += 1;
        }
        if let Some(index) = active_match
            && escape_index > first_escape
        {
            output.push_str(&highlight_background(
                sgr_state,
                current_range == Some(matches[index].2),
                fixed_background,
                depth,
                styles.search_colors(current_range == Some(matches[index].2)),
            ));
        }

        while active_match.is_none()
            && matches
                .get(match_index)
                .is_some_and(|(start, _, _)| *start == position)
        {
            output.push_str(&highlight_background(
                sgr_state,
                current_range == Some(matches[match_index].2),
                fixed_background,
                depth,
                styles.search_colors(current_range == Some(matches[match_index].2)),
            ));
            if matches[match_index].0 == matches[match_index].1 {
                output.push_str(RESET_STYLE);
                output.push_str(&sgr_history);
                match_index += 1;
            } else {
                active_match = Some(match_index);
            }
        }
    }
    output.push_str(&stripped[cursor..]);
    output
}

pub fn highlight_search_matches(
    line: &str,
    query: &Regex,
    current_range: Option<SearchRange>,
    content_start_chars: usize,
    depth: crate::ColorDepth,
    styles: crate::HighlightStyles,
) -> String {
    highlight_matches(
        line,
        query,
        current_range,
        content_start_chars,
        None,
        depth,
        styles,
    )
}

pub fn highlight_line_navigation_target(
    line: &str,
    content_start_chars: usize,
    depth: crate::ColorDepth,
) -> String {
    highlight_matches(
        line,
        &WHOLE_LINE,
        None,
        content_start_chars,
        Some(LINE_NAVIGATION_BACKGROUND),
        depth,
        crate::HighlightStyles::default(),
    )
}
