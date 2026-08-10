#[cfg(feature = "search")]
use crate::SearchMode;
use crate::{
    LineNumbers, PagerState,
    input::{DefaultInputClassifier, InputClassifier, InputEvent},
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers, MouseEvent, MouseEventKind,
};

fn handle_input(ev: Event, p: &PagerState) -> Option<InputEvent> {
    p.input_classifier.classify_input(ev, p)
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_kb_nav() {
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 12;
    pager.line_numbers = LineNumbers::Enabled;
    pager.rows = 5;

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        assert_eq!(
            Some(InputEvent::UpdateUpperMark(pager.upper_mark + 1)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(pager.upper_mark - 1)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(0)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(8)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(8)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(usize::MAX - 1)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(usize::MAX - 1)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(usize::MAX - 1)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(16)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(16)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(14)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(10)),
            handle_input(ev, &pager)
        );
    }
    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(13)),
            handle_input(ev, &pager)
        );
    }
}

#[test]
fn space_has_default_line_down_binding() {
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 12;
    pager.rows = 5;
    let event = Event::Key(KeyEvent {
        code: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    assert_eq!(
        Some(InputEvent::UpdateUpperMark(13)),
        handle_input(event.clone(), &pager)
    );
    assert_eq!(
        Some(InputEvent::UpdateUpperMark(13)),
        DefaultInputClassifier.classify_input(event, &pager)
    );
}

#[test]
fn test_restore_prompt() {
    let mut pager = PagerState::new().unwrap();
    pager.message = Some("Prompt message".to_string());
    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::RestorePrompt),
            pager.input_classifier.classify_input(ev, &pager)
        );
    }
}

#[test]
fn test_mouse_nav() {
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 12;
    pager.line_numbers = LineNumbers::Enabled;
    pager.rows = 5;
    {
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            row: 0,
            column: 0,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            Some(InputEvent::UpdateUpperMark(pager.upper_mark + 5)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            row: 0,
            column: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(pager.upper_mark - 5)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            row: 3,
            column: 7,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            Some(InputEvent::StartSelection { x: 7, y: 3 }),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(crossterm::event::MouseButton::Left),
            row: 4,
            column: 9,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateSelection { x: 9, y: 4 }),
            handle_input(ev, &pager)
        );
    }

    #[cfg(feature = "clipboard")]
    {
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Up(crossterm::event::MouseButton::Left),
            row: 4,
            column: 9,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            Some(InputEvent::FinalizeSelection),
            handle_input(ev, &pager)
        );
    }
}

#[test]
fn test_saturation() {
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 12;
    pager.line_numbers = LineNumbers::Enabled;
    pager.rows = 5;

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        let mut pager = PagerState::new().unwrap();
        pager.upper_mark = usize::MAX;
        pager.line_numbers = LineNumbers::Enabled;
        pager.rows = 5;
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(usize::MAX)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        let mut pager = PagerState::new().unwrap();
        pager.upper_mark = usize::MIN;
        pager.line_numbers = LineNumbers::Enabled;
        pager.rows = 5;
        assert_eq!(
            Some(InputEvent::UpdateUpperMark(usize::MIN)),
            handle_input(ev, &pager)
        );
    }
}

#[test]
fn test_misc_events() {
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 12;
    pager.line_numbers = LineNumbers::Enabled;
    pager.rows = 5;

    {
        let ev = Event::Resize(42, 35);
        assert_eq!(
            Some(InputEvent::UpdateTermArea(42, 35)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::UpdateLineNumber(!pager.line_numbers)),
            handle_input(ev, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(Some(InputEvent::Exit), handle_input(ev, &pager));
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(Some(InputEvent::Exit), handle_input(ev, &pager));
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(Some(InputEvent::Exit), handle_input(ev, &pager));
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(Some(InputEvent::Ignore), handle_input(ev, &pager));
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('5'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(Some(InputEvent::Number('5')), handle_input(ev, &pager));
    }

    #[cfg(feature = "clipboard")]
    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(Some(InputEvent::CopySelection), handle_input(ev, &pager));
    }
}

#[test]
#[allow(clippy::too_many_lines)]
#[cfg(feature = "search")]
fn test_search_bindings() {
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 12;
    pager.line_numbers = LineNumbers::Enabled;
    pager.rows = 5;

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::Search(SearchMode::Forward)),
            handle_input(ev, &pager)
        );
    }

    {
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::Search(SearchMode::Forward)),
            handle_input(event.clone(), &pager)
        );
        assert_eq!(
            Some(InputEvent::Search(SearchMode::Forward)),
            DefaultInputClassifier.classify_input(event, &pager)
        );
    }

    {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(
            Some(InputEvent::Search(SearchMode::Reverse)),
            handle_input(ev, &pager)
        );
    }
    {
        pager.search_state.search_mode = SearchMode::Forward;
        let next_event = Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        let prev_event = Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        assert_eq!(
            pager.input_classifier.classify_input(next_event, &pager),
            Some(InputEvent::MoveToNextMatch(1))
        );
        assert_eq!(
            pager.input_classifier.classify_input(prev_event, &pager),
            Some(InputEvent::MoveToPrevMatch(1))
        );
    }

    {
        pager.search_state.search_mode = SearchMode::Reverse;
        let next_event = Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        let prev_event = Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        assert_eq!(
            pager.input_classifier.classify_input(next_event, &pager),
            Some(InputEvent::MoveToPrevMatch(1))
        );
        assert_eq!(
            pager.input_classifier.classify_input(prev_event, &pager),
            Some(InputEvent::MoveToNextMatch(1))
        );
    }
}
