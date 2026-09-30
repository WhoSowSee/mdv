use super::*;

#[test]
fn home_end_keys() {
    let (mut search_opts, mut out, last_movable_column, _) = pretest_setup_forward_search();

    search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(search_opts.cursor_position, 1);

    search_opts.ev = Some(make_event_from_keycode(KeyCode::End));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(
        search_opts.cursor_position,
        usize::from(last_movable_column)
    );
}

#[test]
fn basic_left_arrow_movement() {
    const FIRST_MOVABLE_COLUMN: u16 = 1;
    let (mut search_opts, mut out, last_movable_column, _) = pretest_setup_forward_search();
    let query_string_length = last_movable_column - 1;

    for i in (FIRST_MOVABLE_COLUMN..=query_string_length).rev() {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
        assert_eq!(search_opts.cursor_position, usize::from(i));
    }
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(
        search_opts.cursor_position,
        usize::from(FIRST_MOVABLE_COLUMN)
    );
}

#[test]
fn basic_right_arrow_movement() {
    let (mut search_opts, mut out, last_movable_column, _) = pretest_setup_forward_search();
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    for i in 2..=last_movable_column {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Right));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
        assert_eq!(search_opts.cursor_position, usize::from(i));
    }
    search_opts.ev = Some(make_event_from_keycode(KeyCode::Right));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(
        search_opts.cursor_position,
        usize::from(last_movable_column)
    );
}

#[test]
fn right_jump_by_word() {
    const JUMP_COLUMNS: [u16; 10] = [1, 5, 6, 8, 9, 16, 17, 28, 29, LAST_MOVABLE_COLUMN];
    let (mut search_opts, mut out, _last_movable_column, _) = pretest_setup_forward_search();
    #[allow(clippy::items_after_statements)]
    const LAST_MOVABLE_COLUMN: u16 = 34;

    search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

    let ev = Event::Key(KeyEvent {
        code: KeyCode::Right,
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::CONTROL,
        state: KeyEventState::NONE,
    });

    for i in &JUMP_COLUMNS[1..] {
        search_opts.ev = Some(ev.clone());
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
        assert_eq!(search_opts.cursor_position, usize::from(*i));
    }
    search_opts.ev = Some(ev);
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(
        search_opts.cursor_position,
        usize::from(LAST_MOVABLE_COLUMN)
    );
}

#[test]
fn left_jump_by_word() {
    const JUMP_COLUMNS: [u16; 10] = [1, 5, 6, 8, 9, 16, 17, 28, 29, LAST_MOVABLE_COLUMN];
    let (mut search_opts, mut out, _last_movable_column, _) = pretest_setup_forward_search();
    #[allow(clippy::items_after_statements)]
    const LAST_MOVABLE_COLUMN: u16 = 34;

    let ev = Event::Key(KeyEvent {
        code: KeyCode::Left,
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::CONTROL,
        state: KeyEventState::NONE,
    });

    for i in (JUMP_COLUMNS[..(JUMP_COLUMNS.len() - 1)]).iter().rev() {
        search_opts.ev = Some(ev.clone());
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
        assert_eq!(search_opts.cursor_position, usize::from(*i));
    }
    search_opts.ev = Some(ev);
    handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    assert_eq!(search_opts.cursor_position, usize::from(JUMP_COLUMNS[0]));
}
