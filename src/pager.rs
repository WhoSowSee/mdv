use crate::editor::EditorCommand;
use anyhow::{Context, Result, anyhow};
use minus::hooks::Hook;
use minus::input::{HashedEventRegister, InputClassifier, InputEvent};
use minus::{Pager, PagerState, PromptLine};
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::hash_map::RandomState;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

mod footer;
mod help;

use footer::PagerFooter;
use help::build_help_panel;

const STATUS_MESSAGE_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) struct PagerDocument {
    output: String,
    source: String,
    title: Option<String>,
}

impl PagerDocument {
    pub(super) fn new(output: String, source: String) -> Self {
        Self {
            output,
            source,
            title: None,
        }
    }

    pub(super) fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

pub(super) type RefreshCallback = Arc<dyn Fn() -> Result<PagerDocument> + Send + Sync>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum PagerScreen {
    Alternate,
    InPlace,
}

struct ActiveWatcher {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ActiveWatcher {
    fn start(
        path: &Path,
        pager: Pager,
        refresh: RefreshCallback,
        document: Arc<RwLock<PagerDocument>>,
    ) -> Result<Self> {
        let target = comparable_path(path)?;
        let directory = target
            .parent()
            .context("Markdown file has no parent directory")?
            .to_path_buf();
        let (event_tx, event_rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(event_tx)
            .context("Failed to initialize pager file watcher")?;
        watcher
            .watch(&directory, RecursiveMode::NonRecursive)
            .with_context(|| format!("Failed to watch {}", directory.display()))?;

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let thread = thread::spawn(move || {
            let _watcher = watcher;
            let debounce = Duration::from_millis(100);
            let poll_interval = Duration::from_millis(25);
            let mut refresh_deadline = None;

            while !thread_stop.load(Ordering::SeqCst) {
                match event_rx.recv_timeout(poll_interval) {
                    Ok(Ok(event)) if event_targets_file(&event, &target) => {
                        refresh_deadline = Some(Instant::now() + debounce);
                    }
                    Ok(Ok(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Ok(Err(error)) => {
                        if pager
                            .send_message(single_line_message(&format!(
                                "File watcher error: {error}"
                            )))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                if refresh_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    match refresh().and_then(|refreshed| {
                        apply_refreshed_document(&pager, &document, refreshed)
                    }) {
                        Ok(()) => {}
                        Err(error) => {
                            if pager
                                .send_message(single_line_message(&format!(
                                    "Failed to refresh file: {error:#}"
                                )))
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                    refresh_deadline = None;
                }
            }
        });

        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for ActiveWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct PagerInputClassifier {
    default: HashedEventRegister<RandomState>,
    editor_requested: Arc<AtomicBool>,
    editor_enabled: bool,
    help_panel: Vec<PromptLine>,
    pager: Pager,
    document: Arc<RwLock<PagerDocument>>,
    refresh: Option<RefreshCallback>,
    reload_in_progress: Arc<AtomicBool>,
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
        match help_input_action(&event, help_visible) {
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

pub(super) fn page(
    document: PagerDocument,
    file: Option<PathBuf>,
    refresh: Option<RefreshCallback>,
    screen: PagerScreen,
) -> Result<()> {
    let editor = EditorCommand::from_env();
    let editor_enabled = !matches!(editor, Ok(None)) && file.is_some();
    let help_panel = build_help_panel(editor_enabled, refresh.is_some())?;
    let document = Arc::new(RwLock::new(document));
    let mut pending_message = None;

    loop {
        let editor_requested = Arc::new(AtomicBool::new(false));
        let pager = Pager::new();
        let (output, title) = {
            let document = document
                .read()
                .map_err(|_| anyhow!("Pager document lock poisoned"))?;
            (document.output.clone(), document.title.clone())
        };
        let footer = PagerFooter::new(title.as_deref(), file.as_deref());
        pager.set_text(output)?;
        pager.set_prompt_renderer(move |context| footer.render(context))?;
        pager.set_search_prompt("Find: ")?;
        pager.remove_hook(Hook::PostPagerExit, 1)?;
        pager.set_input_classifier(Box::new(PagerInputClassifier {
            default: HashedEventRegister::default(),
            editor_requested: editor_requested.clone(),
            editor_enabled,
            help_panel: help_panel.clone(),
            pager: pager.clone(),
            document: document.clone(),
            refresh: refresh.clone(),
            reload_in_progress: Arc::new(AtomicBool::new(false)),
        }))?;
        if let Some(message) = pending_message.take() {
            pager.send_message(message)?;
        }

        let watcher = match (&file, &refresh) {
            (Some(path), Some(refresh)) => Some(ActiveWatcher::start(
                path,
                pager.clone(),
                refresh.clone(),
                document.clone(),
            )?),
            _ => None,
        };

        match screen {
            PagerScreen::Alternate => minus::dynamic_paging(pager)?,
            PagerScreen::InPlace => minus::dynamic_paging_in_place(pager)?,
        }
        drop(watcher);

        if !editor_requested.load(Ordering::SeqCst) {
            return Ok(());
        }

        let Some(file) = &file else {
            return Ok(());
        };
        let editor_opened = match &editor {
            Ok(Some(editor)) => match editor.open(file) {
                Ok(()) => true,
                Err(error) => {
                    pending_message = Some(single_line_message(&format!(
                        "Failed to open editor: {error}"
                    )));
                    false
                }
            },
            Err(error) => {
                pending_message = Some(single_line_message(&format!(
                    "Failed to open editor: {error}"
                )));
                false
            }
            Ok(None) => return Ok(()),
        };

        if editor_opened && let Some(refresh) = &refresh {
            match refresh() {
                Ok(refreshed) => replace_document(&document, refreshed)?,
                Err(error) => {
                    pending_message = Some(single_line_message(&format!(
                        "Failed to refresh file: {error:#}"
                    )));
                }
            }
        }
    }
}

fn apply_refreshed_document(
    pager: &Pager,
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    let output = refreshed.output.clone();
    replace_document(document, refreshed)?;
    pager.set_text(output)?;
    Ok(())
}

fn replace_document(document: &RwLock<PagerDocument>, refreshed: PagerDocument) -> Result<()> {
    *document
        .write()
        .map_err(|_| anyhow!("Pager document lock poisoned"))? = refreshed;
    Ok(())
}

fn copy_document_contents(
    document: &RwLock<PagerDocument>,
    selected_text: Option<String>,
) -> Result<()> {
    let text = clipboard_text(document, selected_text)?;
    let mut clipboard = arboard::Clipboard::new().context("Failed to access system clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to write system clipboard")
}

fn clipboard_text(
    document: &RwLock<PagerDocument>,
    selected_text: Option<String>,
) -> Result<String> {
    match selected_text {
        Some(text) => Ok(text),
        None => Ok(document
            .read()
            .map_err(|_| anyhow!("Pager document lock poisoned"))?
            .source
            .clone()),
    }
}

fn report_operation_result(
    pager: &Pager,
    result: Result<()>,
    success_message: &str,
    failure_message: &str,
) {
    let send_result = match result {
        Ok(()) => pager.send_message_for(success_message, STATUS_MESSAGE_TIMEOUT),
        Err(error) => pager.send_message(single_line_message(&format!(
            "{failure_message}: {error:#}"
        ))),
    };
    let _ = send_result;
}

fn is_editor_key(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                && matches!(key.code, KeyCode::Char('E' | 'e' | 'У' | 'у'))
    )
}

fn is_help_key(event: &minus::input::crossterm_event::Event) -> bool {
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
enum HelpInputAction {
    Toggle,
    Dismiss,
    DismissAndForward,
    Forward,
}

fn help_input_action(
    event: &minus::input::crossterm_event::Event,
    help_visible: bool,
) -> HelpInputAction {
    if is_help_key(event) {
        HelpInputAction::Toggle
    } else if help_visible && is_escape_key(event) {
        HelpInputAction::Dismiss
    } else if help_visible && is_search_key(event) {
        HelpInputAction::DismissAndForward
    } else {
        HelpInputAction::Forward
    }
}

fn is_escape_key(event: &minus::input::crossterm_event::Event) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Esc
    )
}

fn is_search_key(event: &minus::input::crossterm_event::Event) -> bool {
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

fn is_copy_key(event: &minus::input::crossterm_event::Event) -> bool {
    is_plain_character_key(event, 'c')
}

fn is_reload_key(event: &minus::input::crossterm_event::Event) -> bool {
    is_plain_character_key(event, 'r')
}

fn is_plain_character_key(event: &minus::input::crossterm_event::Event, character: char) -> bool {
    use minus::input::crossterm_event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Char(character)
    )
}

fn event_targets_file(event: &notify::Event, target: &Path) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
        && event
            .paths
            .iter()
            .filter_map(|path| comparable_path(path).ok())
            .any(|path| path == target)
}

fn comparable_path(path: &Path) -> Result<PathBuf> {
    if let Ok(path) = path.canonicalize() {
        return Ok(path);
    }

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let Some(file_name) = absolute.file_name() else {
        return Ok(absolute);
    };
    let parent = absolute.parent().context("Path has no parent directory")?;
    let parent = parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf());
    Ok(parent.join(file_name))
}

fn single_line_message(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use notify::{EventKind, event::CreateKind};
    use std::sync::atomic::AtomicUsize;
    use tempfile::TempDir;

    #[test]
    fn editor_key_accepts_supported_layouts_without_control_modifiers() {
        for character in ['E', 'e', 'У', 'у'] {
            let event = Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));

            assert!(is_editor_key(&event), "character: {character}");
        }

        assert!(is_editor_key(&Event::Key(KeyEvent::new(
            KeyCode::Char('E'),
            KeyModifiers::SHIFT,
        ))));
        assert!(!is_editor_key(&Event::Key(KeyEvent::new(
            KeyCode::Char('r'),
            KeyModifiers::NONE,
        ))));
        assert!(!is_editor_key(&Event::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::CONTROL,
        ))));
    }

    #[test]
    fn modified_question_mark_does_not_open_help() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::CONTROL));

