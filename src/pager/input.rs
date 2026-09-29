use super::*;

pub(super) struct PagerInputClassifier {
    pub(super) default: HashedEventRegister<RandomState>,
    pub(super) editor_requested: Arc<AtomicBool>,
    pub(super) editor_enabled: bool,
    pub(super) help_panel: Vec<PromptLine>,
    pub(super) help_transparent: bool,
    pub(super) pager: Pager,
    pub(super) document: Arc<RwLock<PagerDocument>>,
    pub(super) refresh: Option<RefreshCallback>,
    pub(super) reload_in_progress: Arc<AtomicBool>,
}

impl PagerInputClassifier {
    fn set_help_visible(&self, visible: bool, width: usize, toc: bool) {
        let result = if visible {
            let lines = if toc {
                super::help::build_toc_help_panel(width, self.help_transparent)
            } else {
                super::help::fit_help_panel(&self.help_panel, width, self.help_transparent)
            };
            lines
                .map_err(minus::error::MinusError::from)
                .and_then(|lines| self.pager.set_prompt_panel(lines))
        } else {
            self.pager.clear_prompt_panel()
        };
        if let Err(error) = result {
            let _ = self.pager.send_message(single_line_message(&format!(
                "Failed to update help: {error}"
            )));
        }
    }

    fn copy_contents(&self, selected_text: Option<String>) {
        let Some(text) = selected_text.filter(|text| !text.is_empty()) else {
            return;
        };
        let pager = self.pager.clone();
        thread::spawn(move || {
            report_operation_result(
                &pager,
                copy_document_contents(text),
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

    fn cycle_line_numbers(&self) -> bool {
        match cycle_line_number_mode(&self.pager, &self.document) {
            Ok(changed) => changed,
            Err(error) => {
                let _ = self.pager.send_message(single_line_message(&format!(
                    "Failed to change line-number mode: {error}"
                )));
                true
            }
        }
    }
}

impl InputClassifier for PagerInputClassifier {
    fn classify_input(
        &self,
        event: minus::input::crossterm_event::Event,
        state: &PagerState,
    ) -> Option<InputEvent> {
        let help_visible = state.prompt_panel_rows() > 0;
        if help_visible && let minus::input::crossterm_event::Event::Resize(width, _) = &event {
            self.set_help_visible(true, usize::from(*width), state.toc_visible());
        }
        let default_action = self.default.classify_input(event.clone(), state);
        let opens_input_prompt = matches!(
            default_action,
            Some(InputEvent::Search(_) | InputEvent::GoToLine)
        );
        match help_input_action(&event, help_visible, opens_input_prompt) {
            HelpInputAction::Toggle => {
                self.set_help_visible(!help_visible, state.cols, state.toc_visible());
                return None;
            }
            HelpInputAction::Dismiss => {
                self.set_help_visible(false, state.cols, state.toc_visible());
                return None;
            }
            HelpInputAction::DismissAndForward => {
                self.set_help_visible(false, state.cols, state.toc_visible())
            }
            HelpInputAction::Forward => {}
        }

        if let Some(action) = state.toc_input(&event) {
            if help_visible {
                self.set_help_visible(false, state.cols, state.toc_visible());
            }
            return Some(action);
        }

        if is_line_number_key(&event) && self.cycle_line_numbers() {
            None
        } else if is_copy_event(&event) {
            self.copy_contents(state.selected_text());
            None
        } else if self.refresh.is_some() && is_reload_key(&event) {
            self.reload_document();
            None
        } else if self.editor_enabled && is_editor_key(&event) {
            self.editor_requested.store(true, Ordering::SeqCst);
            Some(InputEvent::Exit)
        } else {
            default_action
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
    opens_input_prompt: bool,
) -> HelpInputAction {
    if is_help_key(event) {
        HelpInputAction::Toggle
    } else if help_visible && is_escape_key(event) {
        HelpInputAction::Dismiss
    } else if help_visible && opens_input_prompt {
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

pub(super) fn is_copy_event(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{
        Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
    };

    match event {
        Event::Key(key) => {
            key.kind == KeyEventKind::Press
                && key.code == KeyCode::Char('c')
                && matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::CONTROL)
        }
        Event::Mouse(mouse) => {
            mouse.kind == MouseEventKind::Down(MouseButton::Right)
                && mouse.modifiers == KeyModifiers::NONE
        }
        _ => false,
    }
}

pub(super) fn is_reload_key(event: &minus::input::crossterm_event::Event) -> bool {
    is_plain_character_key(event, 'r')
}

pub(super) fn is_line_number_key(event: &minus::input::crossterm_event::Event) -> bool {
    is_plain_character_key(event, 'l')
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
