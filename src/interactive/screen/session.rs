use super::*;

pub(in crate::interactive) struct TerminalSession {
    stdout: Stdout,
    active: bool,
}

impl TerminalSession {
    pub(in crate::interactive) fn enter() -> Result<Self> {
        let mut session = Self {
            stdout: stdout(),
            active: false,
        };
        session.resume()?;
        Ok(session)
    }

    pub(in crate::interactive) fn draw(&mut self, app: &App) -> Result<()> {
        queue!(self.stdout, Hide, MoveTo(0, 0), Clear(ClearType::All))?;
        draw_browser(&mut self.stdout, app)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub(in crate::interactive) fn suspend(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let restore_result = execute!(
            self.stdout,
            DisableBracketedPaste,
            ResetColor,
            Show,
            LeaveAlternateScreen
        );
        let raw_result = disable_raw_mode();
        self.active = false;
        restore_result?;
        raw_result?;
        Ok(())
    }

    pub(in crate::interactive) fn pause_for_pager(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let display_result = write_pager_pause(&mut self.stdout);
        let raw_result = disable_raw_mode();
        display_result?;
        raw_result?;
        Ok(())
    }

    pub(in crate::interactive) fn resume_after_pager(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        enable_raw_mode()?;
        if let Err(error) = write_pager_resume(&mut self.stdout) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        Ok(())
    }

    pub(in crate::interactive) fn resume(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }
        enable_raw_mode()?;
        if let Err(error) = execute!(
            self.stdout,
            EnterAlternateScreen,
            EnableBracketedPaste,
            Hide
        ) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        self.active = true;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.suspend();
    }
}

pub(super) fn write_pager_pause(output: &mut impl Write) -> std::io::Result<()> {
    execute!(output, DisableBracketedPaste, ResetColor, Show)
}

pub(super) fn write_pager_resume(output: &mut impl Write) -> std::io::Result<()> {
    execute!(output, EnableBracketedPaste, Hide)
}
