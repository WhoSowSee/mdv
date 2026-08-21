#![allow(clippy::trivial_regex)]

use super::{PagerState, Selection};

#[test]
fn backgrounds_follow_foreground_colors() {
    const BLUE_FOREGROUND: &str = "\x1b[38;2;130;170;255m";
    const ORANGE_FOREGROUND: &str = "\x1b[38;2;255;160;100m";
    const INDEXED_FOREGROUND: &str = "\x1b[38;5;75m";
    const BLUE_OTHER_BACKGROUND: &str = "\x1b[48;2;43;53;75m";
    const ORANGE_CURRENT_BACKGROUND: &str = "\x1b[48;2;148;97;66m";
    const INDEXED_OTHER_BACKGROUND: &str = "\x1b[48;2;35;54;75m";

    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = format!(
        "{BLUE_FOREGROUND}match one\x1b[0m\n{ORANGE_FOREGROUND}match two\x1b[0m\n{INDEXED_FOREGROUND}match three\x1b[0m\n"
    );
    pager.search_state.search_term = Some(regex::Regex::new("match").unwrap());
    pager.reformat_display().unwrap();
    pager.search_state.search_mark = 1;

    let rows = pager.render_rows_for_display(0, 3);

    for (row, foreground, background) in [
        (&rows[0], BLUE_FOREGROUND, BLUE_OTHER_BACKGROUND),
        (&rows[1], ORANGE_FOREGROUND, ORANGE_CURRENT_BACKGROUND),
        (&rows[2], INDEXED_FOREGROUND, INDEXED_OTHER_BACKGROUND),
    ] {
        assert!(row.contains(foreground), "{}", row.escape_debug());
        assert!(row.contains(background), "{}", row.escape_debug());
    }
}

#[test]
fn only_current_occurrence_uses_current_background() {
    const FOREGROUND: &str = "\x1b[38;2;130;170;255m";
    const CURRENT_BACKGROUND: &str = "\x1b[48;2;80;103;151m";
    const OTHER_BACKGROUND: &str = "\x1b[48;2;43;53;75m";

    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = format!("{FOREGROUND}match and match\x1b[0m\n");
    pager.search_state.search_term = Some(regex::Regex::new("match").unwrap());
    pager.reformat_display().unwrap();

    assert_eq!(pager.search_state.search_matches.len(), 2);

    let first = pager.render_rows_for_display(0, 1).remove(0);
    assert_eq!(first.matches(CURRENT_BACKGROUND).count(), 1);
    assert_eq!(first.matches(OTHER_BACKGROUND).count(), 1);
    assert!(first.find(CURRENT_BACKGROUND) < first.find(OTHER_BACKGROUND));

    pager.search_state.search_mark = 1;
    let second = pager.render_rows_for_display(0, 1).remove(0);
    assert_eq!(second.matches(CURRENT_BACKGROUND).count(), 1);
    assert_eq!(second.matches(OTHER_BACKGROUND).count(), 1);
    assert!(second.find(OTHER_BACKGROUND) < second.find(CURRENT_BACKGROUND));
}

#[test]
fn selected_match_uses_lighter_background() {
    const BLUE_FOREGROUND: &str = "\x1b[38;2;130;170;255m";
    const SELECTION_BACKGROUND: &str = "\x1b[48;2;46;49;59m";
    const SELECTED_MATCH_BACKGROUND: &str = "\x1b[48;2;92;116;167m";

    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = format!("{BLUE_FOREGROUND}match tail\x1b[0m\n");
    pager.search_state.search_term = Some(regex::Regex::new("match").unwrap());
    pager.reformat_display().unwrap();
    pager.selection_anchor = Some(Selection {
        absolute_row: 0,
        col: 0,
    });
    pager.selection = Some(Selection {
        absolute_row: 0,
        col: 9,
    });

    let row = pager.render_rows_for_display(0, 1).remove(0);

    assert!(row.contains(BLUE_FOREGROUND), "{}", row.escape_debug());
    assert!(row.contains(SELECTION_BACKGROUND), "{}", row.escape_debug());
    assert!(
        row.contains(SELECTED_MATCH_BACKGROUND),
        "{}",
        row.escape_debug()
    );
}