        assert!(!is_help_key(&event));
    }

    #[test]
    fn escape_closes_visible_help_without_reaching_minus() {
        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert_eq!(help_input_action(&event, true), HelpInputAction::Dismiss);
        assert_eq!(help_input_action(&event, false), HelpInputAction::Forward);
    }

    #[test]
    fn search_closes_visible_help_before_reaching_minus() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));

        assert_eq!(
            help_input_action(&event, true),
            HelpInputAction::DismissAndForward
        );
        assert_eq!(help_input_action(&event, false), HelpInputAction::Forward);
    }

    #[test]
    fn control_f_closes_visible_help_before_reaching_minus() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));

        assert_eq!(
            help_input_action(&event, true),
            HelpInputAction::DismissAndForward
        );
        assert_eq!(help_input_action(&event, false), HelpInputAction::Forward);
    }

    #[test]
    fn question_mark_always_toggles_help() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT));

        assert_eq!(help_input_action(&event, false), HelpInputAction::Toggle);
        assert_eq!(help_input_action(&event, true), HelpInputAction::Toggle);
    }

    #[test]
    fn copy_key_accepts_only_unmodified_c() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        let modified = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(is_copy_key(&event));
        assert!(!is_copy_key(&modified));
    }

    #[test]
    fn reload_key_accepts_only_unmodified_r() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        let modified = Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT));

        assert!(is_reload_key(&event));
        assert!(!is_reload_key(&modified));
    }

    #[test]
    fn watcher_event_must_target_the_current_file() {
        let temp_dir = TempDir::new().unwrap();
        let current_file = temp_dir.path().join("current.md");
        let other_file = temp_dir.path().join("other.md");
        std::fs::write(&current_file, "# Current").unwrap();
        std::fs::write(&other_file, "# Other").unwrap();
        let current_target = comparable_path(&current_file).unwrap();

        let current_event =
            notify::Event::new(EventKind::Create(CreateKind::File)).add_path(current_file.clone());
        let other_event =
            notify::Event::new(EventKind::Create(CreateKind::File)).add_path(other_file);

        assert!(event_targets_file(&current_event, &current_target));
        assert!(!event_targets_file(&other_event, &current_target));
    }

    #[test]
    fn pager_messages_are_single_line() {
        assert_eq!(
            single_line_message("first\nsecond\r\nthird"),
            "first second third"
        );
    }

    #[test]
    fn clipboard_text_prefers_the_selection() {
        let document = RwLock::new(PagerDocument::new(
            "rendered output".to_string(),
            "whole source".to_string(),
        ));

        assert_eq!(
            clipboard_text(&document, Some("selected text".to_string())).unwrap(),
            "selected text"
        );
    }

    #[test]
    fn clipboard_text_uses_the_source_without_a_selection() {
        let document = RwLock::new(PagerDocument::new(
            "rendered output".to_string(),
            "whole source".to_string(),
        ));

        assert_eq!(clipboard_text(&document, None).unwrap(), "whole source");
    }

    #[test]
    fn active_watcher_refreshes_modified_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("watched.md");
        std::fs::write(&file, "# Before").unwrap();
        let refresh_count = Arc::new(AtomicUsize::new(0));
        let callback_count = refresh_count.clone();
        let document = Arc::new(std::sync::RwLock::new(PagerDocument::new(
            "rendered before".to_string(),
            "# Before".to_string(),
        )));
        let refresh = Arc::new(move || {
            callback_count.fetch_add(1, Ordering::SeqCst);
            Ok(PagerDocument::new(
                "rendered after".to_string(),
                "# After".to_string(),
            ))
        });
        let watcher = ActiveWatcher::start(&file, Pager::new(), refresh, document.clone()).unwrap();

        std::fs::write(&file, "# After").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while refresh_count.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }

        drop(watcher);
        assert!(refresh_count.load(Ordering::SeqCst) >= 1);
        assert_eq!(document.read().unwrap().source, "# After");
    }
}
