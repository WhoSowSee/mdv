use super::*;

#[test]
fn update_selection_scrolls_up_at_top_edge() {
    let mut ps = PagerState::new().unwrap();
    ps.rows = 5;
    ps.screen.orig_text = (0..10).fold(String::new(), |mut t, idx| {
        let _ = writeln!(t, "line {idx}");
        t
    });
    ps.reformat_display().unwrap();
    ps.upper_mark = 3;
    ps.selection_anchor = Some(Selection {
        absolute_row: 3,
        col: 0,
    });
    ps.selection = ps.selection_anchor;
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        Command::UserInput(InputEvent::UpdateSelection { x: 0, y: 0 }),
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    assert_eq!(ps.upper_mark, 2);
    assert_eq!(
        ps.selection,
        Some(Selection {
            absolute_row: 2,
            col: 0,
        })
    );
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );
}

#[test]
fn update_selection_scrolls_down_at_bottom_edge() {
    let mut ps = PagerState::new().unwrap();
    ps.rows = 5;
    ps.screen.orig_text = (0..10).fold(String::new(), |mut t, idx| {
        let _ = writeln!(t, "line {idx}");
        t
    });
    ps.reformat_display().unwrap();
    ps.upper_mark = 3;
    ps.selection_anchor = Some(Selection {
        absolute_row: 3,
        col: 0,
    });
    ps.selection = ps.selection_anchor;
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        Command::UserInput(InputEvent::UpdateSelection { x: 0, y: 4 }),
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    assert_eq!(ps.upper_mark, 4);
    assert_eq!(
        ps.selection,
        Some(Selection {
            absolute_row: 7,
            col: 0,
        })
    );
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawDisplay))
    );
}

#[test]
fn update_selection_clamps_scroll_at_bottom_bound() {
    let mut ps = PagerState::new().unwrap();
    ps.rows = 5;
    ps.screen.orig_text = (0..6).fold(String::new(), |mut t, idx| {
        let _ = writeln!(t, "line {idx}");
        t
    });
    ps.reformat_display().unwrap();
    ps.upper_mark = 2;
    ps.selection_anchor = Some(Selection {
        absolute_row: 2,
        col: 0,
    });
    ps.selection = ps.selection_anchor;
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        Command::UserInput(InputEvent::UpdateSelection { x: 0, y: 10 }),
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    assert_eq!(ps.upper_mark, 2);
    assert_eq!(
        ps.selection,
        Some(Selection {
            absolute_row: 5,
            col: 0,
        })
    );
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawSelection(2, 5)))
    );
}

#[test]
#[cfg(feature = "clipboard")]
fn clipboard_events_preserve_then_clear_selection() {
    let mut ps = PagerState::new().unwrap();
    ps.selection_anchor = Some(Selection {
        absolute_row: 0,
        col: 0,
    });
    ps.selection = ps.selection_anchor;
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));

    handle_event(
        Command::UserInput(InputEvent::FinalizeSelection),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert!(ps.selection.is_some());
    assert!(ps.selection_anchor.is_some());
    assert!(command_queue.is_empty());

    handle_event(
        Command::UserInput(InputEvent::CopySelection),
        &mut ps,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(ps.selection, None);
    assert_eq!(ps.selection_anchor, None);
    assert_eq!(
        command_queue.pop_front(),
        Some(Command::Io(IoCommand::RedrawSelection(0, 0)))
    );
}
