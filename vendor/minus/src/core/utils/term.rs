#![allow(dead_code)]

use crate::error::{CleanupError, MinusError, SetupError};
use crossterm::{
    cursor, event, execute, queue,
    terminal::{self, Clear},
    tty::IsTty,
};
use std::io;

// Keep setup and cleanup adjacent; cleanup must reverse this sequence.
pub fn setup(
    out: &mut io::Stdout,
    use_alternate_screen: bool,
) -> std::result::Result<(), SetupError> {
    if out.is_tty() {
        Ok(())
    } else {
        Err(SetupError::InvalidTerminal)
    }?;

    if use_alternate_screen {
        execute!(out, terminal::EnterAlternateScreen)
            .map_err(|e| SetupError::AlternateScreen(e.into()))?;
    }
    terminal::enable_raw_mode().map_err(|e| SetupError::RawMode(e.into()))?;
    execute!(out, event::EnableMouseCapture)
        .map_err(|e| SetupError::EnableMouseCapture(e.into()))?;
    execute!(out, cursor::Hide).map_err(|e| SetupError::HideCursor(e.into()))?;
    Ok(())
}

pub fn cleanup(
    out: &mut impl io::Write,
    cleanup_screen: bool,
    use_alternate_screen: bool,
) -> std::result::Result<(), CleanupError> {
    if cleanup_screen {
        execute!(out, cursor::Show).map_err(|e| CleanupError::ShowCursor(e.into()))?;
        execute!(out, event::DisableMouseCapture)
            .map_err(|e| CleanupError::DisableMouseCapture(e.into()))?;
        terminal::disable_raw_mode().map_err(|e| CleanupError::DisableRawMode(e.into()))?;
        if use_alternate_screen {
            execute!(out, terminal::LeaveAlternateScreen)
                .map_err(|e| CleanupError::LeaveAlternateScreen(e.into()))?;
        }
    }
    Ok(())
}

pub fn move_cursor(
    out: &mut impl io::Write,
    x: u16,
    y: u16,
    flush: bool,
) -> Result<(), MinusError> {
    queue!(out, cursor::MoveTo(x, y))?;
    if flush {
        out.flush()?;
    }
    Ok(())
}

pub fn clear_entire_screen(out: &mut impl io::Write, flush: bool) -> crate::Result {
    queue!(out, Clear(terminal::ClearType::All))?;
    if flush {
        out.flush()?;
    }
    Ok(())
}
