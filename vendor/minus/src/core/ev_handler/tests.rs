use super::super::commands::{Command, IoCommand};
use super::handle_event;
use crate::{
    Pager, PagerState, PromptLine, PromptSpan, PromptStyle, input::InputEvent,
    minus_core::CommandQueue, state::Selection,
};
use std::fmt::Write;
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(feature = "search")]
use super::search_actions::set_search_position;
#[cfg(feature = "search")]
use super::{NOT_FOUND_MESSAGE_DURATION, apply_line_navigation_result, apply_search_result};
#[cfg(feature = "search")]
use crate::LineNavigation;

const TEST_STR: &str = "This is some sample text";

#[cfg(feature = "search")]
#[allow(clippy::trivial_regex)]
fn pager_with_active_search() -> PagerState {
    let mut ps = PagerState::new().unwrap();
    ps.screen.orig_text = "pager\nother".to_string();
    ps.search_mode = crate::search::SearchMode::Forward;
    ps.search_state.search_mode = crate::search::SearchMode::Forward;
    ps.search_state.last_search_query = "pager".to_string();
    ps.search_state.search_term = Some(regex::Regex::new("pager").unwrap());
    ps.reformat_display().unwrap();
    ps
}

#[path = "tests/data.rs"]
mod data;
#[path = "tests/prompt.rs"]
mod prompt;
#[path = "tests/search.rs"]
mod search;
#[path = "tests/selection.rs"]
mod selection;
