#![cfg_attr(docsrs, doc(cfg(feature = "search")))]
//! Regex and incremental text search.
//!
//! Incremental search reuses preview results after confirmation. Applications can replace its
//! activation predicate with [`Pager::set_incremental_search_condition`](crate::Pager::set_incremental_search_condition).

use crate::minus_core::utils::{LinesRowMap, display, term};
use crate::screen::Screen;
use crate::{LineNumbers, PagerState};
use crate::{error::MinusError, minus_core::utils, screen};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Attribute,
    terminal::{Clear, ClearType},
};
use regex::Regex;
use std::borrow::Cow;
#[cfg(test)]
use std::fmt;
use std::{
    convert::{TryFrom, TryInto},
    io::Write,
    sync::LazyLock,
    time::Duration,
};

mod highlight;
#[cfg(test)]
mod highlighting;
mod incremental;
mod input;
#[cfg(test)]
pub(crate) use highlighting::highlight_line_matches;
#[cfg(test)]
use incremental::incremental_preview;
use incremental::run_incremental_search;
pub(crate) use incremental::search_ranges;
pub(crate) use input::fetch_input;
#[cfg(test)]
use input::{handle_key_press, write_search_input, write_search_query};

pub(crate) use highlight::{highlight_line_navigation_target, highlight_search_matches};

static INVERT: LazyLock<String> = LazyLock::new(|| Attribute::Reverse.to_string());
static NORMAL: LazyLock<String> = LazyLock::new(|| Attribute::NoReverse.to_string());
static ANSI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("[\\u001b\\u009b]\\[[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]")
        .unwrap()
});

static WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"([\w_]+)|([-?~@#!$%^&*()-+={}\[\]:;\\|'/?<>.,"]+)|\W"#).unwrap()
});

#[derive(Clone, Copy, Debug, Default, Eq)]
#[cfg_attr(docsrs, doc(cfg(feature = "search")))]
#[allow(clippy::module_name_repetitions)]
/// Search direction.
pub enum SearchMode {
    /// Searches forward from the current page.
    Forward,
    /// Searches backward from the current page.
    Reverse,
    /// No active search.
    #[default]
    Unknown,
}

impl PartialEq for SearchMode {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SearchRange {
    pub start: usize,
    pub end: usize,
}

impl SearchRange {
    pub(crate) const fn after_skipping(self, skipped_chars: usize) -> Option<Self> {
        if self.end <= skipped_chars {
            return None;
        }
        Some(Self {
            start: self.start.saturating_sub(skipped_chars),
            end: self.end.saturating_sub(skipped_chars),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SearchMatch {
    pub row: usize,
    pub range: SearchRange,
}

/// State supplied to the incremental-search activation predicate.
#[allow(clippy::module_name_repetitions)]
pub struct SearchOpts<'a> {
    /// Event currently being processed, if any.
    pub ev: Option<Event>,
    /// Current query.
    pub string: String,
    /// Search-input status.
    pub input_status: InputStatus,
    /// One-based character position within the query.
    pub cursor_position: usize,
    /// Active search direction.
    pub search_mode: SearchMode,
    /// Character position where each word starts in the query.
    pub word_index: Vec<usize>,
    /// Search marker selected by [`SearchMode`].
    pub search_char: char,
    pub prompt: String,
    pub rows: u16,
    /// Terminal width in columns.
    pub cols: u16,
    /// Incremental-search state, when available.
    pub incremental_search_options: Option<IncrementalSearchOpts<'a>>,
    compiled_regex: Option<Regex>,
    preview_upper_mark: Option<usize>,
    query_selected: bool,
    output_styling: bool,
}

/// Pager state captured for incremental-search previews.
pub struct IncrementalSearchOpts<'a> {
    /// Line-number configuration.
    pub line_numbers: LineNumbers,
    /// Vertical offset before search opened.
    pub initial_upper_mark: usize,
    /// Screen being searched.
    pub screen: &'a Screen,
    /// Cached map from logical lines to formatted rows.
    pub lines_to_row_map: &'a LinesRowMap,
    /// Horizontal offset before search opened.
    pub initial_left_mark: usize,
    cols: usize,
    writable_rows: usize,
    output_styling: bool,
    color_depth: crate::ColorDepth,
}

impl<'a> From<&'a PagerState> for IncrementalSearchOpts<'a> {
    fn from(ps: &'a PagerState) -> Self {
        Self {
            line_numbers: ps.line_numbers,
            initial_upper_mark: ps.upper_mark,
            screen: &ps.screen,
            lines_to_row_map: &ps.lines_to_row_map,
            initial_left_mark: ps.left_mark,
            cols: ps.cols,
            writable_rows: ps.content_rows(),
            output_styling: ps.output_styling,
            color_depth: ps.color_depth,
        }
    }
}

impl IncrementalSearchOpts<'_> {
    const fn line_number_digits(&self) -> usize {
        utils::digits(self.screen.line_count())
    }

