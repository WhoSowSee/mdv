mod dispatch;
mod io;
#[cfg(feature = "search")]
mod search_actions;
pub use dispatch::handle_event;
pub use io::handle_io_command;
#[cfg(feature = "search")]
use search_actions::{
    apply_line_navigation_result, apply_search_result, deactivate_search, dismiss_timed_message,
    move_to_next_search_match, move_to_previous_search_match, with_general_input_paused,
};

use std::io::Write;
use std::sync::{Arc, atomic::AtomicBool};
#[cfg(feature = "search")]
use std::time::Duration;

#[cfg(feature = "search")]
use parking_lot::{Condvar, Mutex};

use super::CommandQueue;
use super::commands::{Command, IoCommand};
use super::utils::display::{self, AppendStyle};
use crate::ExitStrategy;
#[cfg(feature = "search")]
use crate::{Pager, line_navigation, search};
use crate::{PagerState, PromptError, error::MinusError, hooks::Hook, input::InputEvent};

#[cfg(feature = "search")]
const NO_SEARCH_MATCH_MESSAGE: &str = "No matches found";
#[cfg(feature = "search")]
const LINE_NOT_FOUND_MESSAGE: &str = "Line not found";
#[cfg(feature = "search")]
const NOT_FOUND_MESSAGE_DURATION: Duration = Duration::from_secs(2);

fn queue_prompt_redraw(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
) -> Result<(), PromptError> {
    p.format_prompt()?;
    command_queue.push_back(Command::Io(IoCommand::RedrawPrompt));
    Ok(())
}

fn queue_selection_redraw(
    command_queue: &mut CommandQueue,
    previous: Option<(usize, usize)>,
    current: Option<(usize, usize)>,
) {
    let span = match (previous, current) {
        (Some(previous), Some(current)) => {
            Some((previous.0.min(current.0), previous.1.max(current.1)))
        }
        (Some(span), None) | (None, Some(span)) => Some(span),
        (None, None) => None,
    };
    if let Some((start, end)) = span {
        command_queue.push_back(Command::Io(IoCommand::RedrawSelection(start, end)));
    }
}

#[cfg(feature = "clipboard")]
fn copy_selection(p: &PagerState) {
    if let Some(text) = p.selected_text()
        && let Ok(mut clipboard) = arboard::Clipboard::new()
    {
        let _ = clipboard.set_text(text);
    }
}

#[cfg(test)]
#[path = "ev_handler/tests.rs"]
mod tests;

#[cfg(all(test, feature = "search"))]
#[path = "ev_handler/search_tests.rs"]
mod search_tests;
