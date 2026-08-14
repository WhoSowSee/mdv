#![cfg_attr(docsrs, doc(cfg(feature = "search")))]
//! Regex and incremental text search.
//!
//! Incremental search reuses preview results after confirmation. Applications can replace its
//! activation predicate with [`Pager::set_incremental_search_condition`](crate::Pager::set_incremental_search_condition).

#![allow(unused_imports)]
use crate::minus_core::utils::{LinesRowMap, display, term};
use crate::screen::Screen;
use crate::{LineNumbers, PagerState};
use crate::{error::MinusError, input::HashedEventRegister, minus_core::utils, screen};
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Attribute,
    terminal::{Clear, ClearType},
};
use regex::Regex;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    io::Write,
    sync::LazyLock,
    time::Duration,
};

use std::collections::hash_map::RandomState;

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
    pub cursor_position: u16,
    /// Active search direction.
    pub search_mode: SearchMode,
    /// Character position where each word starts in the query.
    pub word_index: Vec<u16>,
    /// Search marker selected by [`SearchMode`].
    pub search_char: char,
    pub prompt: String,
    pub rows: u16,
    /// Terminal width in columns.
    pub cols: u16,
    /// Incremental-search state, when available.
    pub incremental_search_options: Option<IncrementalSearchOpts<'a>>,
    compiled_regex: Option<Regex>,
    query_selected: bool,
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
        }
    }
}

impl IncrementalSearchOpts<'_> {
    const fn line_number_digits(&self) -> usize {
        utils::digits(self.screen.line_count())
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
            query_selected,
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
        prompt_width.saturating_add(query_width)
    }
}

fn query_end_position(query: &str) -> u16 {
    query.chars().count().saturating_add(1).try_into().unwrap()
}

fn byte_index_at_character_position(query: &str, position: u16) -> usize {
    query
        .char_indices()
        .nth(usize::from(position.saturating_sub(1)))
        .map_or(query.len(), |(index, _)| index)
}

fn character_position_at_byte_index(query: &str, byte_index: usize) -> u16 {
    query[..byte_index]
        .chars()
        .count()
        .saturating_add(1)
        .try_into()
        .unwrap()
}

fn word_start_positions(query: &str) -> Vec<u16> {
    WORD.find_iter(query)
        .map(|word| character_position_at_byte_index(query, word.start()))
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
}

fn line_matches_query(line: &str, query: &Regex) -> bool {
    let stripped = ANSI_REGEX.replace_all(line, "");
    query.is_match(stripped.as_ref())
}

fn preview_line<'a>(
    iso: &IncrementalSearchOpts<'a>,
    query: &Regex,
    line_idx: usize,
    line: &'a str,
    visible_lines: &mut Vec<Cow<'a, str>>,
    upper_mark: &mut Option<usize>,
    wrapped: bool,
) {
    if upper_mark.is_none() && !line_matches_query(line, query) {
        return;
    }

    let row_start = *iso.lines_to_row_map.get(line_idx).unwrap_or(&0);
    let mut match_row_idx = None;
    let formatted_rows = screen::format_line(
        line,
        iso.line_number_digits(),
        line_idx,
        iso.line_numbers,
        iso.cols,
        iso.screen.line_wrapping,
    );

    let mut formatted_rows = screen::format_search_rows(formatted_rows, Some(query))
        .enumerate()
        .map(|(i, (sfr, is_match))| {
            if is_match {
                if wrapped || row_start + i >= iso.initial_upper_mark {
                    match_row_idx = Some(row_start + i);
                }
                Cow::Owned(sfr.to_string())
            } else {
                iso.screen.formatted_lines.get(row_start + i).map_or_else(
                    || Cow::Owned(sfr.to_string()),
                    |s| Cow::Borrowed(s.as_str()),
                )
            }
        })
        .collect::<Vec<Cow<str>>>();

    if upper_mark.is_none() {
        if match_row_idx.is_none() {
            return;
        }
        let match_row_idx = match_row_idx.unwrap();
        let skip_rows = match_row_idx.saturating_sub(row_start);
        *upper_mark = Some(match_row_idx);
        visible_lines.extend(formatted_rows.drain(skip_rows..));
    } else {
        visible_lines.append(&mut formatted_rows);
    }

    if visible_lines.len() >= iso.writable_rows {
        visible_lines.truncate(iso.writable_rows);
    }
}