    const fn content_start_chars(&self) -> usize {
        if self.line_numbers.is_on() {
            self.line_number_digits() + LineNumbers::EXTRA_PADDING + 2
        } else {
            0
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl<'a> From<&'a PagerState> for SearchOpts<'a> {
    fn from(ps: &'a PagerState) -> Self {
        let search_char = if ps.search_state.search_mode == SearchMode::Forward {
            '/'
        } else if ps.search_state.search_mode == SearchMode::Reverse {
            '?'
        } else {
            unreachable!();
        };

        let incremental_search_options = IncrementalSearchOpts::from(ps);
        let prompt = ps
            .search_prompt
            .clone()
            .unwrap_or_else(|| search_char.to_string());
        let string = ps.search_state.last_search_query.clone();
        let cursor_position = query_end_position(&string);
        let word_index = word_start_positions(&string);
        let query_selected = !string.is_empty();

        Self {
            ev: None,
            string,
            input_status: InputStatus::Active,
            cursor_position,
            word_index,
            prompt,
            search_char,
            rows: ps.prompt_row().try_into().unwrap(),
            cols: ps.cols.try_into().unwrap(),
            incremental_search_options: Some(incremental_search_options),
            compiled_regex: None,
            preview_upper_mark: None,
            query_selected,
            output_styling: ps.output_styling,
            search_mode: ps.search_state.search_mode,
        }
    }
}

impl SearchOpts<'_> {
    fn terminal_cursor_column(&self) -> u16 {
        let prompt_width =
            u16::try_from(unicode_width::UnicodeWidthStr::width(self.prompt.as_str()))
                .unwrap_or(u16::MAX);
        let cursor_byte_index =
            byte_index_at_character_position(&self.string, self.cursor_position);
        let query_width = u16::try_from(unicode_width::UnicodeWidthStr::width(
            &self.string[..cursor_byte_index],
        ))
        .unwrap_or(u16::MAX);
        prompt_width
            .saturating_add(query_width)
            .min(self.cols.saturating_sub(1))
    }
}

fn query_end_position(query: &str) -> usize {
    query.chars().count().saturating_add(1)
}

fn byte_index_at_character_position(query: &str, position: usize) -> usize {
    query
        .char_indices()
        .nth(position.saturating_sub(1))
        .map_or(query.len(), |(index, _)| index)
}

fn word_start_positions(query: &str) -> Vec<usize> {
    let mut byte_index = 0;
    let mut position = 1;
    WORD.find_iter(query)
        .map(|word| {
            position += query[byte_index..word.start()].chars().count();
            byte_index = word.start();
            position
        })
        .collect()
}

/// Search-input lifecycle state.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum InputStatus {
    /// Confirmed with Enter.
    Confirmed,
    /// Cancelled with Escape.
    Cancelled,
    /// Accepting input.
    Active,
}

impl InputStatus {
    /// Returns whether input has ended.
    #[must_use]
    pub const fn done(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Confirmed)
    }
}

pub(crate) struct FetchInputResult {
    pub(crate) string: String,
    pub(crate) compiled_regex: Option<Regex>,
    pub(crate) input_status: InputStatus,
    pub(crate) preview_upper_mark: Option<usize>,
}

struct IncrementalPreview<'a> {
    rows: Vec<Cow<'a, str>>,
    upper_mark: usize,
}

#[cfg(test)]
#[path = "search/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "search/incremental_tests.rs"]
mod incremental_tests;
