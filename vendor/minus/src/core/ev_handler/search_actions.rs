use super::{
    Arc, Command, CommandQueue, Condvar, IoCommand, LINE_NOT_FOUND_MESSAGE, MinusError, Mutex,
    NO_SEARCH_MATCH_MESSAGE, NOT_FOUND_MESSAGE_DURATION, Pager, PagerState, PromptError,
    line_navigation, queue_prompt_redraw, search,
};

#[cfg(feature = "search")]
pub(super) fn dismiss_timed_message(p: &mut PagerState) {
    if p.message_id.take().is_some() {
        p.message = None;
    }
}

#[cfg(feature = "search")]
pub(super) fn queue_current_search_match(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
) -> Result<(), PromptError> {
    let Some(search_match) = p
        .search_state
        .search_matches
        .get(p.search_state.search_mark)
    else {
        return Ok(());
    };
    let viewport_rows = p.content_rows().max(1);
    let viewport_end = p.upper_mark.saturating_add(viewport_rows);
    let next_upper_mark = if search_match.row < p.upper_mark {
        search_match.row
    } else if search_match.row >= viewport_end {
        search_match
            .row
            .saturating_add(1)
            .saturating_sub(viewport_rows)
    } else {
        p.upper_mark
    };
    if next_upper_mark == p.upper_mark {
        command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
    } else {
        command_queue.push_back(Command::Io(IoCommand::SetUpperMark(next_upper_mark)));
    }
    queue_prompt_redraw(p, command_queue)
}

#[cfg(feature = "search")]
pub(super) fn move_to_next_search_match(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
    count: usize,
) -> Result<(), PromptError> {
    let total = p.search_state.search_matches.len();
    if total == 0 {
        return Ok(());
    }
    p.search_state.search_mark = (p.search_state.search_mark + count % total) % total;
    queue_current_search_match(p, command_queue)
}

#[cfg(feature = "search")]
pub(super) fn move_to_previous_search_match(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
    count: usize,
) -> Result<(), PromptError> {
    if p.search_state.search_matches.is_empty() {
        return Ok(());
    }
    p.search_state.search_mark = p.search_state.search_mark.saturating_sub(count);
    queue_current_search_match(p, command_queue)
}
#[cfg(feature = "search")]
pub(super) fn set_search_position(p: &mut PagerState, pager: &Pager) -> Result<(), MinusError> {
    let Some(upper_mark) = p
        .search_state
        .search_matches
        .get(p.search_state.search_mark)
        .map(|search_match| search_match.row)
    else {
        pager.send_message_for(NO_SEARCH_MATCH_MESSAGE, NOT_FOUND_MESSAGE_DURATION)?;
        return Ok(());
    };
    p.upper_mark = upper_mark;
    Ok(())
}

#[cfg(feature = "search")]
pub(super) fn deactivate_search(p: &mut PagerState) -> Result<(), PromptError> {
    p.search_mode = search::SearchMode::Unknown;
    p.search_state.search_mode = search::SearchMode::Unknown;
    p.search_state.search_term = None;
    p.search_state.search_mark = 0;
    p.reformat_display()?;
    Ok(())
}

#[cfg(feature = "search")]
pub(super) fn apply_search_result(
    p: &mut PagerState,
    pager: &Pager,
    command_queue: &mut CommandQueue,
    search_result: search::FetchInputResult,
) -> Result<(), MinusError> {
    let search::FetchInputResult {
        string,
        compiled_regex,
        input_status,
        preview_upper_mark,
    } = search_result;

    if string.is_empty() {
        p.search_state.last_search_query.clear();
        deactivate_search(p)?;
        command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        return Ok(());
    }

    if input_status == search::InputStatus::Cancelled {
        p.search_state.last_search_query = string;
        if let Some(upper_mark) = preview_upper_mark {
            p.upper_mark = upper_mark;
        }
        command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        return Ok(());
    }

    let Some(search_term) = compiled_regex.or_else(|| regex::Regex::new(&string).ok()) else {
        p.search_state.last_search_query = string;
        command_queue.push_back(Command::SendMessage(
            "Invalid regular expression. Press Enter".to_string(),
        ));
        return Ok(());
    };
    p.search_state.last_search_query = string;
    p.search_state.search_term = Some(search_term);

    p.reformat_display()?;
    set_search_position(p, pager)?;
    command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
    command_queue.push_back(Command::Io(IoCommand::RedrawPrompt));
    Ok(())
}

#[cfg(feature = "search")]
pub(super) fn apply_line_navigation_result(
    p: &mut PagerState,
    pager: &Pager,
    command_queue: &mut CommandQueue,
    input: line_navigation::LineInputResult,
) -> Result<(), MinusError> {
    let (requested, notify_not_found) = match input {
        line_navigation::LineInputResult::Cancelled => (None, false),
        line_navigation::LineInputResult::Confirmed(line) => (line, true),
    };
    let found = p.finish_line_navigation(requested)?;
    command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
    if notify_not_found && !found {
        pager.send_message_for(LINE_NOT_FOUND_MESSAGE, NOT_FOUND_MESSAGE_DURATION)?;
    }
    Ok(())
}

#[cfg(feature = "search")]
pub(super) fn with_general_input_paused<T>(
    user_input_active: &Arc<(Mutex<bool>, Condvar)>,
    read: impl FnOnce() -> Result<T, MinusError>,
) -> Result<T, MinusError> {
    *user_input_active.0.lock() = false;
    user_input_active.1.notify_one();
    let result = read();
    *user_input_active.0.lock() = true;
    user_input_active.1.notify_one();
    result
}
