use crate::input::{
    InputEvent,
    crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
};
use crate::{
    PagerState, PromptAttribute, PromptColor, PromptError, PromptLine, PromptSpan, PromptStyle,
};
use std::io::Write;

#[cfg(test)]
mod tests;

/// A visible outline heading and its source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TocEntry {
    pub title: String,
    pub source_line: usize,
    /// Zero for a numbered section, one for a subsection.
    pub depth: u8,
}

#[derive(Default)]
pub struct TocState {
    visible: bool,
    offset: usize,
    manual_scroll: bool,
    pinned: Option<(usize, usize)>,
    number: Option<u8>,
    hint_dismissed: bool,
}

impl TocState {
    pub(crate) const fn reset_position(&mut self) {
        self.offset = 0;
        self.manual_scroll = false;
        self.pinned = None;
        self.number = None;
    }
}

impl PagerState {
    /// Whether the outline panel is open.
    #[must_use]
    pub const fn toc_visible(&self) -> bool {
        self.toc.visible
    }

    pub(crate) const fn toc_hint_visible(&self) -> bool {
        self.toc.visible && !self.toc.hint_dismissed
    }

    fn toc_entries(&self) -> &[TocEntry] {
        self.line_navigation
            .as_ref()
            .map_or(&[], |nav| nav.toc.as_slice())
    }

    pub(crate) fn toc_width(&self) -> usize {
        self.cols.min(36).min((self.cols / 2).max(1))
    }

    fn toc_target_row(&self, index: usize) -> Option<usize> {
        self.row_for_source_line(
            self.toc_entries().get(index)?.source_line,
            self.line_navigation_session.is_some(),
        )
    }

    fn active_toc(&self) -> usize {
        if let Some((index, mark)) = self.toc.pinned
            && mark == self.upper_mark
            && index < self.toc_entries().len()
        {
            return index;
        }
        if self.max_upper_mark() > 0 && self.upper_mark >= self.max_upper_mark() {
            return self.toc_entries().len().saturating_sub(1);
        }
        (0..self.toc_entries().len())
            .rev()
            .find(|&i| {
                self.toc_target_row(i)
                    .is_some_and(|row| row <= self.upper_mark)
            })
            .unwrap_or(0)
    }

    fn toc_offset(&self) -> usize {
        let height = self.content_rows().saturating_sub(1).max(1);
        let max = self.toc_entries().len().saturating_sub(height);
        if self.toc.manual_scroll {
            self.toc.offset.min(max)
        } else {
            self.active_toc().saturating_sub(height / 2).min(max)
        }
    }

