//! Runtime pager state.

#[cfg(feature = "search")]
use crate::search::{SearchMatch, SearchMode, SearchOpts, highlight_search_matches};

#[cfg(feature = "search")]
use crate::LineNavigation;
use crate::{
    LineNumbers, PromptContext, PromptError, PromptRenderer,
    error::{MinusError, TermError},
    hooks::{Hook, Hooks},
    input::{self, HashedEventRegister},
    minus_core::{
        self, CommandQueue,
        utils::{
            LinesRowMap,
            display::{self, AppendStyle},
        },
    },
    screen::{self, Screen},
};
use crossterm::{terminal, tty::IsTty};
use parking_lot::Mutex;
use std::{
    borrow::Cow,
    collections::hash_map::RandomState,
    convert::TryInto,
    io::stdout,
    sync::{Arc, atomic::AtomicBool},
};

use crate::minus_core::{commands::Command, ev_handler::handle_event};
use crate::selection::{
    char_index_at_display_column, grapheme_end_char_index, highlight_visible_range, strip_ansi,
};
use crossbeam_channel::Receiver;

const EOF_SCROLL_MARGIN_ROWS: usize = 1;

#[cfg(test)]
#[path = "state/styling_tests.rs"]
mod styling_tests;

#[cfg(feature = "search")]
#[cfg_attr(docsrs, doc(cfg(feature = "search")))]
#[allow(clippy::module_name_repetitions)]
/// Current search state.
pub struct SearchState {
    /// Active search direction.
    pub search_mode: SearchMode,
    pub(crate) search_term: Option<regex::Regex>,
    pub(crate) last_search_query: String,
    pub(crate) search_matches: Vec<SearchMatch>,
    pub(crate) search_mark: usize,
    pub(crate) incremental_search_condition:
        Box<dyn Fn(&SearchOpts) -> bool + Send + Sync + 'static>,
}

#[cfg(feature = "search")]
impl Default for SearchState {
    fn default() -> Self {
        let incremental_search_condition = Box::new(|so: &SearchOpts| {
            so.string.len() > 1
                && so
                    .incremental_search_options
                    .as_ref()
                    .unwrap()
                    .screen
                    .line_count()
                    <= 5000
        });
        Self {
            search_mode: SearchMode::Unknown,
            search_term: None,
            last_search_query: String::new(),
            search_matches: Vec::new(),
            search_mark: 0,
            incremental_search_condition,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Selection {
    pub absolute_row: usize,
    pub col: usize,
}

/// Runtime pager state exposed to [`InputClassifier`](input::InputClassifier) implementations.
#[allow(clippy::module_name_repetitions)]
pub struct PagerState {
    #[cfg(feature = "search")]
    pub(crate) layout_renderer: Option<crate::LayoutRenderer>,
    #[cfg(feature = "search")]
    pub(crate) toc: crate::toc::TocState,
    /// Line-number visibility and toggle behavior.
    pub line_numbers: LineNumbers,
    /// Message displayed in the prompt row.
    pub message: Option<String>,
    pub(crate) message_id: Option<usize>,
    /// Index of the first visible row.
    pub upper_mark: usize,
    /// Number of display columns skipped on the left.
    pub left_mark: usize,
    /// Legacy search direction; new code should use [`SearchState::search_mode`].
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, cfg(feature = "search"))]
    pub search_mode: SearchMode,
    /// Terminal height in rows.
    pub rows: usize,
    /// Terminal width in columns.
    pub cols: usize,
    /// Numeric prefix accumulated before a movement command.
    pub prefix_num: String,
    /// Process-wide pager run mode.
    pub running: &'static Mutex<crate::RunMode>,
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, cfg(feature = "search"))]
    pub search_state: SearchState,
    #[cfg(feature = "search")]
    pub(crate) search_prompt: Option<String>,
    #[cfg(feature = "search")]
    pub(crate) line_navigation: Option<LineNavigation>,
    #[cfg(feature = "search")]
    pub(crate) line_navigation_session: Option<crate::line_navigation::LineNavigationSession>,
    pub screen: Screen,
    pub selection: Option<Selection>,
    pub(crate) prompt: String,
    pub(crate) prompt_renderer: Option<PromptRenderer>,
    pub(crate) prompt_panel: Vec<crate::PromptLine>,
    pub(crate) input_classifier: Box<dyn input::InputClassifier + Sync + Send>,
    pub(crate) exit_callbacks: Vec<Box<dyn FnMut() + Send + Sync + 'static>>,
    pub(crate) hooks: Hooks,
    pub(crate) displayed_prompt: String,
    pub(crate) displayed_prompt_panel: Vec<String>,
    pub(crate) show_prompt: bool,
    #[cfg(feature = "static_output")]
    pub(crate) run_no_overflow: bool,
    pub(crate) lines_to_row_map: LinesRowMap,
    pub(crate) follow_output: bool,
    pub(crate) output_styling: bool,
    pub(crate) color_depth: crate::ColorDepth,
    pub(crate) highlight_styles: crate::HighlightStyles,
    pub(crate) selection_anchor: Option<Selection>,
}

