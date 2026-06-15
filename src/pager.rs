use crate::editor::EditorCommand;
use anyhow::{Context, Result};
use minus::hooks::Hook;
use minus::input::{HashedEventRegister, InputClassifier, InputEvent};
use minus::{Pager, PagerState};
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::hash_map::RandomState;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(crate) type RefreshCallback = Arc<dyn Fn() -> Result<String> + Send + Sync>;

struct ActiveWatcher {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ActiveWatcher {
    fn start(path: &Path, pager: Pager, refresh: RefreshCallback) -> Result<Self> {
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
                    match refresh() {
                        Ok(output) => {
                            if pager.set_text(output).is_err() {
                                break;
                            }
                        }
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
}

impl InputClassifier for PagerInputClassifier {
    fn classify_input(
        &self,
        event: minus::input::crossterm_event::Event,
        state: &PagerState,
    ) -> Option<InputEvent> {
        if is_editor_key(&event) {
            self.editor_requested.store(true, Ordering::SeqCst);
            Some(InputEvent::Exit)
        } else {
            self.default.classify_input(event, state)
        }
    }
}

pub fn page(
    mut output: String,
    file: Option<PathBuf>,
    refresh: Option<RefreshCallback>,
) -> Result<()> {
    let editor = EditorCommand::from_env();
    let editor_enabled = !matches!(editor, Ok(None)) && file.is_some();
    let mut pending_message = None;

    loop {
        let editor_requested = Arc::new(AtomicBool::new(false));
        let pager = Pager::new();
        pager.set_text(output.clone())?;
        if let Some(prompt) = pager_prompt(file.as_deref()) {
            pager.set_prompt(prompt)?;
        }
        pager.remove_hook(Hook::PostPagerExit, 1)?;
        if editor_enabled {
            pager.set_input_classifier(Box::new(PagerInputClassifier {
                default: HashedEventRegister::default(),
                editor_requested: editor_requested.clone(),
            }))?;
        }
        if let Some(message) = pending_message.take() {
            pager.send_message(message)?;
        }

        let watcher = match (&file, &refresh) {
            (Some(path), Some(refresh)) => {
                Some(ActiveWatcher::start(path, pager.clone(), refresh.clone())?)
            }
            _ => None,
        };

        minus::dynamic_paging(pager)?;
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
                Ok(refreshed_output) => output = refreshed_output,
                Err(error) => {
                    pending_message = Some(single_line_message(&format!(
                        "Failed to refresh file: {error:#}"
                    )));
                }
            }
        }
    }
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

fn pager_prompt(file: Option<&Path>) -> Option<String> {
    let file_name = file?.file_name()?.to_string_lossy();
    let prompt = single_line_message(&file_name);
    (!prompt.is_empty()).then_some(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use notify::{EventKind, event::CreateKind};
    use std::sync::atomic::AtomicUsize;
    use tempfile::TempDir;

    #[test]
    fn editor_key_supports_english_and_russian_layouts() {
        for character in ['E', 'e', 'У', 'у'] {
            let event = Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));

            assert!(is_editor_key(&event), "character: {character}");
        }
    }

    #[test]
    fn shifted_uppercase_editor_key_is_supported() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT));

        assert!(is_editor_key(&event));
    }

    #[test]
    fn other_keys_do_not_open_editor() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));

        assert!(!is_editor_key(&event));
    }

    #[test]
    fn modified_editor_keys_do_not_open_editor() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL));

        assert!(!is_editor_key(&event));
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
    fn pager_prompt_uses_file_name() {
        let path = Path::new("documents").join("table-wrap.md");

        assert_eq!(pager_prompt(Some(&path)), Some("table-wrap.md".to_string()));
        assert_eq!(pager_prompt(None), None);
    }

    #[test]
    fn active_watcher_refreshes_modified_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("watched.md");
        std::fs::write(&file, "# Before").unwrap();
        let refresh_count = Arc::new(AtomicUsize::new(0));
        let callback_count = refresh_count.clone();
        let refresh = Arc::new(move || {
            callback_count.fetch_add(1, Ordering::SeqCst);
            Ok("# After".to_string())
        });
        let watcher = ActiveWatcher::start(&file, Pager::new(), refresh).unwrap();

        std::fs::write(&file, "# After").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while refresh_count.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }

        drop(watcher);
        assert!(refresh_count.load(Ordering::SeqCst) >= 1);
    }
}