fn incremental_preview<'a>(
    iso: &IncrementalSearchOpts<'a>,
    query: &'a Regex,
) -> Option<Vec<Cow<'a, str>>> {
    if iso.writable_rows == 0 {
        return None;
    }

    let start_line_idx = iso
        .lines_to_row_map
        .row_to_line(iso.initial_upper_mark)?
        .saturating_sub(1);

    let mut visible_lines: Vec<Cow<str>> = Vec::with_capacity(iso.writable_rows);
    let mut upper_mark = None;

    for (line_idx, line) in iso
        .screen
        .orig_text
        .lines()
        .enumerate()
        .skip(start_line_idx)
    {
        preview_line(
            iso,
            query,
            line_idx,
            line,
            &mut visible_lines,
            &mut upper_mark,
            false,
        );
        if visible_lines.len() >= iso.writable_rows {
            break;
        }
    }

    // Backfill near-EOF matches so the preview still occupies a full viewport.
    if let Some(um) = upper_mark
        && visible_lines.len() < iso.writable_rows
    {
        let start = iso
            .screen
            .formatted_lines_count()
            .saturating_sub(iso.writable_rows);
        let to_insert = um.saturating_sub(start);
        let shift = visible_lines.len();

        visible_lines.extend(
            iso.screen
                .formatted_lines
                .iter()
                .skip(start)
                .take(to_insert)
                .map(Into::into),
        );
        visible_lines.rotate_left(shift);
    }

    if upper_mark.is_none() {
        for (line_idx, line) in iso
            .screen
            .orig_text
            .lines()
            .enumerate()
            .take(start_line_idx)
        {
            preview_line(
                iso,
                query,
                line_idx,
                line,
                &mut visible_lines,
                &mut upper_mark,
                true,
            );
            if visible_lines.len() >= iso.writable_rows {
                break;
            }
        }
    }

    if upper_mark.is_some() {
        Some(visible_lines)
    } else {
        None
    }
}

fn run_incremental_search<'a, F, O>(
    out: &mut O,
    so: &'a SearchOpts<'a>,
    incremental_search_condition: F,
) -> crate::Result<()>
where
    O: Write,
    F: Fn(&'a SearchOpts) -> bool,
{
    let Some(iso) = so.incremental_search_options.as_ref() else {
        return Ok(());
    };
    let screen = iso.screen;
    let line_numbers = iso.line_numbers;
    let initial_upper_mark = iso.initial_upper_mark;
    let initial_left_mark = iso.initial_left_mark;

    let should_proceed = so.compiled_regex.is_some() && incremental_search_condition(so);

    // Failed or disabled previews restore the exact pre-search viewport.
    let reset_screen = |out: &mut O, so: &SearchOpts<'_>| -> crate::Result {
        display::write_text_checked(
            out,
            &screen.formatted_lines,
            initial_upper_mark,
            so.rows.into(),
            so.cols.into(),
            screen.line_wrapping,
            initial_left_mark,
            line_numbers,
            screen.line_count(),
        )?;
        Ok(())
    };

    if !should_proceed {
        reset_screen(out, so)?;
        return Ok(());
    }

    let query = so.compiled_regex.as_ref().unwrap();

    let Some(visible_lines) = incremental_preview(iso, query) else {
        reset_screen(out, so)?;
        return Ok(());
    };

    display::write_text_checked(
        out,
        &visible_lines,
        0,
        so.rows.into(),
        so.cols.into(),
        iso.screen.line_wrapping,
        iso.initial_left_mark,
        iso.line_numbers,
        iso.screen.line_count(),
    )?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn handle_key_press<O, F>(
    out: &mut O,
    so: &mut SearchOpts<'_>,
    incremental_search_condition: F,
) -> crate::Result
where
    O: Write,
    F: Fn(&SearchOpts<'_>) -> bool,
{
    const FIRST_AVAILABLE_COLUMN: u16 = 1;
    let last_available_column = query_end_position(&so.string);

    if so.ev.is_none() {
        return Ok(());
    }

    let refresh_display = |out: &mut O, so: &mut SearchOpts<'_>| -> Result<(), MinusError> {
        so.compiled_regex = if so.string.is_empty() {
            None
        } else {
            Regex::new(&so.string).ok()
        };

        run_incremental_search(out, so, incremental_search_condition)?;

        term::move_cursor(out, 0, so.rows, false)?;
        write!(out, "\r{}{}", Clear(ClearType::CurrentLine), so.prompt)?;
        write_search_query(out, so)?;
        Ok(())
    };
    match so.ev.as_ref().unwrap() {
        Event::Key(KeyEvent { kind, .. }) if *kind != KeyEventKind::Press => (),
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            so.input_status = InputStatus::Cancelled;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if !clear_selected_query(so) {
                if so.cursor_position == FIRST_AVAILABLE_COLUMN {
                    return Ok(());
                }
                so.cursor_position = so.cursor_position.saturating_sub(1);
                let byte_index = byte_index_at_character_position(&so.string, so.cursor_position);
                so.string.remove(byte_index);
            }
            so.word_index = word_start_positions(&so.string);
            refresh_display(out, so)?;
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, false)?;
            out.flush()?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if !clear_selected_query(so) {
                if so.cursor_position >= last_available_column {
                    return Ok(());
                }
                let byte_index = byte_index_at_character_position(&so.string, so.cursor_position);
                so.string.remove(byte_index);
            }
            so.word_index = word_start_positions(&so.string);
            refresh_display(out, so)?;
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, false)?;
            out.flush()?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            so.input_status = InputStatus::Confirmed;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, FIRST_AVAILABLE_COLUMN)? {
                return Ok(());
            }
            if so.cursor_position == FIRST_AVAILABLE_COLUMN {
                return Ok(());
            }
            so.cursor_position = so.cursor_position.saturating_sub(1);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if collapse_query_selection(out, so, FIRST_AVAILABLE_COLUMN)? {
                return Ok(());
            }
            so.cursor_position = *so
                .word_index
                .iter()
                .rfind(|c| c < &&so.cursor_position)
                .unwrap_or(&FIRST_AVAILABLE_COLUMN);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, last_available_column)? {
                return Ok(());
            }
            if so.cursor_position >= last_available_column {
                return Ok(());
            }
            so.cursor_position = so.cursor_position.saturating_add(1);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if collapse_query_selection(out, so, last_available_column)? {
                return Ok(());
            }
            so.cursor_position = *so
                .word_index
                .iter()
                .find(|c| c > &&so.cursor_position)
                .unwrap_or(&last_available_column);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, FIRST_AVAILABLE_COLUMN)? {
                return Ok(());
            }
            so.cursor_position = 1;
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if collapse_query_selection(out, so, last_available_column)? {
                return Ok(());
            }
            so.cursor_position = query_end_position(&so.string);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, true)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        }) if modifiers.is_empty() || *modifiers == KeyModifiers::SHIFT => {
            let c = *c;
            clear_selected_query(so);
            let byte_index = byte_index_at_character_position(&so.string, so.cursor_position);
            so.string.insert(byte_index, c);
            so.word_index = word_start_positions(&so.string);
            refresh_display(out, so)?;
            so.cursor_position = so.cursor_position.saturating_add(1);
            term::move_cursor(out, so.terminal_cursor_column(), so.rows, false)?;
            out.flush()?;
        }
        _ => return Ok(()),
    }
    Ok(())
}

