use super::*;
use crate::LineNavigation;
use crossterm::event::{KeyEvent, MouseEvent};

#[test]
fn scrolling_outline_updates_only_panel_without_screen_clear() {
    use crate::minus_core::{
        CommandQueue,
        commands::{Command, IoCommand},
        ev_handler::handle_event,
        utils::display,
    };
    use std::sync::{Arc, atomic::AtomicBool};
    let mut state = state();
    act(&mut state, InputEvent::ToggleToc);
    for action in [InputEvent::ScrollToc(3), InputEvent::ScrollTocKeyboard(-3)] {
        let mut queue = CommandQueue::new_zero();
        handle_event(
            Command::UserInput(action),
            &mut state,
            &mut queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(queue.pop_front(), Some(Command::Io(IoCommand::RedrawToc)));
        let before = state.upper_mark;
        let mut output = Vec::new();
        display::draw_toc_update(&mut output, &mut state).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains("\x1b[2J"));
        assert!(!output.contains("row"));
        assert!(output.contains("Heading"));
        assert!(output.starts_with("\x1b[?2026h"));
        assert!(output.ends_with("\x1b[?2026l"));
        assert_eq!(state.upper_mark, before);
    }
}

fn act(state: &mut PagerState, action: InputEvent) {
    state.handle_toc_action(action).unwrap();
}

fn state() -> PagerState {
    let mut state = PagerState::new().unwrap();
    let text = "row\n".repeat(60);
    let entries = [(1, 0), (3, 1), (20, 0), (55, 0), (59, 1)]
        .into_iter()
        .map(|(source_line, depth)| TocEntry {
            title: format!("Heading {source_line}"),
            source_line,
            depth,
        })
        .collect();
    state
        .replace_mapped_text(
            text,
            Some(LineNavigation::from_current((1..=60).map(Some).collect()).with_toc(entries)),
        )
        .unwrap();
    state
}

#[test]
fn section_navigation_cycles_subsections_and_tracks_scroll() {
    for source_view in [false, true] {
        let mut state = state();
        if source_view {
            let entries = state.toc_entries().to_vec();
            let map = (1..=60).map(Some).collect();
            let source_map = (1..=60).flat_map(|line| [Some(line), None]).collect();
            let navigation =
                LineNavigation::new("numbered\n".repeat(120), map, source_map).with_toc(entries);
            state
                .replace_mapped_text("row\n".repeat(60), Some(navigation))
                .unwrap();
            assert!(state.begin_line_navigation().unwrap());
            state.finish_line_navigation(Some(1)).unwrap();
        }
        let scale = if source_view { 2 } else { 1 };
        act(&mut state, InputEvent::ToggleToc);
        for (number, index) in [(1, 0), (1, 1), (1, 0), (9, 0)] {
            act(&mut state, InputEvent::CycleToc(number));
            assert_eq!(state.active_toc(), index);
        }
        act(&mut state, InputEvent::MoveToc(1));
        assert_eq!(state.upper_mark, 19 * scale);
        assert_eq!(state.active_toc(), 2);
        assert_eq!(state.line_navigation_session.is_some(), source_view);
        act(&mut state, InputEvent::SelectToc(4));
        assert_eq!(state.active_toc(), 4);
        assert_eq!(state.upper_mark, state.max_upper_mark());
        state.upper_mark = 2 * scale;
        assert_eq!(state.active_toc(), 1);
        act(&mut state, InputEvent::ToggleToc);
        assert_eq!(state.upper_mark, 2 * scale);
        assert!(!state.toc_visible());
    }
}

