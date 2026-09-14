use crate::{PagerState, error::MinusError, minus_core::utils::term};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{Clear, ClearType},
};
use std::io::Write;
use std::time::Duration;

/// Source-line maps and optional alternate source-numbered pager content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineNavigation {
    /// Source line for each logical line in the normal pager content.
    display_source_lines: Vec<Option<usize>>,
    source_view: SourceView,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SourceView {
    Current,
    Alternate {
        text: String,
        source_lines: Vec<Option<usize>>,
    },
}

pub struct LineNavigationSession {
    original_upper_mark: usize,
    original_left_mark: usize,
    target: Option<usize>,
    prompt_origin: Option<usize>,
}

impl LineNavigation {
    /// Creates line-navigation data with alternate source-numbered content.
    ///
    /// Both maps contain one entry per logical line. `None` marks lines that do not
    /// originate from a source line, such as inserted spacing.
    #[must_use]
    pub fn new(
        source_text: impl Into<String>,
        display_source_lines: Vec<Option<usize>>,
        navigation_source_lines: Vec<Option<usize>>,
    ) -> Self {
        Self {
            display_source_lines,
            source_view: SourceView::Alternate {
                text: source_text.into(),
                source_lines: navigation_source_lines,
            },
        }
    }

    /// Creates navigation data when the pager content is already source-numbered.
    #[must_use]
    pub const fn from_current(source_lines: Vec<Option<usize>>) -> Self {
        Self {
            display_source_lines: source_lines,
            source_view: SourceView::Current,
        }
    }
}

#[derive(Clone, Copy)]
pub enum LineInputResult {
    Cancelled,
    Confirmed(Option<usize>),
}

impl PagerState {
    pub(crate) fn replace_mapped_text(
        &mut self,
        text: String,
        navigation: Option<LineNavigation>,
    ) -> Result<(), crate::PromptError> {
        let navigation_view = self.line_navigation_session.is_some();
        let anchor = self.source_line_map(navigation_view).and_then(|map| {
            let index = self.lines_to_row_map.row_to_line(self.upper_mark)?;
            (0..=index).rev().find_map(|i| {
                Some((
                    map.get(i).copied().flatten()?,
                    self.upper_mark
                        .saturating_sub(*self.lines_to_row_map.get(i)?),
                ))
            })
        });
        self.line_navigation_session = None;
        self.line_navigation = navigation;
        self.screen.orig_text = text;
        self.screen.line_count = self.screen.orig_text.lines().count();
        self.clear_selection();
        self.left_mark = 0;
        self.reformat_display()?;
        if let Some((source, offset)) = anchor
            && let Some(row) = self.row_for_source_line(source, false)
        {
            let next = (row + 1..self.screen.formatted_lines.len())
                .find(|&i| {
                    self.source_line_at_row(i, false)
                        .is_some_and(|line| line != source)
                })
                .unwrap_or(self.screen.formatted_lines.len());
            self.upper_mark = row.saturating_add(offset.min(next.saturating_sub(row + 1)));
        }
        self.upper_mark = self.upper_mark.min(self.max_upper_mark());
        self.format_prompt()
    }

    /// Returns whether source-line navigation is configured.
    #[must_use]
    pub(crate) const fn line_navigation_available(&self) -> bool {
        if self.line_navigation.is_none() {
            return false;
        }
        match &self.line_navigation_session {
            Some(session) => session.prompt_origin.is_none(),
            None => true,
        }
    }

    /// Returns whether a source-line target is active.
    #[must_use]
    pub(crate) const fn line_navigation_is_active(&self) -> bool {
        match &self.line_navigation_session {
            Some(session) => session.prompt_origin.is_none() && session.target.is_some(),
            None => false,
        }
    }

    #[must_use]
    pub(crate) const fn line_navigation_position(&self) -> Option<usize> {
        match &self.line_navigation_session {
            Some(session) if session.prompt_origin.is_none() => session.target,
            _ => None,
        }
    }

    pub(crate) fn begin_line_navigation(&mut self) -> Result<bool, crate::PromptError> {
        if !self.line_navigation_available() {
            return Ok(false);
        }

        if self.line_navigation_session.is_none() {
            let original_upper_mark = self.upper_mark;
            let original_left_mark = self.left_mark;
            let uses_alternate_view = self.line_navigation.as_ref().is_some_and(|navigation| {
                matches!(navigation.source_view, SourceView::Alternate { .. })
            });
            let visible_source_line = self.source_line_at_row(self.upper_mark, false);
            if uses_alternate_view {
                self.left_mark = 0;
                self.swap_line_navigation_text();
                self.reformat_display()?;
                self.upper_mark = visible_source_line
                    .and_then(|line| self.row_for_source_line(line, true))
                    .unwrap_or(self.upper_mark)
                    .min(self.max_upper_mark());
            }
            self.line_navigation_session = Some(LineNavigationSession {
                original_upper_mark,
                original_left_mark,
                target: None,
                prompt_origin: None,
            });
        }

        if let Some(session) = self.line_navigation_session.as_mut() {
            session.prompt_origin = Some(self.upper_mark);
        }
        self.selection = None;
        self.selection_anchor = None;
        Ok(true)
    }

    pub(crate) fn configure_line_navigation(
        &mut self,
        navigation: Option<LineNavigation>,
    ) -> Result<(), crate::PromptError> {
        self.exit_line_navigation()?;
        self.line_navigation = navigation;
        Ok(())
    }