fn write_search_query(
    out: &mut impl std::io::Write,
    search_opts: &SearchOpts<'_>,
) -> Result<(), MinusError> {
    if search_opts.query_selected {
        write!(out, "{}{}{}", *INVERT, search_opts.string, *NORMAL)?;
    } else {
        write!(out, "{}", search_opts.string)?;
    }
    Ok(())
}

fn write_search_input(
    out: &mut impl std::io::Write,
    search_opts: &SearchOpts<'_>,
) -> Result<(), MinusError> {
    term::move_cursor(out, 0, search_opts.rows, false)?;
    write!(
        out,
        "{}{}",
        Clear(ClearType::CurrentLine),
        search_opts.prompt
    )?;
    write_search_query(out, search_opts)?;
    write!(out, "{}", cursor::Show)?;
    term::move_cursor(
        out,
        search_opts.terminal_cursor_column(),
        search_opts.rows,
        false,
    )?;
    out.flush()?;
    Ok(())
}

fn clear_selected_query(search_opts: &mut SearchOpts<'_>) -> bool {
    if !search_opts.query_selected {
        return false;
    }
    search_opts.string.clear();
    search_opts.cursor_position = 1;
    search_opts.query_selected = false;
    true
}

fn collapse_query_selection(
    out: &mut impl std::io::Write,
    search_opts: &mut SearchOpts<'_>,
    cursor_position: u16,
) -> Result<bool, MinusError> {
    if !search_opts.query_selected {
        return Ok(false);
    }
    search_opts.query_selected = false;
    search_opts.cursor_position = cursor_position;
    write_search_input(out, search_opts)?;
    Ok(true)
}

#[cfg(feature = "search")]
pub(crate) fn fetch_input(
    out: &mut impl std::io::Write,
    ps: &PagerState,
) -> Result<FetchInputResult, MinusError> {
    let mut search_opts = SearchOpts::from(ps);

    write_search_input(out, &search_opts)?;

    loop {
        if event::poll(Duration::from_millis(100)).map_err(|e| MinusError::HandleEvent(e.into()))? {
            let ev = event::read().map_err(|e| MinusError::HandleEvent(e.into()))?;
            search_opts.ev = Some(ev);
            handle_key_press(
                out,
                &mut search_opts,
                &ps.search_state.incremental_search_condition,
            )?;
            search_opts.ev = None;
        }
        if search_opts.input_status.done() {
            break;
        }
    }
    term::move_cursor(out, 0, search_opts.rows, false)?;
    write!(out, "{}{}", Clear(ClearType::CurrentLine), cursor::Hide)?;
    out.flush()?;

    Ok(FetchInputResult {
        string: search_opts.string,
        compiled_regex: search_opts.compiled_regex,
        input_status: search_opts.input_status,
    })
}

