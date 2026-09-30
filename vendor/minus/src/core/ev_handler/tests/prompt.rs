use super::*;

#[test]
fn prompt_renderer_tracks_prompt_updates_and_horizontal_scroll() {
    let mut ps = PagerState::new().unwrap();
    ps.cols = 5;
    ps.screen.line_wrapping = false;
    ps.screen.orig_text = "0123456789".to_string();
    ps.reformat_display().unwrap();
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));

    handle_event(
        Command::SetPromptRenderer(Some(Arc::new(|context| {
            Ok(PromptLine::new()
                .left(PromptSpan::new(context.prompt(), PromptStyle::default())?)
                .right(PromptSpan::new(
                    context.left_mark().to_string(),
                    PromptStyle::default(),
                )?)
                .truncation_indicator(PromptSpan::new("…", PromptStyle::default())?))
        }))),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    handle_event(
        Command::SetPrompt("updated".to_string()),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    handle_event(
        Command::UserInput(InputEvent::UpdateLeftMark(1)),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.displayed_prompt, "upd…1\x1b[0m");

    ps.cols = 80;
    handle_event(
        Command::SetPromptRenderer(None),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    assert!(ps.displayed_prompt.contains("updated"));
}

#[test]
fn prompt_panel_reserves_rows_and_preserves_bottom_position() {
    let mut ps = PagerState::new().unwrap();
    ps.rows = 10;
    ps.screen.orig_text = (0..30)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    ps.reformat_display().unwrap();
    ps.upper_mark = ps.max_upper_mark();
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));
    let panel = vec![
        PromptLine::plain("help one").unwrap(),
        PromptLine::plain("help two").unwrap(),
    ];

    handle_event(
        Command::SetPromptPanel(panel),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.content_rows(), 7);
    assert_eq!(ps.upper_mark, 24);
    assert_eq!(ps.displayed_prompt_panel.len(), 2);
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );

    handle_event(
        Command::SetPromptPanel(Vec::new()),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.content_rows(), 9);
    assert_eq!(ps.upper_mark, 22);
}

#[test]
#[cfg(any(feature = "dynamic_output", feature = "static_output"))]
fn set_prompt() {
    let mut ps = PagerState::new().unwrap();
    let ev = Command::SetPrompt(TEST_STR.to_string());
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        ev,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(ps.prompt, TEST_STR.to_string());
}

#[test]
#[cfg(any(feature = "dynamic_output", feature = "static_output"))]
fn send_message() {
    let mut ps = PagerState::new().unwrap();
    let ev = Command::SendMessage(TEST_STR.to_string());
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        ev,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(ps.message.unwrap(), TEST_STR.to_string());
}

#[test]
fn timed_message_clear_is_generation_guarded() {
    let mut ps = PagerState::new().unwrap();
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));

    handle_event(
        Command::SetTimedMessage {
            text: "older".to_string(),
            id: 7,
        },
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    handle_event(
        Command::SetTimedMessage {
            text: "newer".to_string(),
            id: 8,
        },
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    assert_eq!(ps.message.as_deref(), Some("newer"));
    assert_eq!(ps.message_id, Some(8));
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawPrompt))
    );
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawPrompt))
    );

    handle_event(
        Command::ClearMessage(7),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.message.as_deref(), Some("newer"));
    assert_eq!(ps.message_id, Some(8));
    assert!(command_queue.is_empty());

    handle_event(
        Command::ClearMessage(8),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.message, None);
    assert_eq!(ps.message_id, None);
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawPrompt))
    );
}