    /// Classifies outline controls without intercepting ordinary document scrolling.
    #[must_use]
    pub fn toc_input(&self, event: &Event) -> Option<InputEvent> {
        match event {
            Event::Key(key)
                if self.toc.visible
                    && key.kind != KeyEventKind::Release
                    && matches!(key.code, KeyCode::Up | KeyCode::Down)
                    && (key.modifiers == KeyModifiers::SHIFT
                        || key.modifiers == KeyModifiers::ALT
                        || key.modifiers == (KeyModifiers::ALT | KeyModifiers::SHIFT)) =>
            {
                let direction = if key.code == KeyCode::Up { -1 } else { 1 };
                if key.modifiers == (KeyModifiers::ALT | KeyModifiers::SHIFT) {
                    Some(InputEvent::ScrollTocKeyboard(
                        isize::try_from(self.content_rows() / 2).unwrap_or(1) * direction,
                    ))
                } else if key.modifiers == KeyModifiers::ALT {
                    Some(InputEvent::MoveTocEntry(if direction < 0 { -1 } else { 1 }))
                } else {
                    Some(InputEvent::MoveToc(if direction < 0 { -1 } else { 1 }))
                }
            }
            Event::Key(key)
                if key.kind != KeyEventKind::Release
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                match key.code {
                    KeyCode::Char('t') => Some(InputEvent::ToggleToc),
                    KeyCode::Esc if self.toc.visible => Some(InputEvent::ToggleToc),
                    KeyCode::Char('J') if self.toc.visible => Some(InputEvent::MoveToc(1)),
                    KeyCode::Char('K') if self.toc.visible => Some(InputEvent::MoveToc(-1)),
                    KeyCode::Char('D') if self.toc.visible => Some(InputEvent::ScrollTocKeyboard(
                        isize::try_from(self.content_rows() / 2).unwrap_or(1),
                    )),
                    KeyCode::Char('U') if self.toc.visible => Some(InputEvent::ScrollTocKeyboard(
                        -isize::try_from(self.content_rows() / 2).unwrap_or(1),
                    )),
                    KeyCode::Char(c @ '1'..='9') if self.toc.visible => {
                        Some(InputEvent::CycleToc(c as u8 - b'0'))
                    }
                    _ => None,
                }
            }
            Event::Mouse(mouse)
                if self.toc.visible
                    && usize::from(mouse.column) < self.toc_width()
                    && usize::from(mouse.row) < self.content_rows() =>
            {
                match mouse.kind {
                    MouseEventKind::ScrollDown => Some(InputEvent::ScrollToc(3)),
                    MouseEventKind::ScrollUp => Some(InputEvent::ScrollToc(-3)),
                    MouseEventKind::Down(MouseButton::Left) if mouse.row >= 1 => Some(
                        InputEvent::SelectToc(self.toc_offset() + usize::from(mouse.row) - 1),
                    ),
                    _ => Some(InputEvent::Ignore),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn handle_toc_action(&mut self, action: InputEvent) -> Result<(), PromptError> {
        if self.toc.visible
            && matches!(
                action,
                InputEvent::MoveToc(_)
                    | InputEvent::MoveTocEntry(_)
                    | InputEvent::ScrollTocKeyboard(_)
                    | InputEvent::CycleToc(_)
            )
        {
            self.toc.hint_dismissed = true;
        }
        match action {
            InputEvent::ToggleToc => {
                let anchor = self.lines_to_row_map.row_to_line(self.upper_mark);
                self.toc.visible = !self.toc.visible;
                self.toc.reset_position();
                self.clear_selection();
                if self.refresh_layout()? {
                    return Ok(());
                }
                self.reformat_display()?;
                if let Some(row) = anchor.and_then(|line| self.lines_to_row_map.get(line)) {
                    self.upper_mark = (*row).min(self.max_upper_mark());
                }
            }
            InputEvent::ScrollToc(delta) | InputEvent::ScrollTocKeyboard(delta) => {
                self.toc.offset = self.toc_offset().saturating_add_signed(delta).min(
                    self.toc_entries()
                        .len()
                        .saturating_sub(self.content_rows().saturating_sub(1).max(1)),
                );
                self.toc.manual_scroll = true;
            }
            InputEvent::MoveToc(direction) => {
                let active = self.active_toc();
                let target = if direction > 0 {
                    (active + 1..self.toc_entries().len())
                        .find(|&i| self.toc_entries()[i].depth == 0)
                } else {
                    (0..active)
                        .rev()
                        .find(|&i| self.toc_entries()[i].depth == 0)
                };
                if let Some(index) = target {
                    self.select_toc(index);
                }
            }
            InputEvent::MoveTocEntry(direction) => {
                let index = self
                    .active_toc()
                    .saturating_add_signed(isize::from(direction))
                    .min(self.toc_entries().len().saturating_sub(1));
                self.select_toc(index);
            }
            InputEvent::SelectToc(index) => self.select_toc(index),
            InputEvent::CycleToc(number) => {
                if let Some(root) = self
                    .toc_entries()
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.depth == 0)
                    .nth(usize::from(number.saturating_sub(1)))
                    .map(|(i, _)| i)
                {
                    let active = self.active_toc();
                    let next = active + 1;
                    let index = if self.toc.number == Some(number)
                        && active >= root
                        && self.toc_entries().get(next).is_some_and(|e| e.depth == 1)
                    {
                        next
                    } else {
                        root
                    };
                    self.select_toc(index);
                    self.toc.number = Some(number);
                }
            }
            _ => {}
        }
        self.format_prompt()
    }

    fn select_toc(&mut self, index: usize) {
        if let Some(row) = self.toc_target_row(index) {
            self.upper_mark = row.min(self.max_upper_mark());
            self.left_mark = 0;
            self.toc.pinned = Some((index, self.upper_mark));
            self.toc.manual_scroll = false;
            self.toc.number = None;
            self.clear_selection();
        }
    }

    pub(crate) fn draw_toc(&self, out: &mut impl Write) -> Result<(), crate::MinusError> {
        if !self.toc.visible {
            return Ok(());
        }
        let width = self.toc_width();
        let active = self.active_toc();
        let offset = self.toc_offset();
        if self.output_styling {
            write!(out, "\x1b[0m")?;
        }
        let base = PromptStyle::default().foreground(PromptColor::Rgb {
            r: 135,
            g: 135,
            b: 145,
        });
        for row in 0..self.content_rows() {
            let index = offset + row.saturating_sub(1);
            let entry = self.toc_entries().get(index).filter(|_| row >= 1);
            let selected = entry.is_some() && index == active;
            let mut style = if selected {
                base.foreground(PromptColor::Rgb {
                    r: 126,
                    g: 156,
                    b: 216,
                })
            } else {
                base
            };
            if selected || entry.is_some_and(|entry| entry.depth == 0) {
                style = style.attribute(PromptAttribute::Bold);
            }
            let label = match (row, entry) {
                (_, Some(entry)) => {
                    let marker = if selected { "▎" } else { " " };
                    if entry.depth == 0 {
                        let number = self.toc_entries()[..=index]
                            .iter()
                            .filter(|e| e.depth == 0)
                            .count();
                        format!("{marker} {number:02} {}", entry.title)
                    } else {
                        format!("{marker}    • {}", entry.title)
                    }
                }
                (1, None) if self.toc_entries().is_empty() => "  No headings".to_owned(),
                _ => String::new(),
            };
            let line = PromptLine::new()
                .left(PromptSpan::new(label, style)?)
                .truncation_indicator(PromptSpan::new("…", style)?)
                .right(PromptSpan::new("│", base)?)
                .fill_style(base);
            let text = if self.output_styling {
                self.color_depth.adapt(line.render(width)).into_owned()
            } else {
                line.render_plain(width)
            };
            write!(
                out,
                "{}{}",
                crossterm::cursor::MoveTo(0, u16::try_from(row).unwrap_or(u16::MAX)),
                text
            )?;
        }
        Ok(())
    }
}
