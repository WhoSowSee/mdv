use crate::{
    SearchMode,
    search::{InputStatus, SearchOpts, handle_key_press, write_search_input},
};
use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    style::Attribute,
    terminal::{Clear, ClearType},
};
use std::io::Write;

fn new_search_opts(sm: SearchMode) -> SearchOpts<'static> {
    let search_char = match sm {
        SearchMode::Forward => '/',
        SearchMode::Reverse => '?',
        SearchMode::Unknown => unreachable!(),
    };

    SearchOpts {
        ev: None,
        string: String::with_capacity(200),
        input_status: InputStatus::Active,
        cursor_position: 1,
        word_index: Vec::with_capacity(200),
        prompt: search_char.to_string(),
        search_char,
        rows: 25,
        cols: 100,
        incremental_search_options: None,
        compiled_regex: None,
        preview_upper_mark: None,
        query_selected: false,
        output_styling: true,
        search_mode: sm,
    }
}

const fn make_event_from_keycode(kc: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code: kc,
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::NONE,
        state: KeyEventState::NONE,
    })
}

fn pretest_setup_forward_search() -> (SearchOpts<'static>, Vec<u8>, u16, &'static str) {
    const QUERY_STRING: &str = "this is@complex-text_search?query"; // length = 33
    #[allow(clippy::cast_possible_truncation)]
    let last_movable_column: u16 = (QUERY_STRING.len() as u16) + 1; // 34

    let mut search_opts = new_search_opts(SearchMode::Forward);
    let mut out = Vec::with_capacity(1500);

    for c in QUERY_STRING.chars() {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    }
    assert_eq!(
        search_opts.cursor_position,
        usize::from(last_movable_column)
    );
    (search_opts, out, last_movable_column, QUERY_STRING)
}

#[path = "input_handling/editing.rs"]
mod editing;
#[path = "input_handling/navigation.rs"]
mod navigation;
#[path = "input_handling/screen.rs"]
mod screen;
