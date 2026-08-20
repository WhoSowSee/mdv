use super::*;

#[derive(PartialEq)]
pub(super) struct ScreenFrame {
    width: u16,
    rows: Vec<String>,
    cursor: Option<(u16, u16)>,
}

impl ScreenFrame {
    pub(super) fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            rows: vec![String::new(); usize::from(height)],
            cursor: None,
        }
    }

    pub(super) fn write_line(&mut self, y: u16, text: &str) {
        if let Some(row) = self.rows.get_mut(usize::from(y)) {
            row.clear();
            row.push_str(text);
        }
    }

    pub(super) fn show_cursor_at(&mut self, x: u16, y: u16) {
        if x < self.width && usize::from(y) < self.rows.len() {
            self.cursor = Some((x, y));
        }
    }
}

fn render_frame_diff(
    output: &mut String,
    previous: Option<&ScreenFrame>,
    next: &ScreenFrame,
) -> std::fmt::Result {
    Hide.write_ansi(output)?;
    let full_redraw = previous
        .is_none_or(|frame| frame.width != next.width || frame.rows.len() != next.rows.len());
    if full_redraw {
        MoveTo(0, 0).write_ansi(output)?;
        Clear(ClearType::All).write_ansi(output)?;
        for (y, row) in next
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| !row.is_empty())
        {
            render_row(output, y, row)?;
        }
    } else if let Some(previous) = previous {
        for (y, row) in next.rows.iter().enumerate() {
            if previous.rows[y] != *row {
                render_row(output, y, row)?;
            }
        }
    }

    if let Some((x, y)) = next.cursor {
        MoveTo(x, y).write_ansi(output)?;
        Show.write_ansi(output)?;
    }
    Ok(())
}

pub(super) fn encode_synchronized_frame(
    output: &mut String,
    previous: Option<&ScreenFrame>,
    next: &ScreenFrame,
) -> std::fmt::Result {
    if previous == Some(next) {
        return Ok(());
    }

    BeginSynchronizedUpdate.write_ansi(output)?;
    let render_result = render_frame_diff(output, previous, next);
    EndSynchronizedUpdate.write_ansi(output)?;
    render_result
}

fn render_row(output: &mut String, y: usize, row: &str) -> std::fmt::Result {
    MoveTo(0, y as u16).write_ansi(output)?;
    output.push_str(row);
    Clear(ClearType::UntilNewLine).write_ansi(output)
}
