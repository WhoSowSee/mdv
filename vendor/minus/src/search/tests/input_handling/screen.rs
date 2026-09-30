use super::*;

#[test]
fn forward_sequential_text_input_screen_data() {
    let (search_opts, out, _last_movable_column, query_string) = pretest_setup_forward_search();

    let mut result_out = Vec::with_capacity(1500);

    let mut string = String::with_capacity(query_string.len());
    let mut cursor_position: u16 = 1;
    for c in query_string.chars() {
        string.push(c);
        cursor_position = cursor_position.saturating_add(1);
        write!(
            result_out,
            "{move_to_prompt}\r{clear_line}/{string}{move_to_position}",
            move_to_prompt = MoveTo(0, search_opts.rows),
            clear_line = Clear(ClearType::CurrentLine),
            move_to_position = MoveTo(cursor_position, search_opts.rows),
        )
        .unwrap();
    }
    assert_eq!(out, result_out);
}

#[test]
fn backward_sequential_text_input_screen_data() {
    const QUERY_STRING: &str = "this is@complex-text_search?query"; // length = 33
    #[allow(clippy::cast_possible_truncation)]
    const LAST_MOVABLE_COLUMN: u16 = (QUERY_STRING.len() as u16) + 1; // 34

    let mut search_opts = new_search_opts(SearchMode::Reverse);
    let mut out = Vec::with_capacity(1500);

    for c in QUERY_STRING.chars() {
        search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
        handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
    }
    assert_eq!(
        search_opts.cursor_position,
        usize::from(LAST_MOVABLE_COLUMN)
    );

    let mut result_out = Vec::with_capacity(1500);

    let mut string = String::with_capacity(QUERY_STRING.len());
    let mut cursor_position: u16 = 1;
    for c in QUERY_STRING.chars() {
        string.push(c);
        cursor_position = cursor_position.saturating_add(1);
        write!(
            result_out,
            "{move_to_prompt}\r{clear_line}?{string}{move_to_position}",
            move_to_prompt = MoveTo(0, search_opts.rows),
            clear_line = Clear(ClearType::CurrentLine),
            move_to_position = MoveTo(cursor_position, search_opts.rows),
        )
        .unwrap();
    }
    assert_eq!(out, result_out);
}