#[test]
fn panel_mouse_and_keyboard_do_not_steal_document_controls() {
    let mut state = state();
    act(&mut state, InputEvent::ToggleToc);
    let classify = |code, modifiers| state.toc_input(&Event::Key(KeyEvent::new(code, modifiers)));
    for (arrow, letter, scroll, direction) in
        [(KeyCode::Up, 'K', 'U', -1), (KeyCode::Down, 'J', 'D', 1)]
    {
        assert_eq!(
            classify(arrow, KeyModifiers::ALT),
            Some(InputEvent::MoveTocEntry(direction))
        );
        for code in [arrow, KeyCode::Char(letter)] {
            assert_eq!(
                classify(code, KeyModifiers::SHIFT),
                Some(InputEvent::MoveToc(direction))
            );
        }
        assert_eq!(
            classify(arrow, KeyModifiers::ALT | KeyModifiers::SHIFT),
            classify(KeyCode::Char(scroll), KeyModifiers::SHIFT)
        );
        for modifiers in [
            KeyModifiers::NONE,
            KeyModifiers::CONTROL,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ] {
            assert_eq!(classify(arrow, modifiers), None);
        }
    }
    assert_eq!(classify(KeyCode::Char('j'), KeyModifiers::NONE), None);
    let mouse = |column, row, kind| {
        Event::Mouse(MouseEvent {
            column,
            row,
            kind,
            modifiers: KeyModifiers::NONE,
        })
    };
    assert_eq!(
        state.toc_input(&mouse(2, 2, MouseEventKind::Down(MouseButton::Left))),
        Some(InputEvent::SelectToc(1))
    );
    assert_eq!(
        state.toc_input(&mouse(60, 4, MouseEventKind::ScrollDown)),
        None
    );
    act(&mut state, InputEvent::ScrollToc(isize::MAX));
    assert!(state.toc_offset() < state.toc_entries().len());
}

#[test]
fn panel_renders_bounded_unicode_rows_and_obeys_color_policy() {
    let mut state = state();
    act(&mut state, InputEvent::ToggleToc);
    state.line_navigation.as_mut().unwrap().toc[0].title = "Раздел 界 👩‍💻 ".repeat(10);
    let rows = regex::Regex::new(r"\x1b\[[0-9;]*H").unwrap();
    let sgr = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    for cols in [0, 1, 12, 80] {
        state.cols = cols;
        for (enabled, depth) in [
            (false, crate::ColorDepth::TrueColor),
            (true, crate::ColorDepth::TrueColor),
            (true, crate::ColorDepth::Ansi256),
            (true, crate::ColorDepth::Ansi16),
        ] {
            state.output_styling = enabled;
            state.color_depth = depth;
            let mut bytes = Vec::new();
            state.draw_toc(&mut bytes).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("48;"));
            if enabled && depth == crate::ColorDepth::TrueColor && cols == 80 {
                assert!(text.contains("38;2;126;156;216m"));
                assert!(text.starts_with("\x1b[0m"));
            }
            if !enabled {
                assert!(!sgr.is_match(&text));
            }
            if depth == crate::ColorDepth::Ansi16 {
                assert!(!text.contains("38;"));
            }
            for row in rows.split(&text).skip(1) {
                let plain = crate::selection::strip_ansi(row);
                assert!(unicode_width::UnicodeWidthStr::width(plain.as_ref()) <= state.toc_width());
            }
        }
    }
}

#[test]
fn docked_panel_reflows_content_and_maps_mouse_selection() {
    let mut state = state();
    state
        .replace_mapped_text(
            format!("{}\n", "word ".repeat(14)),
            Some(LineNavigation::from_current(vec![Some(1)])),
        )
        .unwrap();
    assert_eq!(state.screen.formatted_lines_count(), 1);
    act(&mut state, InputEvent::ToggleToc);
    assert_eq!(state.content_columns(), 44);
    assert_eq!(state.screen.formatted_lines_count(), 2);
    let selection = state.selection_from_coordinates(36, 0).unwrap();
    assert_eq!(selection.col, 0);
    let selection = state.selection_from_coordinates(40, 0).unwrap();
    assert_eq!(selection.col, 4);
    let rows = state.render_rows_for_display(0, 2);
    assert!(rows.iter().all(|row| row.starts_with(&" ".repeat(36))));
    state.cols = 50;
    state.reformat_display().unwrap();
    assert_eq!(state.content_columns(), 50);
    assert_eq!(state.content_left(), 0);
    state.cols = 80;
    act(&mut state, InputEvent::ToggleToc);
    assert_eq!(state.screen.formatted_lines_count(), 1);
}

