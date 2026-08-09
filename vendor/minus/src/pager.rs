//! Provides [`Pager`], the application-facing command handle.

use crate::{
    ExitStrategy, LineNumbers, PromptContext, PromptError, PromptLine, PromptRenderer,
    error::MinusError,
    hooks::{Hook, HookCallback},
    input,
    minus_core::commands::Command,
    prompt::validate_single_line,
};
use crossbeam_channel::{Receiver, Sender};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

#[cfg(feature = "search")]
use crate::search::SearchOpts;

/// Sends content and configuration commands to a running pager.
///
/// Clones share the same command channel. The handle also implements [`fmt::Write`].
#[derive(Clone)]
pub struct Pager {
    pub(crate) tx: Sender<Command>,
    pub(crate) rx: Receiver<Command>,
    message_sequence: Arc<AtomicUsize>,
}

#[allow(clippy::missing_errors_doc)]
impl Pager {
    /// Creates an unattached pager handle.
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self {
            tx,
            rx,
            message_sequence: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Replaces all pager content.
    pub fn set_text(&self, s: impl Into<String>) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetData(s.into()))?)
    }

    /// Appends content without requiring a mutable handle.
    pub fn push_str(&self, s: impl Into<String>) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::AppendData(s.into()))?)
    }

    /// Sets line-number visibility and toggle behavior.
    pub fn set_line_numbers(&self, l: LineNumbers) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetLineNumbers(l))?)
    }

    /// Sets the single-line text displayed in the prompt.
    pub fn set_prompt(&self, text: impl Into<String>) -> Result<(), MinusError> {
        let text: String = text.into();
        validate_single_line(&text)?;
        Ok(self.tx.send(Command::SetPrompt(text))?)
    }

    /// Installs the renderer used to build the prompt line.
    pub fn set_prompt_renderer<F>(&self, renderer: F) -> Result<(), MinusError>
    where
        F: for<'a> Fn(&PromptContext<'a>) -> Result<PromptLine, PromptError>
            + Send
            + Sync
            + 'static,
    {
        let renderer: PromptRenderer = Arc::new(renderer);
        Ok(self.tx.send(Command::SetPromptRenderer(Some(renderer)))?)
    }

    /// Restores the default prompt renderer.
    pub fn clear_prompt_renderer(&self) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetPromptRenderer(None))?)
    }

    /// Panel rows reduce the document viewport and preserve bottom anchoring.
    pub fn set_prompt_panel(&self, lines: Vec<PromptLine>) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetPromptPanel(lines))?)
    }

    /// Removes all rows below the prompt.
    pub fn clear_prompt_panel(&self) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetPromptPanel(Vec::new()))?)
    }

    /// Sets the single-line prefix used by both search directions.
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn set_search_prompt(&self, text: impl Into<String>) -> Result<(), MinusError> {
        let text = text.into();
        validate_single_line(&text)?;
        Ok(self.tx.send(Command::SetSearchPrompt(Some(text)))?)
    }

    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    /// Restores the default search prefixes.
    pub fn clear_search_prompt(&self) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetSearchPrompt(None))?)
    }

    /// Displays a single-line message until the next input event.
    pub fn send_message(&self, text: impl Into<String>) -> Result<(), MinusError> {
        let text: String = text.into();
        validate_single_line(&text)?;
        Ok(self.tx.send(Command::SendMessage(text))?)
    }

    /// Displays a single-line message for `duration` without clearing a newer message.
    pub fn send_message_for(
        &self,
        text: impl Into<String>,
        duration: Duration,
    ) -> Result<(), MinusError> {
        let text = text.into();
        validate_single_line(&text)?;
        let id = self
            .message_sequence
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        self.tx.send(Command::SetTimedMessage { text, id })?;

        let tx = self.tx.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let _ = tx.send(Command::ClearMessage(id));
        });
        Ok(())
    }

    /// Sets how the pager exits.
    #[deprecated(
        since = "5.7.0",
        note = "Add a callback for [`PostPagerExit`](crate::hooks::Hook::PostPagerExit) hook. See [`hooks`](crate::hooks) for more info."
    )]
    pub fn set_exit_strategy(&self, es: ExitStrategy) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetExitStrategy(es))?)
    }

    /// Forces static paging even when all content fits in the terminal.
    #[cfg(feature = "static_output")]
    #[cfg_attr(docsrs, doc(cfg(feature = "static_output")))]
    pub fn set_run_no_overflow(&self, val: bool) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetRunNoOverflow(val))?)
    }

    /// Enables horizontal scrolling and disables line wrapping.
    pub fn horizontal_scroll(&self, value: bool) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::LineWrapping(!value))?)
    }

    /// Replaces the input classifier used for keyboard and mouse events.
    pub fn set_input_classifier(
        &self,
        handler: Box<dyn input::InputClassifier + Send + Sync>,
    ) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::SetInputClassifier(handler))?)
    }

    /// Adds an exit callback, invoked in registration order.
    #[deprecated(
        since = "5.7.0",
        note = "Add a callback for [PostPagerExit](crate::hooks::Hook::PostPagerExit) hook. See [hooks](crate::hooks) for more info."
    )]
    pub fn add_exit_callback(
        &self,
        cb: Box<dyn FnMut() + Send + Sync + 'static>,
    ) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::AddExitCallback(cb))?)
    }

    /// Adds a callback for `hook`; an `id` of zero requests automatic assignment.
    ///
    /// # Panics
    /// Panics when the same nonzero `id` is registered twice for one hook.
    pub fn add_hook(&self, hook: Hook, id: u64, cb: HookCallback) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::AddHook(hook, id, cb))?)
    }

    /// Removes the callback identified by `id` from `hook`.
    pub fn remove_hook(&self, hook: Hook, id: u64) -> Result<(), MinusError> {
        Ok(self.tx.send(Command::RemoveHook(hook, id))?)
    }

    /// Replaces the predicate that enables incremental search.
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn set_incremental_search_condition(
        &self,
        cb: Box<dyn Fn(&SearchOpts) -> bool + Send + Sync + 'static>,
    ) -> crate::Result {
        self.tx.send(Command::IncrementalSearchCondition(cb))?;
        Ok(())
    }

    /// Shows or hides the prompt row without disabling search input.
    pub fn show_prompt(&self, show: bool) -> crate::Result {
        self.tx.send(Command::ShowPrompt(show))?;
        Ok(())
    }

    /// Keeps the viewport anchored to the end of incoming output.
    pub fn follow_output(&self, follow_output: bool) -> crate::Result {
        self.tx.send(Command::FollowOutput(follow_output))?;
        Ok(())
    }
}

impl Default for Pager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for Pager {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "dynamic_output")]
    #[test]
    fn basic_dynamic_paging() {
        use super::*;
        use crate::{RunMode, input::InputEvent, minus_core::RUNMODE};

        // Tests share the process-global run mode.
        *RUNMODE.lock() = RunMode::Uninitialized;

        let pager = Pager::new();
        pager.follow_output(true).unwrap();

        let pager2 = pager.clone();

        std::thread::scope(|s| {
            s.spawn(move || crate::dynamic_pager::dynamic_paging(pager2));
            s.spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(50));
                pager.tx.send(Command::UserInput(InputEvent::Exit)).unwrap();
            });
        });

        assert_eq!(*RUNMODE.lock(), RunMode::Uninitialized);
    }
}
