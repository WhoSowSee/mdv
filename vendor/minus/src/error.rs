//! Public pager error types.

use crate::{PromptError, minus_core::commands::Command};
use std::io;

/// A terminal operation failed.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[allow(clippy::module_name_repetitions)]
pub struct TermError(
    // Keep crossterm's error type out of the public dependency surface.
    #[from] io::Error,
);

/// There was an error while compiling the regex
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[allow(clippy::module_name_repetitions)]
#[cfg(feature = "search")]
#[cfg_attr(docsrs, doc(cfg(feature = "search")))]
pub struct RegexError(
    // Keep regex's error type out of the public dependency surface.
    #[from] regex::Error,
);

/// Errors that can occur during setup.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::module_name_repetitions)]
pub enum SetupError {
    #[error("The standard output is not a valid terminal")]
    InvalidTerminal,

    #[error("Failed to switch to alternate screen")]
    AlternateScreen(#[source] TermError),

    #[error("Failed to enable raw mode")]
    RawMode(#[source] TermError),

    #[error("Failed to hide the cursor")]
    HideCursor(#[source] TermError),

    #[error("Failed to enable mouse capture")]
    EnableMouseCapture(#[source] TermError),

    #[error("Couldn't determine the terminal size")]
    TerminalSize(#[source] TermError),
}

/// Errors that can occur during clean up.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::module_name_repetitions)]
pub enum CleanupError {
    #[error("Failed to disable mouse capture")]
    DisableMouseCapture(#[source] TermError),

    #[error("Failed to show the cursor")]
    ShowCursor(#[source] TermError),

    #[error("Failed to disable raw mode")]
    DisableRawMode(#[source] TermError),

    #[error("Failed to switch back to main screen")]
    LeaveAlternateScreen(#[source] TermError),
}

/// Errors that can happen during runtime.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::module_name_repetitions)]
pub enum MinusError {
    #[error("Failed to initialize the terminal")]
    Setup(#[from] SetupError),

    #[error("Failed to clean up the terminal")]
    Cleanup(#[from] CleanupError),

    #[error("Failed to draw the new data")]
    Draw(#[from] std::io::Error),

    #[error("Failed to handle terminal event")]
    HandleEvent(#[source] TermError),

    #[error("Failed to do an operation on the cursor")]
    Cursor(#[from] TermError),

    #[error("Failed to send formatted data to the pager")]
    FmtWriteError(#[from] std::fmt::Error),

    #[error("Failed to send data to the receiver")]
    Communication(#[from] crossbeam_channel::SendError<Command>),

    #[error("Failed to convert between some primitives")]
    Conversion,

    #[error(transparent)]
    Prompt(#[from] PromptError),

    #[error(transparent)]
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    SearchExpError(#[from] RegexError),

    #[cfg(test)]
    #[error(transparent)]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    JoinError(#[from] tokio::task::JoinError),
}

#[cfg(feature = "search")]
impl From<regex::Error> for MinusError {
    fn from(e: regex::Error) -> Self {
        Self::SearchExpError(RegexError::from(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn terminal_error_chains_preserve_the_system_cause() {
        let failure = || {
            TermError::from(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "access denied",
            ))
        };
        let setup = [
            SetupError::AlternateScreen,
            SetupError::RawMode,
            SetupError::HideCursor,
            SetupError::EnableMouseCapture,
            SetupError::TerminalSize,
        ]
        .map(|wrap| MinusError::Setup(wrap(failure())));
        let cleanup = [
            CleanupError::DisableMouseCapture,
            CleanupError::ShowCursor,
            CleanupError::DisableRawMode,
            CleanupError::LeaveAlternateScreen,
        ]
        .map(|wrap| MinusError::Cleanup(wrap(failure())));
        for error in setup
            .into_iter()
            .chain(cleanup)
            .chain([MinusError::HandleEvent(failure())])
        {
            let mut cause: &dyn Error = &error;
            while let Some(source) = cause.source() {
                cause = source;
            }
            assert_eq!(cause.to_string(), "access denied", "{error:?}");
        }
    }
}
