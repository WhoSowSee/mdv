use super::*;

pub(super) struct PagerInputClassifier {
    pub(super) default: HashedEventRegister<RandomState>,
    pub(super) editor_requested: Arc<AtomicBool>,
    pub(super) editor_enabled: bool,
    pub(super) help_panel: Vec<PromptLine>,
    pub(super) pager: Pager,
    pub(super) document: Arc<RwLock<PagerDocument>>,
    pub(super) refresh: Option<RefreshCallback>,
    pub(super) reload_in_progress: Arc<AtomicBool>,
}

impl PagerInputClassifier {
    fn set_help_visible(&self, visible: bool) {
        let result = if visible {
            self.pager.set_prompt_panel(self.help_panel.clone())
        } else {
            self.pager.clear_prompt_panel()
        };
        if let Err(error) = result {
            let _ = self.pager.send_message(single_line_message(&format!(
                "Failed to update help: {error}"
            )));
        }
    }

    fn toggle_help(&self, visible: bool) {
        self.set_help_visible(!visible);
    }

    fn copy_contents(&self, selected_text: Option<String>) {
        let pager = self.pager.clone();
        let document = self.document.clone();
        thread::spawn(move || {
            report_operation_result(
                &pager,
                copy_document_contents(&document, selected_text),
                "Copied contents",
                "Failed to copy contents",
            );
        });
    }

    fn reload_document(&self) {
        let Some(refresh) = self.refresh.clone() else {
            return;
        };
        if self
            .reload_in_progress
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let pager = self.pager.clone();
        let document = self.document.clone();
        let reload_in_progress = self.reload_in_progress.clone();
        thread::spawn(move || {
            let result = refresh()
                .and_then(|refreshed| apply_refreshed_document(&pager, &document, refreshed));
            reload_in_progress.store(false, Ordering::SeqCst);
            report_operation_result(&pager, result, "Reloaded document", "Failed to reload file");
        });
    }
}

impl InputClassifier for PagerInputClassifier {
    fn classify_input(
        &self,
        event: minus::input::crossterm_event::Event,
        state: &PagerState,
    ) -> Option<InputEvent> {
        let help_visible = state.prompt_panel_rows() > 0;
        match help_input_action(&event, help_visible, state.search_is_active()) {
            HelpInputAction::Toggle => {
                self.toggle_help(help_visible);
                return None;
            }
            HelpInputAction::Dismiss => {
                self.set_help_visible(false);
                return None;
            }
            HelpInputAction::DismissAndForward => self.set_help_visible(false),
            HelpInputAction::Forward => {}
        }

        if is_copy_key(&event) {
            self.copy_contents(state.selected_text());
            None
        } else if self.refresh.is_some() && is_reload_key(&event) {
            self.reload_document();
            None
        } else if self.editor_enabled && is_editor_key(&event) {
            self.editor_requested.store(true, Ordering::SeqCst);
            Some(InputEvent::Exit)
        } else {
            self.default.classify_input(event, state)
        }
    }
}

pub(super) fn is_editor_key(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                && matches!(key.code, KeyCode::Char('E' | 'e' | 'У' | 'у'))
    )
}

pub(super) fn is_help_key(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                && key.code == KeyCode::Char('?')
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum HelpInputAction {
    Toggle,
    Dismiss,
    DismissAndForward,
    Forward,
}

pub(super) fn help_input_action(
    event: &minus::input::crossterm_event::Event,
    help_visible: bool,
    search_active: bool,
) -> HelpInputAction {
    if is_help_key(event) {
        HelpInputAction::Toggle
    } else if help_visible && is_escape_key(event) && !search_active {
        HelpInputAction::Dismiss
    } else if help_visible && is_search_key(event) {
        HelpInputAction::DismissAndForward
    } else {
        HelpInputAction::Forward
    }
}

pub(super) fn is_escape_key(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Esc
    )
}

pub(super) fn is_search_key(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    is_plain_character_key(event, '/')
        || matches!(
            event,
            Event::Key(key)
                if key.kind == KeyEventKind::Press
                    && key.modifiers == KeyModifiers::CONTROL
                    && key.code == KeyCode::Char('f')
        )
}

pub(super) fn is_copy_key(event: &minus::input::crossterm_event::Event) -> bool {
    is_plain_character_key(event, 'c')
}

pub(super) fn is_reload_key(event: &minus::input::crossterm_event::Event) -> bool {
    is_plain_character_key(event, 'r')
}

pub(super) fn is_plain_character_key(
    event: &minus::input::crossterm_event::Event,
    character: char,
) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Char(character)
    )
}
