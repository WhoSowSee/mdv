#![allow(dead_code)]

use crossterm::{
    SynchronizedUpdate,
    cursor::MoveTo,
    queue,
    terminal::{Clear, ClearType},
};

use std::{cmp::Ordering, convert::TryInto, fmt::Display, io::Write};

use super::term;
use crate::{LineNumbers, PagerState, error::MinusError, minus_core};

#[derive(Debug, PartialEq, Eq)]
pub enum AppendStyle {
    PartialUpdate((usize, usize)),

    FullRedraw,
}

pub fn draw_for_change(
    out: &mut impl Write,
    ps: &mut PagerState,
    new_upper_mark: &mut usize,
) -> Result<(), MinusError> {
    let line_count = ps.screen.formatted_lines_count();

    // Scroll bounds must use the panel-aware viewport height.
    let writable_rows = ps.content_rows();

    let lower_bound = ps.upper_mark.saturating_add(writable_rows).min(line_count);
    *new_upper_mark = (*new_upper_mark).min(ps.max_upper_mark());
    let new_lower_bound = new_upper_mark.saturating_add(writable_rows).min(line_count);

    if ps.prompt_panel_rows() > 0 || ps.search_is_active() {
        if *new_upper_mark == ps.upper_mark {
            return Ok(());
        }
        ps.upper_mark = *new_upper_mark;
        out.sync_update(|out| draw_full(out, ps))??;
        return Ok(());
    }

    let delta = new_upper_mark.abs_diff(ps.upper_mark);
    // Large jumps redraw at most one viewport; downward bounds still use the clamped lower bound.
    let normalized_delta = delta.min(writable_rows);

    if *new_upper_mark == ps.upper_mark {
        return Ok(());
    }

    out.sync_update(|out| -> Result<(), MinusError> {
        let (start, end) = match (*new_upper_mark).cmp(&ps.upper_mark) {
            Ordering::Greater => {
                queue!(
                    out,
                    crossterm::terminal::ScrollUp(normalized_delta.try_into().unwrap())
                )?;
                term::move_cursor(
                    out,
                    0,
                    writable_rows
                        .saturating_sub(normalized_delta)
                        .try_into()
                        .unwrap(),
                    false,
                )?;
                queue!(out, Clear(ClearType::CurrentLine))?;

                if delta < writable_rows {
                    (lower_bound, new_lower_bound)
                } else {
                    (*new_upper_mark, new_lower_bound)
                }
            }
            Ordering::Less => {
                queue!(
                    out,
                    crossterm::terminal::ScrollDown(normalized_delta.try_into().unwrap())
                )?;
                term::move_cursor(out, 0, 0, false)?;

                (
                    *new_upper_mark,
                    new_upper_mark.saturating_add(normalized_delta),
                )
            }
            Ordering::Equal => return Ok(()),
        };

        let lines = ps.render_rows_for_display(start, end);
        write_raw_lines(out, &lines, Some("\r"))?;

        ps.upper_mark = *new_upper_mark;
        ps.format_prompt()?;

        if ps.show_prompt {
            super::display::write_prompt_view(out, ps)?;
        }
        out.flush()?;

        Ok(())
    })??;

    Ok(())
}

pub fn write_prompt_view(out: &mut impl Write, ps: &PagerState) -> Result<(), MinusError> {
    let prompt_row: u16 = ps
        .prompt_row()
        .try_into()
        .map_err(|_| MinusError::Conversion)?;
    write!(
        out,
        "{}\r{}{}",
        MoveTo(0, prompt_row),
        crossterm::style::Attribute::Reset,
        ps.displayed_prompt
    )?;
    for line in ps
        .displayed_prompt_panel
        .iter()
        .take(ps.prompt_panel_rows())
    {
        write!(out, "\n\r{line}")?;
    }
    out.flush()?;
    Ok(())
}

pub fn draw_full(out: &mut impl Write, ps: &mut PagerState) -> Result<(), MinusError> {
    ps.format_prompt()?;
    super::term::move_cursor(out, 0, 0, false)?;
    queue!(out, Clear(ClearType::All))?;

    write_from_pagerstate(out, ps)?;

    if ps.show_prompt {
        write_prompt_view(out, ps)?;
    }

    out.flush().map_err(MinusError::Draw)
}

pub fn draw_selection_rows(
    out: &mut impl Write,
    ps: &PagerState,
    start: usize,
    end: usize,
) -> Result<(), MinusError> {
    let viewport_start = ps.upper_mark;
    let viewport_end = viewport_start.saturating_add(ps.content_rows());
    let first_row = start.max(viewport_start);
    let last_row = end.min(viewport_end.saturating_sub(1));
    if first_row > last_row || viewport_start >= viewport_end {
        return Ok(());
    }

    let rows = ps.render_rows_for_display(first_row, last_row.saturating_add(1));
    out.sync_update(|out| -> Result<(), MinusError> {
        for (offset, row) in rows.iter().enumerate() {
            let screen_row = first_row
                .saturating_sub(viewport_start)
                .saturating_add(offset)
                .try_into()
                .map_err(|_| MinusError::Conversion)?;
            queue!(out, MoveTo(0, screen_row), Clear(ClearType::CurrentLine))?;
            write!(out, "\r{row}")?;
        }
        Ok(())
    })??;
    Ok(())
}