mod layout;
mod prompt;
mod selection;

impl PagerState {
    pub(crate) fn new() -> Result<Self, TermError> {
        let (cols, rows) = if cfg!(test) {
            (80, 10)
        } else if stdout().is_tty() {
            let size = terminal::size()?;
            (size.0 as usize, size.1 as usize)
        } else {
            (1, 1)
        };

        let prompt = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("minus"))
            .file_name()
            .map_or_else(
                || std::ffi::OsString::from("minus"),
                std::ffi::OsStr::to_os_string,
            )
            .into_string()
            .unwrap_or_else(|_| String::from("minus"));

        let mut state = Self {
            #[cfg(feature = "search")]
            layout_renderer: None,
            #[cfg(feature = "search")]
            toc: crate::toc::TocState::default(),
            line_numbers: LineNumbers::Disabled,
            upper_mark: 0,
            prompt,
            prompt_renderer: None,
            prompt_panel: Vec::new(),
            running: &minus_core::RUNMODE,
            left_mark: 0,
            input_classifier: Box::<HashedEventRegister<RandomState>>::default(),
            exit_callbacks: Vec::with_capacity(5),
            hooks: Hooks::new(),
            message: None,
            message_id: None,
            screen: Screen::default(),
            selection: None,
            displayed_prompt: String::new(),
            displayed_prompt_panel: Vec::new(),
            show_prompt: true,
            #[cfg(feature = "static_output")]
            run_no_overflow: false,
            #[cfg(feature = "search")]
            search_mode: SearchMode::default(),
            #[cfg(feature = "search")]
            search_state: SearchState::default(),
            #[cfg(feature = "search")]
            search_prompt: None,
            #[cfg(feature = "search")]
            line_navigation: None,
            #[cfg(feature = "search")]
            line_navigation_session: None,
            cols,
            rows,
            prefix_num: String::new(),
            lines_to_row_map: LinesRowMap::new(),
            follow_output: false,
            output_styling: true,
            color_depth: crate::ColorDepth::TrueColor,
            highlight_styles: crate::HighlightStyles::default(),
            selection_anchor: None,
        };

        state.hooks.add_callback(
            Hook::PostPagerExit,
            1,
            Box::new(|_| {
                std::process::exit(0);
            }),
        );

        state.format_default_prompt();
        Ok(state)
    }

    pub(crate) fn generate_initial_state(rx: &Receiver<Command>) -> Result<Self, MinusError> {
        let mut ps = Self::new()?;
        let mut command_queue = CommandQueue::new_zero();
        for ev in rx.try_iter() {
            handle_event(
                ev,
                &mut ps,
                &mut command_queue,
                &Arc::new(AtomicBool::new(false)),
            )?;
        }
        Ok(ps)
    }

    /// Returns whether search highlights are currently active.
    #[must_use]
    pub const fn search_is_active(&self) -> bool {
        #[cfg(feature = "search")]
        {
            self.search_state.search_term.is_some()
        }
        #[cfg(not(feature = "search"))]
        {
            false
        }
    }

    pub(crate) fn run_hooks(&mut self, hook: crate::hooks::Hook) {
        let mut hooks = std::mem::take(&mut self.hooks);
        hooks.run_hooks(hook, self);
        self.hooks = hooks;
    }

    pub(crate) fn exit(&mut self) {
        for func in &mut self.exit_callbacks {
            func();
        }
    }
}

fn slice_chars(line: &str, start: usize, end: usize) -> &str {
    let mut indices = line
        .char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(line.len()));
    let start_byte = indices.nth(start).unwrap_or(line.len());
    let end_byte = indices
        .nth(end.saturating_sub(start + 1))
        .unwrap_or(line.len());

    &line[start_byte..end_byte]
}

#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;

#[cfg(all(test, feature = "search"))]
#[path = "state/search_tests.rs"]
mod search_tests;
