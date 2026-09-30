use super::*;

#[test]
fn input_sequential_text() {
    let mut search_opts = new_search_opts(SearchMode::Forward);
    let mut out = Vec::with_capacity(1500);
    for (i, c) in "text search matches".chars().enumerate() {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
        assert_eq!(search_opts.input_status, InputStatus::Active);
        assert_eq!(search_opts.cursor_position, i + 2);
    }
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Enter));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(search_opts.word_index, vec![1, 5, 6, 12, 13]);
    assert_eq!(&search_opts.string, "text search matches");
    assert_eq!(search_opts.input_status, InputStatus::Confirmed);
}

#[test]
fn shifted_unicode_query_supports_navigation_and_deletion() {
    let mut search_opts = new_search_opts(SearchMode::Forward);
    let mut out = Vec::new();
    search_opts.ev = Some(Event::Key(KeyEvent {
        code: KeyCode::Char('Å'),
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::SHIFT,
        state: KeyEventState::NONE,
    }));

    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    for c in "ngström".chars() {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    }

    assert_eq!(search_opts.string, "Ångström");
    assert_eq!(search_opts.cursor_position, 9);

    search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Backspace));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    assert_eq!(search_opts.string, "Ångstrm");
    assert_eq!(search_opts.cursor_position, 7);

    search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Delete));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    assert_eq!(search_opts.string, "ngstrm");
    assert_eq!(search_opts.cursor_position, 1);
}

#[test]
fn custom_prompt_uses_the_status_row_with_a_panel() {
    let mut state = crate::PagerState::new().unwrap();
    state.rows = 10;
    state.search_state.search_mode = SearchMode::Forward;
    state.search_prompt = Some("Find: ".to_string());
    state.prompt_panel = vec![
        crate::PromptLine::plain("help one").unwrap(),
        crate::PromptLine::plain("help two").unwrap(),
    ];
    let mut search_opts = SearchOpts::from(&state);
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Char('x')));
    let mut out = Vec::new();

    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    let rendered = String::from_utf8(out).unwrap();
    assert!(rendered.contains("Find: x"));
    assert!(rendered.contains(&MoveTo(7, search_opts.rows).to_string()));
    assert_eq!(usize::from(search_opts.rows), state.prompt_row());
}

#[test]
fn restored_query_is_selected_and_rendered() {
    let mut state = crate::PagerState::new().unwrap();
    state.search_state.search_mode = SearchMode::Forward;
    state.search_state.last_search_query = "pager".to_string();
    state.search_prompt = Some("Find: ".to_string());
    let mut search_opts = SearchOpts::from(&state);
    let mut out = Vec::new();

    write_search_input(&mut out, &search_opts).unwrap();

    assert_eq!(search_opts.string, "pager");
    assert_eq!(search_opts.cursor_position, 6);
    assert!(search_opts.query_selected);

    search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    assert_eq!(search_opts.string, "pager");
    assert_eq!(search_opts.cursor_position, 1);
    assert!(!search_opts.query_selected);
    let rendered = String::from_utf8(out).unwrap();
    assert!(rendered.contains(&format!(
        "Find: {}pager{}",
        Attribute::Reverse,
        Attribute::NoReverse
    )));
    assert!(rendered.contains(&MoveTo(11, search_opts.rows).to_string()));
}

#[test]
fn deleting_selected_query_returns_an_empty_cancelled_draft() {
    for key_code in [KeyCode::Backspace, KeyCode::Delete] {
        let mut state = crate::PagerState::new().unwrap();
        state.search_state.search_mode = SearchMode::Forward;
        state.search_state.last_search_query = "pager".to_string();
        let mut search_opts = SearchOpts::from(&state);
        let mut out = Vec::new();

        search_opts.ev = Some(make_event_from_keycode(key_code));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

        assert!(search_opts.string.is_empty());
        assert_eq!(search_opts.cursor_position, 1);
        assert!(search_opts.compiled_regex.is_none());
        assert_eq!(search_opts.input_status, InputStatus::Active);

        search_opts.ev = Some(make_event_from_keycode(KeyCode::Esc));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

        assert!(search_opts.string.is_empty());
        assert_eq!(search_opts.input_status, InputStatus::Cancelled);
    }
}

#[test]
fn input_complex_sequential_text() {
    let mut search_opts = new_search_opts(SearchMode::Forward);
    let mut out = Vec::with_capacity(1500);
    for (i, c) in "this is@complex-text_search?query".chars().enumerate() {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
        assert_eq!(search_opts.input_status, InputStatus::Active);
        assert_eq!(search_opts.cursor_position, i + 2);
    }
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Enter));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(search_opts.word_index, vec![1, 5, 6, 8, 9, 16, 17, 28, 29]);
    assert_eq!(&search_opts.string, "this is@complex-text_search?query");
    assert_eq!(search_opts.input_status, InputStatus::Confirmed);
}

#[test]
fn escape_cancels_a_non_empty_search_without_clearing_it() {
    let (mut search_opts, mut out, _, query) = pretest_setup_forward_search();

    search_opts.ev = Some(make_event_from_keycode(KeyCode::Esc));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    assert_eq!(search_opts.string, query);
    assert_eq!(search_opts.input_status, InputStatus::Cancelled);
}
