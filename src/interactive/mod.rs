mod app;
pub(crate) mod browser;
pub(crate) mod discovery;
pub(crate) mod screen;

use crate::config::Config;
use crate::editor::EditorCommand;
use crate::pager::{self, PagerDocument, PagerScreen, RefreshCallback};
use anyhow::{Result, anyhow, ensure};
use app::{App, AppAction};
use crossterm::event::{self, Event};
use screen::TerminalSession;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum InteractiveTarget {
    Directory(PathBuf),
    File(PathBuf),
    Stdin,
}

pub(crate) fn select_interactive_target(
    filename: Option<&str>,
    requested: bool,
    pager_requested: bool,
    stdin_is_terminal: bool,
) -> Result<Option<InteractiveTarget>> {
    if pager_requested {
        return Ok(None);
    }

    let target = match filename {
        Some("-") if requested => Some(InteractiveTarget::Stdin),
        Some("-") => None,
        Some(filename) => {
            let path = PathBuf::from(filename);
            match std::fs::metadata(&path) {
                Ok(metadata) if metadata.is_dir() => {
                    Some(InteractiveTarget::Directory(path.canonicalize()?))
                }
                Ok(_) if requested => Some(InteractiveTarget::File(path.canonicalize()?)),
                Err(_) if requested => Some(InteractiveTarget::File(path)),
                Ok(_) | Err(_) => None,
            }
        }
        None if stdin_is_terminal => Some(InteractiveTarget::Directory(std::env::current_dir()?)),
        None if requested => Some(InteractiveTarget::Stdin),
        None => None,
    };

    Ok(target)
}

pub(crate) fn run(target: InteractiveTarget, config: Config) -> Result<()> {
    ensure!(
        std::io::stdout().is_terminal(),
        "interactive mode requires a terminal"
    );
    let root = match target {
        InteractiveTarget::Directory(root) => root,
        InteractiveTarget::File(path) => {
            return open_file_in_pager(path, &config, PagerScreen::Alternate);
        }
        InteractiveTarget::Stdin => {
            let mut source = String::new();
            std::io::stdin().read_to_string(&mut source)?;
            crate::strip_leading_bom(&mut source);
            return open_source_in_pager(source, &config);
        }
    };
    let (width, height) = crossterm::terminal::size()?;
    let mut app = App::new(root, config.clone(), width, height);
    let mut terminal = TerminalSession::enter()?;

    loop {
        app.tick();
        terminal.draw(&app)?;
        if !event::poll(Duration::from_millis(80))? {
            continue;
        }
        let action = match event::read()? {
            Event::Key(key) => app.handle_key(key),
            Event::Mouse(mouse) => {
                app.handle_mouse(mouse);
                AppAction::None
            }
            Event::Resize(width, height) => {
                app.resize(width, height);
                AppAction::None
            }
            Event::Paste(text) => {
                app.handle_paste(&text);
                AppAction::None
            }
            Event::FocusGained | Event::FocusLost => AppAction::None,
        };

        match action {
            AppAction::None => {}
            AppAction::Quit => return Ok(()),
            AppAction::OpenPager(path) => {
                terminal.pause_for_pager()?;
                let result = open_file_in_pager(path, &config, PagerScreen::InPlace);
                terminal.resume_after_pager()?;
                app.after_pager(result);
            }
            AppAction::OpenEditor(path) => {
                terminal.suspend()?;
                let result = match EditorCommand::from_env() {
                    Ok(Some(editor)) => editor.open(&path).map_err(anyhow::Error::from),
                    Ok(None) => Err(anyhow!("MDV_EDITOR or EDITOR is not set")),
                    Err(error) => Err(error),
                };
                terminal.resume()?;
                app.after_editor(result);
            }
            AppAction::Suspend => suspend_process(&mut terminal)?,
        }
    }
}

fn open_file_in_pager(path: PathBuf, config: &Config, screen: PagerScreen) -> Result<()> {
    let document = crate::render_document_file(&path, config, false, false, None)?;
    let refresh_path = path.clone();
    let refresh_config = config.clone();
    let refresh = Arc::new(move || {
        crate::render_document_file(&refresh_path, &refresh_config, false, false, None)
    }) as RefreshCallback;
    pager::page(document, Some(path), Some(refresh), screen)
}

fn open_source_in_pager(source: String, config: &Config) -> Result<()> {
    let output = crate::render_document(&source, config, false, false, None, true)?;
    pager::page(
        PagerDocument::new(output, source),
        None,
        None,
        PagerScreen::Alternate,
    )
}

#[cfg(unix)]
fn suspend_process(terminal: &mut TerminalSession) -> Result<()> {
    terminal.suspend()?;
    // SAFETY: SIGTSTP targets the current process and resumes through the controlling shell.
    let result = unsafe { libc::raise(libc::SIGTSTP) };
    terminal.resume()?;
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(windows)]
fn suspend_process(_terminal: &mut TerminalSession) -> Result<()> {
    Ok(())
}
