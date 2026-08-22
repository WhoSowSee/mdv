#![allow(clippy::trivial_regex)]

use super::{IncrementalSearchOpts, incremental_preview};
use crate::PagerState;

#[test]
fn preview_highlights_matches_before_confirmation() {
    const FOREGROUND: &str = "\x1b[38;2;130;170;255m";
    const CURRENT_BACKGROUND: &str = "\x1b[48;2;80;103;151m";
    const OTHER_BACKGROUND: &str = "\x1b[48;2;43;53;75m";

    let mut state = PagerState::new().unwrap();
    state.cols = 80;
    state.rows = 5;
    state.screen.orig_text = format!(
        "before\n{FOREGROUND}match here match\x1b[0m\n{FOREGROUND}match again\x1b[0m\nafter\n"
    );
    state.reformat_display().unwrap();
    let options = IncrementalSearchOpts::from(&state);
    let query = regex::Regex::new("match").unwrap();

    let preview = incremental_preview(&options, &query).unwrap();
    let matching_rows = preview
        .rows
        .iter()
        .filter(|row| row.contains("match"))
        .collect::<Vec<_>>();

    assert_eq!(matching_rows.len(), 2);
    assert!(matching_rows.iter().all(|row| row.contains(FOREGROUND)));
    assert_eq!(
        matching_rows
            .iter()
            .map(|row| row.matches(CURRENT_BACKGROUND).count())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        matching_rows
            .iter()
            .map(|row| row.matches(OTHER_BACKGROUND).count())
            .sum::<usize>(),
        2
    );
}

#[test]
fn preview_reports_backfilled_viewport_position() {
    let mut state = PagerState::new().unwrap();
    state.cols = 80;
    state.rows = 4;
    state.screen.orig_text = "zero\none\ntwo\nthree\nfour\nfive\ntarget\n".to_string();
    state.reformat_display().unwrap();
    let options = IncrementalSearchOpts::from(&state);
    let query = regex::Regex::new("target").unwrap();

    let preview = incremental_preview(&options, &query).unwrap();

    assert_eq!(
        preview.upper_mark,
        state
            .screen
            .formatted_lines_count()
            .saturating_sub(state.content_rows())
    );
    assert!(preview.rows.last().unwrap().contains("target"));
}
