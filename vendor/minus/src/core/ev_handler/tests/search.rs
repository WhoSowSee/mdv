use super::*;

#[cfg(feature = "search")]
#[test]
fn cancel_search_clears_highlights_but_keeps_the_query_for_reuse() {
    let mut ps = pager_with_active_search();
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));
    assert!(!ps.search_state.search_matches.is_empty());

    handle_event(
        Command::UserInput(InputEvent::CancelSearch),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert!(ps.search_state.search_term.is_none());
    assert!(ps.search_state.search_matches.is_empty());
    assert_eq!(ps.search_mode, crate::search::SearchMode::Unknown);
    assert_eq!(
        ps.search_state.search_mode,
        crate::search::SearchMode::Unknown
    );
    assert!(!is_exited.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );

    handle_event(
        Command::UserInput(InputEvent::Search(crate::search::SearchMode::Forward)),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    let search_opts = crate::search::SearchOpts::from(&ps);

    assert_eq!(search_opts.string, "pager");
}

#[cfg(feature = "search")]
#[test]
fn manually_cleared_search_input_forgets_the_saved_query() {
    let mut ps = pager_with_active_search();
    let pager = Pager::new();
    let mut command_queue = CommandQueue::new_zero();

    super::apply_search_result(
        &mut ps,
        &pager,
        &mut command_queue,
        crate::search::FetchInputResult {
            string: String::new(),
            compiled_regex: None,
            input_status: crate::search::InputStatus::Cancelled,
            preview_upper_mark: None,
        },
    )
    .unwrap();

    assert!(ps.search_state.last_search_query.is_empty());
    assert!(ps.search_state.search_term.is_none());
    assert!(ps.search_state.search_matches.is_empty());
    assert_eq!(ps.search_mode, crate::search::SearchMode::Unknown);
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );

    ps.search_state.search_mode = crate::search::SearchMode::Forward;
    assert!(crate::search::SearchOpts::from(&ps).string.is_empty());
}

#[cfg(feature = "search")]
#[test]
fn cancelled_search_input_preserves_the_new_draft() {
    let mut ps = pager_with_active_search();
    let pager = Pager::new();
    let mut command_queue = CommandQueue::new_zero();

    super::apply_search_result(
        &mut ps,
        &pager,
        &mut command_queue,
        crate::search::FetchInputResult {
            string: "[draft".to_string(),
            compiled_regex: None,
            input_status: crate::search::InputStatus::Cancelled,
            preview_upper_mark: None,
        },
    )
    .unwrap();

    assert_eq!(
        ps.search_state
            .search_term
            .as_ref()
            .map(regex::Regex::as_str),
        Some("pager")
    );
    assert_eq!(crate::search::SearchOpts::from(&ps).string, "[draft");
}

#[cfg(feature = "search")]
#[test]
fn cancelled_search_input_keeps_incremental_preview_position() {
    let mut ps = pager_with_active_search();
    let pager = Pager::new();
    let mut command_queue = CommandQueue::new_zero();

    super::apply_search_result(
        &mut ps,
        &pager,
        &mut command_queue,
        crate::search::FetchInputResult {
            string: "pager".to_string(),
            compiled_regex: None,
            input_status: crate::search::InputStatus::Cancelled,
            preview_upper_mark: Some(1),
        },
    )
    .unwrap();

    assert_eq!(ps.upper_mark, 1);
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );
}

#[cfg(feature = "search")]
#[test]
fn empty_search_notification_is_dismissed_by_next_search() {
    let mut ps = PagerState::new().unwrap();
    let pager = Pager::new();
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));
    ps.screen.orig_text = TEST_STR.to_string();
    ps.search_state.search_term = Some(regex::Regex::new(r"missing\s+section").unwrap());
    ps.reformat_display().unwrap();
    ps.upper_mark = 3;

    assert!(ps.search_state.search_matches.is_empty());
    super::set_search_position(&mut ps, &pager).unwrap();
    assert_eq!(ps.upper_mark, 3);
    assert_eq!(
        super::NOT_FOUND_MESSAGE_DURATION,
        std::time::Duration::from_secs(2)
    );
    let notification = pager.rx.try_recv().unwrap();
    assert_eq!(
        notification,
        Command::SetTimedMessage {
            text: "No matches found".to_string(),
            id: 1,
        }
    );

    handle_event(notification, &mut ps, &mut command_queue, &is_exited).unwrap();
    assert_eq!(ps.message.as_deref(), Some("No matches found"));
    assert_eq!(ps.message_id, Some(1));
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawPrompt))
    );

    handle_event(
        Command::UserInput(InputEvent::Search(crate::search::SearchMode::Forward)),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.message, None);
    assert_eq!(ps.message_id, None);
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::FetchSearchQuery))
    );
}

#[cfg(feature = "search")]
#[test]
fn missing_source_line_uses_the_standard_timed_notification() {
    let mut ps = PagerState::new().unwrap();
    let pager = Pager::new();
    let mut command_queue = CommandQueue::new_zero();
    ps.screen.orig_text = "first\nsecond".to_string();
    ps.screen.line_count = 2;
    ps.reformat_display().unwrap();
    ps.line_navigation = Some(LineNavigation::new(
        "1 first\n2 second",
        vec![Some(1), Some(2)],
        vec![Some(1), Some(2)],
    ));
    assert!(ps.begin_line_navigation().unwrap());

    super::apply_line_navigation_result(
        &mut ps,
        &pager,
        &mut command_queue,
        crate::line_navigation::LineInputResult::Confirmed(Some(99)),
    )
    .unwrap();

    assert!(!ps.line_navigation_is_active());
    assert_eq!(ps.screen.orig_text, "first\nsecond");
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );
    assert_eq!(
        pager.rx.try_recv().unwrap(),
        Command::SetTimedMessage {
            text: "Line not found".to_string(),
            id: 1,
        }
    );
}
