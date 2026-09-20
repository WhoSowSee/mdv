use crate::editor::EditorCommand;
use anyhow::{Context, Result, anyhow};
use minus::hooks::Hook;
use minus::input::{HashedEventRegister, InputClassifier, InputEvent};
use minus::{LineNavigation, LineNumbers, Pager, PagerState, PromptLine};
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::hash_map::RandomState;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

mod command;
mod document;
mod footer;
mod help;
mod input;
mod interrupt;
mod operations;
mod page;
mod rendering;
mod watcher;

pub(crate) use command::PagerBackend;
pub(super) use document::{PagerDocument, PagerScreen, RefreshCallback};
pub(crate) use rendering::{RenderedOutput, render_terminal_document};

use document::{PagerContent, PagerDisplay, PagerLineNumberMode, PagerLineNumberViews};
use footer::PagerFooter;
use help::build_help_panel;
use input::PagerInputClassifier;
use operations::{
    apply_refreshed_document, copy_document_contents, cycle_line_number_mode, replace_document,
    report_operation_result, single_line_message,
};
use watcher::ActiveWatcher;

const STATUS_MESSAGE_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) fn page(
    document: PagerDocument,
    file: Option<PathBuf>,
    refresh: Option<RefreshCallback>,
    screen: PagerScreen,
    backend: &PagerBackend,
) -> Result<()> {
    match backend {
        PagerBackend::Builtin => page::page(document, file, refresh, screen),
        PagerBackend::External(command) => {
            let output = document.into_output();
            command.page(&output)
        }
    }
}

#[cfg(test)]
use input::{
    HelpInputAction, help_input_action, is_copy_key, is_editor_key, is_help_key, is_reload_key,
};
#[cfg(test)]
use operations::clipboard_text;
#[cfg(test)]
use watcher::{comparable_path, event_targets_file};

#[cfg(test)]
mod tests;