pub fn draw_append_text<L: Display + AsRef<str>>(
    out: &mut impl Write,
    rows: usize,
    prev_unterminated: usize,
    prev_fmt_lines_count: usize,
    fmt_text: &[L],
) -> Result<(), MinusError> {
    if prev_fmt_lines_count < rows {
        term::move_cursor(
            out,
            0,
            prev_fmt_lines_count
                .saturating_sub(prev_unterminated)
                .try_into()
                .unwrap(),
            false,
        )?;
        let available_rows = rows.saturating_sub(
            prev_fmt_lines_count
                .saturating_sub(prev_unterminated)
                .saturating_add(1),
        );
        let num_appendable = fmt_text.len().min(available_rows);
        if num_appendable >= 1 {
            crossterm::execute!(out, crossterm::terminal::Clear(ClearType::CurrentLine))?;
        }
        for line in &fmt_text[0..num_appendable] {
            write!(out, "{line}\n\r")?;
        }
        out.flush()?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn write_text_checked<L: Display + AsRef<str>>(
    out: &mut impl Write,
    lines: &[L],
    mut upper_mark: usize,
    rows: usize,
    cols: usize,
    line_wrapping: bool,
    left_mark: usize,
    line_numbers: LineNumbers,
    total_line_count: usize,
) -> Result<(), MinusError> {
    let line_count = lines.len();

    let writable_rows = rows.saturating_sub(1);

    let mut lower_mark = upper_mark.saturating_add(writable_rows.min(line_count));

    if lower_mark > line_count {
        upper_mark = line_count.saturating_sub(writable_rows);
        lower_mark = upper_mark.saturating_add(writable_rows.min(line_count));
    }

    let display_lines = &lines[upper_mark..lower_mark];

    term::move_cursor(out, 0, 0, false)?;
    term::clear_entire_screen(out, false)?;

    write_lines(
        out,
        display_lines,
        cols,
        line_wrapping,
        left_mark,
        line_numbers.is_on(),
        total_line_count,
    )
}

pub fn write_from_pagerstate(out: &mut impl Write, ps: &mut PagerState) -> Result<(), MinusError> {
    let line_count = ps.screen.formatted_lines_count();

    let writable_rows = ps.content_rows();
    ps.upper_mark = ps.upper_mark.min(ps.max_upper_mark());
    let lower_mark = ps.upper_mark.saturating_add(writable_rows).min(line_count);

    let display_lines = ps.render_rows_for_display(ps.upper_mark, lower_mark);
    write_raw_lines(out, &display_lines, Some("\r"))
}

pub fn write_lines<L: Display + AsRef<str>>(
    out: &mut impl Write,
    lines: &[L],
    cols: usize,
    line_wrapping: bool,
    left_mark: usize,
    line_numbers: bool,
    line_count: usize,
) -> crate::Result {
    if line_wrapping {
        write_raw_lines(out, lines, Some("\r"))
    } else {
        write_lines_in_horizontal_scroll(out, lines, cols, left_mark, line_numbers, line_count)
    }
}

pub fn write_lines_in_horizontal_scroll<L: Display + AsRef<str>>(
    out: &mut impl Write,
    lines: &[L],
    cols: usize,
    start: usize,
    line_numbers: bool,
    line_count: usize,
) -> crate::Result {
    for line in lines {
        let line_str = line.as_ref();
        let (first_end, second_start, second_end) =
            get_horizontal_scroll_bounds(line_str, cols, start, line_numbers, line_count);

        if start < line.as_ref().len() {
            if line_numbers {
                writeln!(
                    out,
                    "\r{}{}",
                    &line_str[0..first_end],
                    &line_str[second_start..second_end]
                )?;
            } else {
                writeln!(out, "\r{}", &line_str[second_start..second_end])?;
            }
        } else {
            writeln!(out, "\r")?;
        }
    }
    Ok(())
}

#[must_use]
pub fn get_horizontal_scroll_bounds(
    line: &str,
    cols: usize,
    start: usize,
    line_numbers: bool,
    line_count: usize,
) -> (usize, usize, usize) {
    let line_number_ascii_seq_len = if line_numbers { 8 } else { 0 };
    let line_number_padding = if line_numbers {
        minus_core::utils::digits(line_count) + LineNumbers::EXTRA_PADDING + 2
    } else {
        0
    };
    let shifted_start = if line_numbers {
        start + line_number_padding + line_number_ascii_seq_len
    } else {
        start
    };

    let end = if line_numbers {
        shifted_start
            + cols
                .saturating_sub(line_number_padding)
                .min(line.len().saturating_sub(shifted_start))
    } else {
        shifted_start + cols.min(line.len().saturating_sub(shifted_start))
    };

    // Byte offsets advance to the next UTF-8 boundary before slicing.
    if line_numbers {
        let first_end = (line_number_padding + line_number_ascii_seq_len).min(line.len());
        let second_start = shifted_start.min(line.len());
        let second_end = end.min(line.len());

        let mut i = second_start;
        while i < second_end && !line.is_char_boundary(i) {
            i += 1;
        }
        (first_end, i, second_end)
    } else {
        let resolved_start = shifted_start.min(line.len());
        let resolved_end = end.min(line.len());

        let mut i = resolved_start;
        while i < resolved_end && !line.is_char_boundary(i) {
            i += 1;
        }
        (0, i, resolved_end)
    }
}

pub fn write_raw_lines<L: Display>(
    out: &mut impl Write,
    lines: &[L],
    initial: Option<&str>,
) -> Result<(), MinusError> {
    for line in lines {
        writeln!(out, "{}{line}", initial.unwrap_or(""))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