pub(crate) fn highlight_matches_args<'a, 'b>(
    line: &'a str,
    query: &'b Regex,
    accurate: bool,
) -> HighlightMatchesArgs<'a, 'b> {
    let stripped_str = ANSI_REGEX.replace_all(line, "");
    let is_match = query.is_match(&stripped_str);
    HighlightMatchesArgs {
        line,
        query,
        accurate,
        is_match,
    }
}

fn highlight_line_matches_ansi(line: &str, query: &regex::Regex, accurate: bool) -> String {
    let stripped_str = ANSI_REGEX.replace_all(line, "");

    if !query.is_match(&stripped_str) {
        return line.to_string();
    }

    let mut sum_width = 0;

    // Original ANSI escapes are tracked by offsets in the stripped string.
    let escapes = ANSI_REGEX
        .find_iter(line)
        .map(|escape| {
            let start = escape.start();
            let as_str = escape.as_str();
            let ret = (start - sum_width, as_str);
            sum_width += as_str.len();
            ret
        })
        .collect::<Vec<_>>();

    let matches = query
        .find_iter(&stripped_str)
        .flat_map(|c| [c.start(), c.end()])
        .collect::<Vec<_>>();

    let mut inverted = query
        .replace_all(&stripped_str, |caps: &regex::Captures| {
            format!("{}{}{}", *INVERT, &caps[0], *NORMAL)
        })
        .to_string();

    let mut inserted_escs_len = 0;
    for esc in escapes {
        let match_count = matches.iter().take_while(|m| **m <= esc.0).count();
        let num_invert = match_count / 2;
        let num_normal = match_count - num_invert;

        // Approximate mode moves escapes inside a match behind that match's reset sequence.
        let mut pos = if !accurate && match_count % 2 == 1 {
            // An odd boundary count guarantees the matching end boundary exists.
            matches.get(match_count).unwrap()
                + NORMAL.len()
                + inserted_escs_len
                + (num_invert * INVERT.len())
                + (num_normal * NORMAL.len())
        } else {
            esc.0 + inserted_escs_len + (num_invert * INVERT.len()) + (num_normal * NORMAL.len())
        };

        if match_count % 2 == 1 {
            pos = pos.saturating_sub(1);
        }

        inverted.insert_str(pos, esc.1);

        inserted_escs_len += esc.1.len();
    }

    inverted
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn highlight_line_matches(
    line: &str,
    query: &regex::Regex,
    accurate: bool,
) -> (String, bool) {
    let highlighted = highlight_matches_args(line, query, accurate);
    (highlighted.to_string(), highlighted.is_match)
}

pub(crate) struct HighlightMatchesArgs<'a, 'b> {
    line: &'a str,
    query: &'b Regex,
    accurate: bool,
    is_match: bool,
}

impl fmt::Display for HighlightMatchesArgs<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_match {
            return f.write_str(self.line);
        }

        if !ANSI_REGEX.is_match(self.line) {
            let mut last = 0;
            for matched in self.query.find_iter(self.line) {
                f.write_str(&self.line[last..matched.start()])?;
                write!(f, "{}{}{}", *INVERT, matched.as_str(), *NORMAL)?;
                last = matched.end();
            }
            return f.write_str(&self.line[last..]);
        }

        f.write_str(&highlight_line_matches_ansi(
            self.line,
            self.query,
            self.accurate,
        ))
    }
}

// jump == 0 includes upper_mark; larger jumps start after it and wrap at the end.
#[must_use]
pub(crate) fn next_nth_match(
    search_idx: &BTreeSet<usize>,
    upper_mark: usize,
    jump: usize,
) -> Option<usize> {
    if search_idx.is_empty() {
        return None;
    }

    let nearest_idx = search_idx.iter().position(|i| {
        if jump == 0 {
            *i >= upper_mark
        } else {
            *i > upper_mark
        }
    });

    let start_idx = nearest_idx.unwrap_or(0);
    let position_of_next_match = if jump == 0 {
        start_idx
    } else {
        start_idx.saturating_add(jump - 1) % search_idx.len()
    };

    Some(position_of_next_match)
}

#[cfg(test)]
mod tests {
    mod input_handling {
        use crate::{
            SearchMode,
            search::{InputStatus, SearchOpts, handle_key_press, write_search_input},
        };
        use crossterm::{
            cursor::MoveTo,
            event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
            style::Attribute,
            terminal::{Clear, ClearType},
        };
        use std::{convert::TryInto, io::Write};