    pub(crate) fn finish_line_navigation(
        &mut self,
        source_line: Option<usize>,
    ) -> Result<bool, crate::PromptError> {
        let Some(prompt_upper_mark) = self
            .line_navigation_session
            .as_mut()
            .and_then(|session| session.prompt_origin.take())
        else {
            return Ok(false);
        };

        if let Some(source_line) = source_line
            && let Some(target_row) = self.row_for_source_line(source_line, true)
        {
            self.upper_mark = target_row;
            if let Some(session) = self.line_navigation_session.as_mut() {
                session.target = Some(source_line);
            }
            return Ok(true);
        }

        if self
            .line_navigation_session
            .as_ref()
            .is_some_and(|session| session.target.is_some())
        {
            self.upper_mark = prompt_upper_mark;
        } else {
            self.exit_line_navigation()?;
        }
        Ok(false)
    }

    pub(crate) fn exit_line_navigation(&mut self) -> Result<bool, crate::PromptError> {
        let Some(session) = self.line_navigation_session.take() else {
            return Ok(false);
        };
        let visible_source_line = self.source_line_at_row(self.upper_mark, true);

        let uses_alternate_view = self.line_navigation.as_ref().is_some_and(|navigation| {
            matches!(navigation.source_view, SourceView::Alternate { .. })
        });
        if uses_alternate_view {
            self.swap_line_navigation_text();
            self.left_mark = session.original_left_mark;
            self.reformat_display()?;
        }

        let source_line = visible_source_line.or(session.target);
        if let Some(target_row) = source_line.and_then(|line| self.row_for_source_line(line, false))
        {
            self.upper_mark = target_row;
        } else {
            self.upper_mark = session.original_upper_mark.min(self.max_upper_mark());
        }
        Ok(true)
    }

    pub(crate) fn source_line_is_highlighted(&self, absolute_row: usize) -> bool {
        if !self.line_navigation_is_active() {
            return false;
        }
        self.source_line_at_row(absolute_row, true) == self.line_navigation_position()
    }

    fn swap_line_navigation_text(&mut self) {
        let navigation = self
            .line_navigation
            .as_mut()
            .expect("line navigation session requires configured content");
        let SourceView::Alternate { text, .. } = &mut navigation.source_view else {
            unreachable!("swapped line navigation session requires alternate content");
        };
        std::mem::swap(&mut self.screen.orig_text, text);
        self.screen.line_count = self.screen.orig_text.lines().count();
    }

    fn source_line_at_row(&self, row: usize, navigation_view: bool) -> Option<usize> {
        let line_map = self.source_line_map(navigation_view)?;
        let line_index = self.lines_to_row_map.row_to_line(row)?;
        line_map.get(line_index).copied().flatten()
    }

    fn row_for_source_line(&self, source_line: usize, navigation_view: bool) -> Option<usize> {
        let line_map = self.source_line_map(navigation_view)?;
        let line_index = line_map
            .iter()
            .position(|line| *line == Some(source_line))?;
        self.lines_to_row_map.get(line_index).copied()
    }

    fn source_line_map(&self, navigation_view: bool) -> Option<&[Option<usize>]> {
        let navigation = self.line_navigation.as_ref()?;
        match (&navigation.source_view, navigation_view) {
            (SourceView::Alternate { source_lines, .. }, true) => Some(source_lines),
            _ => Some(&navigation.display_source_lines),
        }
    }
}

pub fn fetch_line_number(
    out: &mut impl Write,
    pager: &PagerState,
) -> Result<LineInputResult, MinusError> {
    let mut input = String::new();
    draw_input(out, pager, &input)?;

    let result = loop {
        if !event::poll(Duration::from_millis(100))
            .map_err(|error| MinusError::HandleEvent(error.into()))?
        {
            continue;
        }
        let event = event::read().map_err(|error| MinusError::HandleEvent(error.into()))?;
        let Event::Key(key) = event else {
            continue;
        };
        if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
            continue;
        }
        match key.code {
            KeyCode::Esc => break LineInputResult::Cancelled,
            KeyCode::Enter => {
                let line = input.parse::<usize>().ok().filter(|line| *line > 0);
                break LineInputResult::Confirmed(line);
            }
            KeyCode::Backspace => {
                input.pop();
                draw_input(out, pager, &input)?;
            }
            KeyCode::Char(character) if character.is_ascii_digit() => {
                input.push(character);
                draw_input(out, pager, &input)?;
            }
            _ => {}
        }
    };

    let row = prompt_row(pager)?;
    term::move_cursor(out, 0, row, false)?;
    write!(out, "{}{}", Clear(ClearType::CurrentLine), cursor::Hide)?;
    out.flush()?;
    Ok(result)
}

fn draw_input(out: &mut impl Write, pager: &PagerState, input: &str) -> Result<(), MinusError> {
    let row = prompt_row(pager)?;
    let column = input
        .chars()
        .count()
        .saturating_add(1)
        .try_into()
        .unwrap_or(u16::MAX);
    term::move_cursor(out, 0, row, false)?;
    write!(
        out,
        "{}:{input}{}",
        Clear(ClearType::CurrentLine),
        cursor::Show
    )?;
    term::move_cursor(out, column, row, false)?;
    out.flush().map_err(MinusError::Draw)
}

fn prompt_row(pager: &PagerState) -> Result<u16, MinusError> {
    pager
        .prompt_row()
        .try_into()
        .map_err(|_| MinusError::Conversion)
}

#[cfg(test)]
mod tests;
