#![allow(clippy::trivial_regex)]

use super::*;
use crate::{search::SearchMode, state::Selection};

fn pager_with_search(text: &str, query: &str) -> PagerState {
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = text.to_string();
    pager.search_mode = SearchMode::Forward;
    pager.search_state.search_mode = SearchMode::Forward;
    pager.search_state.search_term = Some(regex::Regex::new(query).unwrap());
    pager.reformat_display().unwrap();
    pager
}

fn move_to_next_match(pager: &mut PagerState, command_queue: &mut CommandQueue) {
    handle_event(
        Command::UserInput(InputEvent::MoveToNextMatch(1)),
        pager,
        command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
}

fn assert_io_commands(
    command_queue: &mut CommandQueue,
    expected: impl IntoIterator<Item = IoCommand>,
) {
    for command in expected {
        assert_eq!(command_queue.pop_front(), Some(Command::Io(command)));
    }
    assert!(command_queue.is_empty());
}

#[test]
fn mouse_selection_updates_while_search_is_active() {
    let mut pager = pager_with_search("pager\nother", "pager");
    let mut command_queue = CommandQueue::new_zero();
    let is_exited = Arc::new(AtomicBool::new(false));

    handle_event(
        Command::UserInput(InputEvent::StartSelection { x: 0, y: 0 }),
        &mut pager,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();
    handle_event(
        Command::UserInput(InputEvent::UpdateSelection { x: 4, y: 0 }),
        &mut pager,
        &mut command_queue,
        &is_exited,
    )
    .unwrap();

    assert_eq!(
        pager.selection_anchor,
        Some(Selection {
            absolute_row: 0,
            col: 0,
        })
    );
    assert_eq!(
        pager.selection,
        Some(Selection {
            absolute_row: 0,
            col: 4,
        })
    );
    assert_eq!(pager.selected_text().as_deref(), Some("pager"));
    assert!(!command_queue.is_empty());
}

#[test]
fn next_match_scrolls_only_after_visible_occurrences() {
    let mut pager = pager_with_search("match and match\nmatch\nmatch\nmatch\n", "match");
    pager.rows = 4;
    let mut command_queue = CommandQueue::new_zero();

    assert_eq!(pager.content_rows(), 3);
    assert_eq!(pager.search_state.search_matches.len(), 5);

    move_to_next_match(&mut pager, &mut command_queue);

    assert_eq!(pager.search_state.search_mark, 1);
    assert_io_commands(
        &mut command_queue,
        [IoCommand::RedrawDisplay, IoCommand::RedrawPrompt],
    );

    move_to_next_match(&mut pager, &mut command_queue);

    assert_eq!(pager.search_state.search_mark, 2);
    assert_io_commands(
        &mut command_queue,
        [IoCommand::RedrawDisplay, IoCommand::RedrawPrompt],
    );

    move_to_next_match(&mut pager, &mut command_queue);

    assert_eq!(pager.search_state.search_mark, 3);
    assert_io_commands(
        &mut command_queue,
        [IoCommand::RedrawDisplay, IoCommand::RedrawPrompt],
    );

    move_to_next_match(&mut pager, &mut command_queue);

    assert_eq!(pager.search_state.search_mark, 4);
    assert_io_commands(
        &mut command_queue,
        [IoCommand::SetUpperMark(1), IoCommand::RedrawPrompt],
    );
}