        fn new_search_opts(sm: SearchMode) -> SearchOpts<'static> {
            let search_char = match sm {
                SearchMode::Forward => '/',
                SearchMode::Reverse => '?',
                SearchMode::Unknown => unreachable!(),
            };

            SearchOpts {
                ev: None,
                string: String::with_capacity(200),
                input_status: InputStatus::Active,
                cursor_position: 1,
                word_index: Vec::with_capacity(200),
                prompt: search_char.to_string(),
                search_char,
                rows: 25,
                cols: 100,
                incremental_search_options: None,
                compiled_regex: None,
                query_selected: false,
                search_mode: sm,
            }
        }

        const fn make_event_from_keycode(kc: KeyCode) -> Event {
            Event::Key(KeyEvent {
                code: kc,
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                state: KeyEventState::NONE,
            })
        }

        fn pretest_setup_forward_search() -> (SearchOpts<'static>, Vec<u8>, u16, &'static str) {
            const QUERY_STRING: &str = "this is@complex-text_search?query"; // length = 33
            #[allow(clippy::cast_possible_truncation)]
            let last_movable_column: u16 = (QUERY_STRING.len() as u16) + 1; // 34

            let mut search_opts = new_search_opts(SearchMode::Forward);
            let mut out = Vec::with_capacity(1500);

            for c in QUERY_STRING.chars() {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            }
            assert_eq!(search_opts.cursor_position, last_movable_column);
            (search_opts, out, last_movable_column, QUERY_STRING)
        }