#[test]
fn hint_waits_for_keyboard_navigation_and_stays_dismissed_after_reload() {
    let mut state = state();
    assert!(!state.toc_hint_visible());
    act(&mut state, InputEvent::ToggleToc);
    assert!(state.toc_hint_visible());
    act(&mut state, InputEvent::ScrollToc(1));
    act(&mut state, InputEvent::SelectToc(0));
    state.toc.reset_position();
    assert!(state.toc_hint_visible());
    act(&mut state, InputEvent::ToggleToc);
    act(&mut state, InputEvent::ToggleToc);
    assert!(state.toc_hint_visible());
    act(&mut state, InputEvent::MoveTocEntry(1));
    assert_eq!(state.active_toc(), 1);
    assert_eq!(state.upper_mark, 2);
    assert!(!state.toc_hint_visible());
    act(&mut state, InputEvent::MoveTocEntry(1));
    assert_eq!(state.active_toc(), 2);
    act(&mut state, InputEvent::MoveTocEntry(-1));
    assert_eq!(state.active_toc(), 1);
    state
        .replace_mapped_text(
            "row\n".into(),
            Some(LineNavigation::from_current(vec![Some(1)])),
        )
        .unwrap();
    act(&mut state, InputEvent::ToggleToc);
    act(&mut state, InputEvent::ToggleToc);
    assert!(!state.toc_hint_visible());
    assert!(state.toc_visible());
    assert!(state.toc_entries().is_empty());
    assert_eq!(state.active_toc(), 0);
    act(&mut state, InputEvent::MoveToc(1));
}

#[test]
fn sidebar_requests_new_layout_before_wrapping_and_preserves_source_anchor() {
    use std::sync::{Arc, Mutex};
    let mut state = state();
    let widths = Arc::new(Mutex::new(Vec::new()));
    let captured = widths.clone();
    state.layout_renderer = Some(Arc::new(move |width| {
        captured.lock().unwrap().push(width);
        let line = format!("│{}│\n", " ".repeat(width.saturating_sub(2)));
        Ok((
            line.repeat(60),
            Some(LineNavigation::from_current((1..=60).map(Some).collect())),
        ))
    }));
    state.upper_mark = 19;
    act(&mut state, InputEvent::ToggleToc);
    assert_eq!(state.upper_mark, 19);
    assert_eq!(state.screen.formatted_lines_count(), 60);
    assert!(
        state
            .screen
            .formatted_lines
            .iter()
            .all(|line| line.starts_with('│') && line.ends_with('│'))
    );
    act(&mut state, InputEvent::ToggleToc);
    assert_eq!(state.upper_mark, 19);
    assert_eq!(*widths.lock().unwrap(), vec![44, 80]);
    use crate::minus_core::{CommandQueue, commands::Command, ev_handler::handle_event};
    use std::sync::atomic::AtomicBool;
    let mut queue = CommandQueue::new_zero();
    let exited = Arc::new(AtomicBool::new(false));
    handle_event(
        Command::SetMappedData("replacement\n".into(), None),
        &mut state,
        &mut queue,
        &exited,
    )
    .unwrap();
    assert_eq!(state.screen.orig_text, "replacement\n");
    assert_eq!(widths.lock().unwrap().len(), 2);
    handle_event(Command::RefreshLayout, &mut state, &mut queue, &exited).unwrap();
    assert_eq!(state.screen.formatted_lines_count(), 60);
    assert_eq!(*widths.lock().unwrap(), vec![44, 80, 80]);
}
