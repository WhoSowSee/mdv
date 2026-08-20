use super::browser::{BrowserState, FilterState};
use crate::config::Config;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use std::path::PathBuf;

pub(super) enum AppAction {
    None,
    Quit,
    OpenPager(PathBuf),
    OpenEditor(PathBuf),
    Suspend,
}

pub(super) struct App {
    pub(super) config: Config,
    pub(super) browser: BrowserState,
    pub(super) width: u16,
    pub(super) height: u16,
}

impl App {
    pub(super) fn new(root: PathBuf, config: Config, width: u16, height: u16) -> Self {
        Self {
            config,
            browser: BrowserState::new(root, height),
            width,
            height,
        }
    }

    pub(super) fn tick(&mut self) {
        self.browser.poll_discovery();
    }

    pub(super) fn is_loading(&self) -> bool {
        !self.browser.is_loaded()
    }

    pub(super) fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.browser.set_height(height);
    }

    pub(super) fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return AppAction::None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if key.code == KeyCode::Char('c') {
                return AppAction::Quit;
            }
            if key.code == KeyCode::Char('z') {
                return AppAction::Suspend;
            }
            if key.code == KeyCode::Char('f') {
                self.browser.begin_filter();
                return AppAction::None;
            }
        }

        self.handle_browser_key(key)
    }

    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.browser.move_up(),
            MouseEventKind::ScrollDown => self.browser.move_down(),
            _ => {}
        }
    }

    pub(super) fn handle_paste(&mut self, text: &str) {
        if self.browser.filter_state() == FilterState::Editing {
            for character in text.chars().filter(|character| !character.is_control()) {
                self.browser.push_filter_char(character);
            }
        }
    }

    pub(super) fn after_editor(&mut self, result: Result<()>) {
        if let Err(error) = result {
            self.browser
                .add_error(format!("Failed to open editor: {error:#}"));
            self.browser.open_error();
        } else {
            self.browser.refresh();
        }
    }

    pub(super) fn after_pager(&mut self, result: Result<()>) {
        if let Err(error) = result {
            self.browser
                .add_error(format!("Failed to open pager: {error:#}"));
            self.browser.open_error();
        }
    }

    fn handle_browser_key(&mut self, key: KeyEvent) -> AppAction {
        if self.browser.show_error() {
            self.browser.close_error();
            return AppAction::None;
        }
        if self.browser.filter_state() == FilterState::Editing {
            return self.handle_filter_key(key);
        }
        if key.code == KeyCode::Esc && self.browser.show_full_help() {
            self.browser.toggle_help();
            return AppAction::None;
        }

        match key.code {
            KeyCode::Char('q') => return AppAction::Quit,
            KeyCode::Esc => self.browser.cancel_filter(),
            KeyCode::Char('r') => self.browser.refresh(),
            KeyCode::Up | KeyCode::Char('k') => self.browser.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.browser.move_down(),
            KeyCode::Home | KeyCode::Char('g') => self.browser.go_top(),
            KeyCode::End | KeyCode::Char('G') => self.browser.go_bottom(),
            KeyCode::Left | KeyCode::Char('h' | 'b' | 'u') | KeyCode::PageUp => {
                self.browser.page_back()
            }
            KeyCode::Right | KeyCode::Char('l' | 'f' | 'd') | KeyCode::PageDown => {
                self.browser.page_forward()
            }
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('L' | 'H') => {
                self.browser.next_section()
            }
            KeyCode::Char('/') => self.browser.begin_filter(),
            KeyCode::Char('?') => self.browser.toggle_help(),
            KeyCode::Char('!') => self.browser.open_error(),
            KeyCode::Char('e' | 'E' | 'у' | 'У') => {
                if let Some(document) = self.browser.selected_document() {
                    return AppAction::OpenEditor(document.path.clone());
                }
            }
            KeyCode::Enter => return self.open_selected_document(),
            _ => {}
        }
        AppAction::None
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> AppAction {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('j' | 'k'))
        {
            return self.confirm_filter();
        }
        match key.code {
            KeyCode::Esc => self.browser.cancel_filter(),
            KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                return self.confirm_filter();
            }
            KeyCode::Backspace => self.browser.pop_filter_char(),
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.browser.push_filter_char(character);
            }
            _ => {}
        }
        AppAction::None
    }

    fn confirm_filter(&mut self) -> AppAction {
        self.browser.confirm_filter();
        AppAction::None
    }

    fn open_selected_document(&self) -> AppAction {
        self.browser
            .selected_document()
            .map(|document| AppAction::OpenPager(document.path.clone()))
            .unwrap_or(AppAction::None)
    }
}

#[cfg(test)]
mod tests {
    use super::super::discovery::DocumentEntry;
    use super::*;

    #[test]
    fn enter_returns_the_selected_document_for_the_existing_pager() {
        let browser = BrowserState::for_test(vec![DocumentEntry::for_test("README.md")], 24);
        let mut app = App {
            config: Config::default(),
            browser,
            width: 80,
            height: 24,
        };

        let action = app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(matches!(action, AppAction::OpenPager(path) if path.as_os_str() == "README.md"));
    }

    #[test]
    fn control_f_starts_filtering() {
        let browser = BrowserState::for_test(vec![DocumentEntry::for_test("README.md")], 24);
        let mut app = App {
            config: Config::default(),
            browser,
            width: 80,
            height: 24,
        };

        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));

        assert_eq!(app.browser.filter_state(), FilterState::Editing);
    }

    #[test]
    fn escape_closes_full_help_without_clearing_browser_state() {
        let browser = BrowserState::for_test(
            vec![
                DocumentEntry::for_test("README.md"),
                DocumentEntry::for_test("README-RU.md"),
                DocumentEntry::for_test("DOC.md"),
            ],
            24,
        );
        let mut app = App {
            config: Config::default(),
            browser,
            width: 80,
            height: 24,
        };
        app.browser.begin_filter();
        app.browser.set_filter("readme");
        app.browser.confirm_filter();
        assert_eq!(app.browser.filter_state(), FilterState::Applied);
        app.browser.toggle_help();
        assert!(app.browser.show_full_help());

        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert!(!app.browser.show_full_help());
        assert_eq!(app.browser.filter_state(), FilterState::Applied);
        assert_eq!(app.browser.query(), "readme");
    }
}