        #[test]
        fn input_sequential_text() {
            let mut search_opts = new_search_opts(SearchMode::Forward);
            let mut out = Vec::with_capacity(1500);
            for (i, c) in "text search matches".chars().enumerate() {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
                assert_eq!(search_opts.input_status, InputStatus::Active);
                assert_eq!(search_opts.cursor_position as usize, i + 2);
            }
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Enter));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.word_index, vec![1, 5, 6, 12, 13]);
            assert_eq!(&search_opts.string, "text search matches");
            assert_eq!(search_opts.input_status, InputStatus::Confirmed);
        }

        #[test]
        fn shifted_unicode_query_supports_navigation_and_deletion() {
            let mut search_opts = new_search_opts(SearchMode::Forward);
            let mut out = Vec::new();
            search_opts.ev = Some(Event::Key(KeyEvent {
                code: KeyCode::Char('Т'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::SHIFT,
                state: KeyEventState::NONE,
            }));

            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            for c in "екст".chars() {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            }

            assert_eq!(search_opts.string, "Текст");
            assert_eq!(search_opts.cursor_position, 6);

            search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Backspace));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            assert_eq!(search_opts.string, "Тект");
            assert_eq!(search_opts.cursor_position, 4);

            search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Delete));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            assert_eq!(search_opts.string, "ект");
            assert_eq!(search_opts.cursor_position, 1);
        }

        #[test]
        fn custom_prompt_uses_the_status_row_with_a_panel() {
            let mut state = crate::PagerState::new().unwrap();
            state.rows = 10;
            state.search_state.search_mode = SearchMode::Forward;
            state.search_prompt = Some("Find: ".to_string());
            state.prompt_panel = vec![
                crate::PromptLine::plain("help one").unwrap(),
                crate::PromptLine::plain("help two").unwrap(),
            ];
            let mut search_opts = SearchOpts::from(&state);
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Char('x')));
            let mut out = Vec::new();

            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            let rendered = String::from_utf8(out).unwrap();
            assert!(rendered.contains("Find: x"));
            assert!(rendered.contains(&MoveTo(7, search_opts.rows).to_string()));
            assert_eq!(usize::from(search_opts.rows), state.prompt_row());
        }

        #[test]
        fn restored_query_is_selected_and_rendered() {
            let mut state = crate::PagerState::new().unwrap();
            state.search_state.search_mode = SearchMode::Forward;
            state.search_state.last_search_query = "pager".to_string();
            state.search_prompt = Some("Find: ".to_string());
            let mut search_opts = SearchOpts::from(&state);
            let mut out = Vec::new();

            write_search_input(&mut out, &search_opts).unwrap();

            assert_eq!(search_opts.string, "pager");
            assert_eq!(search_opts.cursor_position, 6);
            assert!(search_opts.query_selected);

            search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            assert_eq!(search_opts.string, "pager");
            assert_eq!(search_opts.cursor_position, 1);
            assert!(!search_opts.query_selected);
            let rendered = String::from_utf8(out).unwrap();
            assert!(rendered.contains(&format!(
                "Find: {}pager{}",
                Attribute::Reverse,
                Attribute::NoReverse
            )));
            assert!(rendered.contains(&MoveTo(11, search_opts.rows).to_string()));
        }

        #[test]
        fn deleting_selected_query_returns_an_empty_cancelled_draft() {
            for key_code in [KeyCode::Backspace, KeyCode::Delete] {
                let mut state = crate::PagerState::new().unwrap();
                state.search_state.search_mode = SearchMode::Forward;
                state.search_state.last_search_query = "pager".to_string();
                let mut search_opts = SearchOpts::from(&state);
                let mut out = Vec::new();

                search_opts.ev = Some(make_event_from_keycode(key_code));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

                assert!(search_opts.string.is_empty());
                assert_eq!(search_opts.cursor_position, 1);
                assert!(search_opts.compiled_regex.is_none());
                assert_eq!(search_opts.input_status, InputStatus::Active);

                search_opts.ev = Some(make_event_from_keycode(KeyCode::Esc));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

                assert!(search_opts.string.is_empty());
                assert_eq!(search_opts.input_status, InputStatus::Cancelled);
            }
        }

        #[test]
        fn input_complex_sequential_text() {
            let mut search_opts = new_search_opts(SearchMode::Forward);
            let mut out = Vec::with_capacity(1500);
            for (i, c) in "this is@complex-text_search?query".chars().enumerate() {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
                assert_eq!(search_opts.input_status, InputStatus::Active);
                assert_eq!(search_opts.cursor_position as usize, i + 2);
            }
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Enter));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.word_index, vec![1, 5, 6, 8, 9, 16, 17, 28, 29]);
            assert_eq!(&search_opts.string, "this is@complex-text_search?query");
            assert_eq!(search_opts.input_status, InputStatus::Confirmed);
        }

        #[test]
        fn home_end_keys() {
            let (mut search_opts, mut out, last_movable_column, _) = pretest_setup_forward_search();

            search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.cursor_position as usize, 1);

            search_opts.ev = Some(make_event_from_keycode(KeyCode::End));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.cursor_position, last_movable_column);
        }

        #[test]
        fn basic_left_arrow_movement() {
            const FIRST_MOVABLE_COLUMN: u16 = 1;
            let (mut search_opts, mut out, last_movable_column, _) = pretest_setup_forward_search();
            let query_string_length = last_movable_column - 1;

            for i in (FIRST_MOVABLE_COLUMN..=query_string_length).rev() {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
                assert_eq!(search_opts.cursor_position, i);
            }
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Left));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.cursor_position, FIRST_MOVABLE_COLUMN);
        }

        #[test]
        fn basic_right_arrow_movement() {
            let (mut search_opts, mut out, last_movable_column, _) = pretest_setup_forward_search();
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            for i in 2..=last_movable_column {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Right));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
                assert_eq!(search_opts.cursor_position, i);
            }
            search_opts.ev = Some(make_event_from_keycode(KeyCode::Right));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.cursor_position, last_movable_column);
        }

        #[test]
        fn right_jump_by_word() {
            const JUMP_COLUMNS: [u16; 10] = [1, 5, 6, 8, 9, 16, 17, 28, 29, LAST_MOVABLE_COLUMN];
            let (mut search_opts, mut out, _last_movable_column, _) =
                pretest_setup_forward_search();
            #[allow(clippy::items_after_statements)]
            const LAST_MOVABLE_COLUMN: u16 = 34;

            search_opts.ev = Some(make_event_from_keycode(KeyCode::Home));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            let ev = Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                state: KeyEventState::NONE,
            });

            for i in &JUMP_COLUMNS[1..] {
                search_opts.ev = Some(ev.clone());
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
                assert_eq!(search_opts.cursor_position, *i);
            }
            search_opts.ev = Some(ev);
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.cursor_position, LAST_MOVABLE_COLUMN);
        }

        #[test]
        fn left_jump_by_word() {
            const JUMP_COLUMNS: [u16; 10] = [1, 5, 6, 8, 9, 16, 17, 28, 29, LAST_MOVABLE_COLUMN];
            let (mut search_opts, mut out, _last_movable_column, _) =
                pretest_setup_forward_search();
            #[allow(clippy::items_after_statements)]
            const LAST_MOVABLE_COLUMN: u16 = 34;

            let ev = Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                state: KeyEventState::NONE,
            });

            for i in (JUMP_COLUMNS[..(JUMP_COLUMNS.len() - 1)]).iter().rev() {
                search_opts.ev = Some(ev.clone());
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
                assert_eq!(search_opts.cursor_position, *i);
            }
            search_opts.ev = Some(ev);
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            assert_eq!(search_opts.cursor_position, JUMP_COLUMNS[0]);
        }

        #[test]
        fn escape_cancels_a_non_empty_search_without_clearing_it() {
            let (mut search_opts, mut out, _, query) = pretest_setup_forward_search();

            search_opts.ev = Some(make_event_from_keycode(KeyCode::Esc));
            handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();

            assert_eq!(search_opts.string, query);
            assert_eq!(search_opts.input_status, InputStatus::Cancelled);
        }

        #[test]
        fn forward_sequential_text_input_screen_data() {
            let (search_opts, out, _last_movable_column, query_string) =
                pretest_setup_forward_search();

            let mut result_out = Vec::with_capacity(1500);

            let mut string = String::with_capacity(query_string.len());
            let mut cursor_position: u16 = 1;
            for c in query_string.chars() {
                string.push(c);
                cursor_position = cursor_position.saturating_add(1);
                write!(
                    result_out,
                    "{move_to_prompt}\r{clear_line}/{string}{move_to_position}",
                    move_to_prompt = MoveTo(0, search_opts.rows),
                    clear_line = Clear(ClearType::CurrentLine),
                    move_to_position = MoveTo(cursor_position, search_opts.rows),
                )
                .unwrap();
            }
            assert_eq!(out, result_out);
        }

        #[test]
        fn backward_sequential_text_input_screen_data() {
            const QUERY_STRING: &str = "this is@complex-text_search?query"; // length = 33
            #[allow(clippy::cast_possible_truncation)]
            const LAST_MOVABLE_COLUMN: u16 = (QUERY_STRING.len() as u16) + 1; // 34

            let mut search_opts = new_search_opts(SearchMode::Reverse);
            let mut out = Vec::with_capacity(1500);

            for c in QUERY_STRING.chars() {
                search_opts.ev = Some(make_event_from_keycode(KeyCode::Char(c)));
                handle_key_press(&mut out, &mut search_opts, |_| false).unwrap();
            }
            assert_eq!(search_opts.cursor_position, LAST_MOVABLE_COLUMN);

            let mut result_out = Vec::with_capacity(1500);

            let mut string = String::with_capacity(QUERY_STRING.len());
            let mut cursor_position: u16 = 1;
            for c in QUERY_STRING.chars() {
                string.push(c);
                cursor_position = cursor_position.saturating_add(1);
                write!(
                    result_out,
                    "{move_to_prompt}\r{clear_line}?{string}{move_to_position}",
                    move_to_prompt = MoveTo(0, search_opts.rows),
                    clear_line = Clear(ClearType::CurrentLine),
                    move_to_position = MoveTo(cursor_position, search_opts.rows),
                )
                .unwrap();
            }
            assert_eq!(out, result_out);
        }
    }

    #[test]
    fn test_next_match() {
        let search_idx = std::collections::BTreeSet::from([2, 10, 15, 17, 50]);
        let mut upper_mark = 0;
        let mut search_mark;
        for (i, v) in search_idx.iter().enumerate() {
            search_mark = super::next_nth_match(&search_idx, upper_mark, 1);
            assert_eq!(search_mark, Some(i));
            let next_upper_mark = *search_idx.iter().nth(search_mark.unwrap()).unwrap();
            assert_eq!(next_upper_mark, *v);
            upper_mark = next_upper_mark;
        }
    }

    #[allow(clippy::trivial_regex)]
    mod highlighting {
        use std::collections::BTreeSet;

        use crate::PagerState;
        use crate::search::{INVERT, NORMAL, highlight_line_matches, next_nth_match};
        use crossterm::style::Attribute;
        use regex::Regex;

        const ESC: &str = "\x1b[34m";
        const NONE: &str = "\x1b[0m";

        mod consistent {
            use super::*;

            #[test]
            fn test_highlight_matches() {
                let line = "Integer placerat tristique nisl. placerat non mollis, magna orci dolor, placerat at vulputate neque nulla lacinia eros.".to_string();
                let pat = Regex::new(r"\W\w+t\W").unwrap();
                let result = format!(
                    "Integer{inverse} placerat {noinverse}tristique nisl.\
{inverse} placerat {noinverse}non mollis, magna orci dolor,\
{inverse} placerat {noinverse}at vulputate neque nulla lacinia \
eros.",
                    inverse = Attribute::Reverse,
                    noinverse = Attribute::NoReverse
                );

                assert_eq!(highlight_line_matches(&line, &pat, false).0, result);
            }

            #[test]
            fn no_match() {
                let orig = "no match";
                let res = highlight_line_matches(orig, &Regex::new("test").unwrap(), false);
                assert_eq!(res.0, orig.to_string());
            }

            #[test]
            fn single_match_no_esc() {
                let res =
                    highlight_line_matches("this is a test", &Regex::new(" a ").unwrap(), false);
                assert_eq!(res.0, format!("this is{} a {}test", *INVERT, *NORMAL));
            }

            #[test]
            fn multi_match_no_esc() {
                let res = highlight_line_matches(
                    "test another test",
                    &Regex::new("test").unwrap(),
                    false,
                );
                assert_eq!(
                    res.0,
                    format!("{i}test{n} another {i}test{n}", i = *INVERT, n = *NORMAL)
                );
            }

            #[test]
            fn esc_pair_outside_match() {
                let res = highlight_line_matches(
                    &format!("{ESC}color{NONE} and test"),
                    &Regex::new("test").unwrap(),
                    false,
                );
                assert_eq!(
                    res.0,
                    format!("{}color{} and {}test{}", ESC, NONE, *INVERT, *NORMAL)
                );
            }

            #[test]
            fn esc_pair_end_in_match() {
                let orig = format!("this {ESC}is a te{NONE}st");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
                assert_eq!(
                    res.0,
                    format!("this {}is a {}test{}{}", ESC, *INVERT, *NORMAL, NONE)
                );
            }

            #[test]
            fn esc_pair_start_in_match() {
                let orig = format!("this is a te{ESC}st again{NONE}");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
                assert_eq!(
                    res.0,
                    format!("this is a {}test{}{ESC} again{}", *INVERT, *NORMAL, NONE)
                );
            }

            #[test]
            fn esc_pair_around_match() {
                let orig = format!("this is {ESC}a test again{NONE}");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
                assert_eq!(
                    res.0,
                    format!("this is {}a {}test{} again{}", ESC, *INVERT, *NORMAL, NONE)
                );
            }

            #[test]
            fn esc_pair_within_match() {
                let orig = format!("this is a t{ESC}es{NONE}t again");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
                assert_eq!(
                    res.0,
                    format!("this is a {}test{}{ESC}{NONE} again", *INVERT, *NORMAL)
                );
            }

            #[test]
            fn multi_escape_match() {
                let orig = format!("this {ESC}is a te{NONE}st again {ESC}yeah{NONE} test");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
                assert_eq!(
                    res.0,
                    format!(
                        "this {e}is a {i}test{n}{nn} again {e}yeah{nn} {i}test{n}",
                        e = ESC,
                        i = *INVERT,
                        n = *NORMAL,
                        nn = NONE
                    )
                );
            }
        }
        mod accurate {
            use super::*;
            #[test]
            fn correct_ascii_sequence_placement() {
                let orig = format!(
                    "{ESC}test{NONE} this {ESC}is a te{NONE}st again {ESC}yeah{NONE} test",
                );

                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
                assert_eq!(
                    res.0,
                    format!(
                        "{i}{e}test{n}{nn} this {e}is a {i}te{NONE}st{n} again {e}yeah{nn} {i}test{n}",
                        e = ESC,
                        i = *INVERT,
                        n = *NORMAL,
                        nn = NONE
                    )
                );
            }

            #[test]
            fn esc_pair_outside_match() {
                let res = highlight_line_matches(
                    &format!("{ESC}color{NONE} and test"),
                    &Regex::new("test").unwrap(),
                    true,
                );
                assert_eq!(
                    res.0,
                    format!("{}color{} and {}test{}", ESC, NONE, *INVERT, *NORMAL)
                );
            }

            #[test]
            fn esc_pair_end_in_match() {
                let orig = format!("this {ESC}is a te{NONE}st");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
                assert_eq!(
                    res.0,
                    format!("this {ESC}is a {}te{NONE}st{}", *INVERT, *NORMAL)
                );
            }

            #[test]
            fn esc_pair_start_in_match() {
                let orig = format!("this is a te{ESC}st again{NONE}");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
                assert_eq!(
                    res.0,
                    format!("this is a {}te{ESC}st{} again{NONE}", *INVERT, *NORMAL)
                );
            }

            #[test]
            fn esc_pair_around_match() {
                let orig = format!("this is {ESC}a test again{NONE}");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
                assert_eq!(
                    res.0,
                    format!("this is {ESC}a {}test{} again{NONE}", *INVERT, *NORMAL)
                );
            }

            #[test]
            fn esc_pair_within_match() {
                let orig = format!("this is a t{ESC}es{NONE}t again");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
                assert_eq!(
                    res.0,
                    format!("this is a {}t{ESC}es{NONE}t{} again", *INVERT, *NORMAL)
                );
            }

            #[test]
            fn multi_escape_match() {
                let orig = format!("this {ESC}is a te{NONE}st again {ESC}yeah{NONE} test");
                let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
                assert_eq!(
                    res.0,
                    format!(
                        "this {e}is a {i}te{nn}st{n} again {e}yeah{nn} {i}test{n}",
                        e = ESC,
                        i = *INVERT,
                        n = *NORMAL,
                        nn = NONE
                    )
                );
            }
        }
    }
}
