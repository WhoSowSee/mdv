use super::{
    Clear, ClearType, Duration, Event, FetchInputResult, INVERT, InputStatus, KeyCode, KeyEvent,
    KeyEventKind, KeyModifiers, MinusError, NORMAL, PagerState, Regex, SearchOpts, Write,
    byte_index_at_character_position, cursor, event, query_end_position, run_incremental_search,
    term, word_start_positions,
};

#[allow(clippy::too_many_lines)]
pub(super) fn handle_key_press<O, F>(
    out: &mut O,
    so: &mut SearchOpts<'_>,
    incremental_search_condition: F,
) -> crate::Result
where
    O: Write,
    F: Fn(&SearchOpts<'_>) -> bool,
{
    const FIRST_AVAILABLE_COLUMN: usize = 1;
    let last_available_column = query_end_position(&so.string);

    if so.ev.is_none() {
        return Ok(());
    }

    let refresh_display = |out: &mut O, so: &mut SearchOpts<'_>| -> Result<(), MinusError> {
        so.compiled_regex = if so.string.is_empty() {
            None
        } else {
            Regex::new(&so.string).ok()
        };

        so.preview_upper_mark = run_incremental_search(out, so, incremental_search_condition)?;

        term::move_cursor(out, 0, so.rows, false)?;
        write!(out, "\r{}{}", Clear(ClearType::CurrentLine), so.prompt)?;
        write_search_query(out, so)?;
        Ok(())
    };
    match so.ev.as_ref().unwrap() {
        Event::Key(KeyEvent { kind, .. }) if *kind != KeyEventKind::Press => (),
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            so.input_status = InputStatus::Cancelled;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if !clear_selected_query(so) {
                if so.cursor_position == FIRST_AVAILABLE_COLUMN {
                    return Ok(());
                }
                so.cursor_position = so.cursor_position.saturating_sub(1);
                let byte_index = byte_index_at_character_position(&so.string, so.cursor_position);
                so.string.remove(byte_index);
            }
            so.word_index = word_start_positions(&so.string);
            refresh_display(out, so)?;
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, false)?;
            out.flush()?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if !clear_selected_query(so) {
                if so.cursor_position >= last_available_column {
                    return Ok(());
                }
                let byte_index = byte_index_at_character_position(&so.string, so.cursor_position);
                so.string.remove(byte_index);
            }
            so.word_index = word_start_positions(&so.string);
            refresh_display(out, so)?;
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, false)?;
            out.flush()?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            so.input_status = InputStatus::Confirmed;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, FIRST_AVAILABLE_COLUMN)? {
                return Ok(());
            }
            if so.cursor_position == FIRST_AVAILABLE_COLUMN {
                return Ok(());
            }
            so.cursor_position = so.cursor_position.saturating_sub(1);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if collapse_query_selection(out, so, FIRST_AVAILABLE_COLUMN)? {
                return Ok(());
            }
            so.cursor_position = *so
                .word_index
                .iter()
                .rfind(|c| c < &&so.cursor_position)
                .unwrap_or(&FIRST_AVAILABLE_COLUMN);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, last_available_column)? {
                return Ok(());
            }
            if so.cursor_position >= last_available_column {
                return Ok(());
            }
            so.cursor_position = so.cursor_position.saturating_add(1);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if collapse_query_selection(out, so, last_available_column)? {
                return Ok(());
            }
            so.cursor_position = *so
                .word_index
                .iter()
                .find(|c| c > &&so.cursor_position)
                .unwrap_or(&last_available_column);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, FIRST_AVAILABLE_COLUMN)? {
                return Ok(());
            }
            so.cursor_position = 1;
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, last_available_column)? {
                return Ok(());
            }
            so.cursor_position = query_end_position(&so.string);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        }) if modifiers.is_empty() || *modifiers == KeyModifiers::SHIFT => {
            let c = *c;
            clear_selected_query(so);
            let byte_index = byte_index_at_character_position(&so.string, so.cursor_position);
            so.string.insert(byte_index, c);
            so.word_index = word_start_positions(&so.string);
            refresh_display(out, so)?;
            so.cursor_position = so.cursor_position.saturating_add(1);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, false)?;
            out.flush()?;
        }
        _ => return Ok(()),
    }
    Ok(())
}

pub(super) fn write_search_query(
    out: &mut impl std::io::Write,
    search_opts: &SearchOpts<'_>,
) -> Result<(), MinusError> {
    if search_opts.query_selected && search_opts.output_styling {
        write!(out, "{}{}{}", *INVERT, search_opts.string, *NORMAL)?;
    } else {
        write!(out, "{}", search_opts.string)?;
    }
    Ok(())
}

pub(super) fn write_search_input(
    out: &mut impl std::io::Write,
    search_opts: &SearchOpts<'_>,
) -> Result<(), MinusError> {
    term::move_cursor(out, 0, search_opts.rows, false)?;
    write!(
        out,
        "{}{}",
        Clear(ClearType::CurrentLine),
        search_opts.prompt
    )?;
    write_search_query(out, search_opts)?;
    write!(out, "{}", cursor::Show)?;
    term::move_cursor(
        out,
        search_opts.terminal_cursor_column(),
        search_opts.rows,
        false,
    )?;
    out.flush()?;
    Ok(())
}

pub(super) fn clear_selected_query(search_opts: &mut SearchOpts<'_>) -> bool {
    if !search_opts.query_selected {
        return false;
    }
    search_opts.string.clear();
    search_opts.cursor_position = 1;
    search_opts.query_selected = false;
    true
}

pub(super) fn collapse_query_selection(
    out: &mut impl std::io::Write,
    search_opts: &mut SearchOpts<'_>,
    cursor_position: usize,
) -> Result<bool, MinusError> {
    if !search_opts.query_selected {
        return Ok(false);
    }
    search_opts.query_selected = false;
    search_opts.cursor_position = cursor_position;
    write_search_input(out, search_opts)?;
    Ok(true)
}

#[cfg(feature = "search")]
pub fn fetch_input(
    out: &mut impl std::io::Write,
    ps: &PagerState,
) -> Result<FetchInputResult, MinusError> {
    let mut search_opts = SearchOpts::from(ps);

    write_search_input(out, &search_opts)?;

    loop {
        if event::poll(Duration::from_millis(100)).map_err(|e| MinusError::HandleEvent(e.into()))? {
            let ev = event::read().map_err(|e| MinusError::HandleEvent(e.into()))?;
            search_opts.ev = Some(ev);
            handle_key_press(
                out,
                &mut search_opts,
                &ps.search_state.incremental_search_condition,
            )?;
            search_opts.ev = None;
        }
        if search_opts.input_status.done() {
            break;
        }
    }
    term::move_cursor(out, 0, search_opts.rows, false)?;
    write!(out, "{}{}", Clear(ClearType::CurrentLine), cursor::Hide)?;
    out.flush()?;

    Ok(FetchInputResult {
        string: search_opts.string,
        compiled_regex: search_opts.compiled_regex,
        input_status: search_opts.input_status,
        preview_upper_mark: search_opts.preview_upper_mark,
    })
}
