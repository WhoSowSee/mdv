//! Runtime pager state.

#![allow(dead_code)]
#[cfg(feature = "search")]
use crate::search::{SearchMode, SearchOpts, next_nth_match};

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
#[cfg(feature = "search")]
use std::collections::BTreeSet;
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

#[cfg(feature = "search")]
#[cfg_attr(docsrs, doc(cfg(feature = "search")))]
#[allow(clippy::module_name_repetitions)]
/// Current search state.
pub struct SearchState {
    /// Active search direction.
    pub search_mode: SearchMode,
    pub(crate) search_term: Option<regex::Regex>,
    pub(crate) search_idx: BTreeSet<usize>,
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
            search_idx: BTreeSet::new(),
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
    pub(crate) selection_anchor: Option<Selection>,
}

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
            cols,
            rows,
            prefix_num: String::new(),
            lines_to_row_map: LinesRowMap::new(),
            follow_output: false,
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

    pub(crate) fn reformat_display(&mut self) -> Result<(), PromptError> {
        let format_result = screen::format_lines_into(
            &mut self.screen.formatted_lines,
            &self.screen.orig_text,
            self.line_numbers,
            self.cols,
            self.screen.line_wrapping,
            #[cfg(feature = "search")]
            self.search_state.search_term.as_ref(),
        );

        #[cfg(feature = "search")]
        {
            self.search_state.search_idx = format_result.append_search_idx;
            self.search_state.search_mark =
                next_nth_match(&self.search_state.search_idx, self.upper_mark, 0).unwrap_or(0);
        }
        self.lines_to_row_map = format_result.lines_to_row_map;
        self.screen.max_line_length = format_result.max_line_length;

        self.screen.unterminated = format_result.num_unterminated;
        self.format_prompt()
    }

    pub(crate) fn format_prompt(&mut self) -> Result<(), PromptError> {
        if let Some(renderer) = &self.prompt_renderer {
            let prompt = renderer(&PromptContext::new(self))?;
            self.displayed_prompt = prompt.render(self.cols);
        } else {
            self.format_default_prompt();
        }

        self.displayed_prompt_panel = self
            .prompt_panel
            .iter()
            .map(|line| line.render(self.cols))
            .collect();
        Ok(())
    }

    #[must_use]
    pub const fn content_rows(&self) -> usize {
        self.rows
            .saturating_sub(self.prompt_panel_rows().saturating_add(1))
    }

    #[must_use]
    pub const fn prompt_panel_rows(&self) -> usize {
        let available = self.rows.saturating_sub(1);
        if self.prompt_panel.len() < available {
            self.prompt_panel.len()
        } else {
            available
        }
    }

    #[must_use]
    pub const fn max_upper_mark(&self) -> usize {
        let content_rows = self.content_rows();
        let available_trailing_rows = content_rows.saturating_sub(1);
        let trailing_rows = if available_trailing_rows < EOF_SCROLL_MARGIN_ROWS {
            available_trailing_rows
        } else {
            EOF_SCROLL_MARGIN_ROWS
        };
        self.screen
            .formatted_lines_count()
            .saturating_add(trailing_rows)
            .saturating_sub(content_rows)
    }

    pub(crate) const fn prompt_row(&self) -> usize {
        self.content_rows()
    }

    fn format_default_prompt(&mut self) {
        const PROMPT_SPEC: &str = "\x1b[2;40;37m";
        const SEARCH_SPEC: &str = "\x1b[30;44m";
        const INPUT_SPEC: &str = "\x1b[30;43m";
        const MSG_SPEC: &str = "\x1b[30;1;41m";
        const RESET: &str = "\x1b[0m";
        const FOLLOW_MODE_SPEC: &str = "\x1b[1m";

        let mut format_string = String::with_capacity(self.cols + (SEARCH_SPEC.len() * 5) + 4);

        #[cfg(feature = "search")]
        let mut search_str = String::new();
        #[cfg(feature = "search")]
        if !self.search_state.search_idx.is_empty() {
            search_str.push(' ');
            search_str.push_str(&(self.search_state.search_mark + 1).to_string());
            search_str.push('/');
            search_str.push_str(&self.search_state.search_idx.len().to_string());
            search_str.push(' ');
        }

        let mut prefix_str = String::new();
        if !self.prefix_num.is_empty() {
            prefix_str.push(' ');
            prefix_str.push_str(&self.prefix_num);
            prefix_str.push(' ');
        }

        let prompt_str = self.message.as_ref().unwrap_or(&self.prompt);

        #[cfg(feature = "search")]
        let search_len = search_str.len();
        #[cfg(not(feature = "search"))]
        let search_len = 0;

        let follow_mode_str: &str = if self.follow_output { "[F]" } else { "" };

        // Prompt width counts Unicode characters rather than UTF-8 bytes.
        let prefix_len = prefix_str.len();
        let extra_space = self.cols.saturating_sub(
            search_len + prefix_len + follow_mode_str.len() + prompt_str.chars().count(),
        );

        let byte_idx = prompt_str
            .char_indices()
            .nth(search_len + prefix_len + follow_mode_str.len());

        let dsp_prompt: &str = if extra_space == 0
            && let Some((idx, _)) = byte_idx
        {
            &prompt_str[..idx]
        } else {
            prompt_str
        };

        if self.message.is_some() {
            format_string.push_str(MSG_SPEC);
        } else {
            format_string.push_str(PROMPT_SPEC);
        }
        format_string.push_str(dsp_prompt);
        format_string.push_str(&" ".repeat(extra_space));

        if prefix_len > 0 {
            format_string.push_str(INPUT_SPEC);
            format_string.push_str(&prefix_str);
        }

        #[cfg(feature = "search")]
        if search_len > 0 {
            format_string.push_str(SEARCH_SPEC);
            format_string.push_str(&search_str);
        }

        if !follow_mode_str.is_empty() {
            format_string.push_str(FOLLOW_MODE_SPEC);
            format_string.push_str(follow_mode_str);
        }

        format_string.push_str(RESET);

        self.displayed_prompt = format_string;
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

    pub(crate) fn selection_from_coordinates(&self, x: u16, y: u16) -> Option<Selection> {
        let writable_rows = self.content_rows();
        let row_count = self.screen.formatted_lines_count();

        if row_count == 0 || usize::from(y) >= writable_rows {
            return None;
        }

        let absolute_row = self
            .upper_mark
            .saturating_add(usize::from(y))
            .min(row_count - 1);
        let raw_row = self.screen.formatted_lines.get(absolute_row)?;
        let prefix_width = self.line_number_padding();
        let (displayed_row, skipped_content_chars) = if self.screen.line_wrapping {
            (Cow::Borrowed(raw_row.as_str()), 0)
        } else {
            self.horizontal_scroll_view(raw_row)
        };
        let visible_row = strip_ansi(&displayed_row);
        let prefix_chars = char_index_at_display_column(&visible_row, prefix_width);
        let col = char_index_at_display_column(&visible_row, usize::from(x))
            .saturating_sub(prefix_chars)
            .saturating_add(skipped_content_chars);

        Some(Selection { absolute_row, col })
    }

    pub(crate) const fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
    }

    pub(crate) fn selection_row_span(&self) -> Option<(usize, usize)> {
        let (start, end) = self.normalized_selection()?;
        Some((start.absolute_row, end.absolute_row))
    }

    /// Omits ANSI and OSC sequences from the active selection.
    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.normalized_selection()?;
        let start_line = self.lines_to_row_map.row_to_line(start.absolute_row)?;
        let end_line = self.lines_to_row_map.row_to_line(end.absolute_row)?;
        let mut lines = self.screen.orig_text.lines().skip(start_line);

        let mut selected = Vec::with_capacity(end_line.saturating_sub(start_line) + 1);
        for line_idx in start_line..=end_line {
            let line = strip_ansi(lines.next()?);
            let line = line.as_ref();
            let line_len = line.chars().count();
            let start_col = if line_idx == start_line {
                self.selection_col_in_line(start, line_idx, line)
                    .min(line_len)
            } else {
                0
            };
            let end_col = if line_idx == end_line {
                let end_char = self
                    .selection_col_in_line(end, line_idx, line)
                    .min(line_len);
                grapheme_end_char_index(line, end_char)
            } else {
                line_len
            };

            selected.push(slice_chars(line, start_col, end_col).to_string());
        }

        Some(selected.join("\n"))
    }

    pub(crate) fn render_rows_for_display(&self, start: usize, end: usize) -> Vec<Cow<'_, str>> {
        (start..end)
            .filter_map(|absolute_row| self.render_row_for_display(absolute_row))
            .collect()
    }

    fn render_row_for_display(&self, absolute_row: usize) -> Option<Cow<'_, str>> {
        let raw_row = self.screen.formatted_lines.get(absolute_row)?;
        let Some((start_col, end_col)) = self.selection_bounds_for_row(absolute_row) else {
            return Some(if self.screen.line_wrapping {
                raw_row.into()
            } else {
                self.horizontal_scroll_view(raw_row).0
            });
        };

        let prefix_width = self.line_number_padding();
        let (row, skipped_chars) = if self.screen.line_wrapping {
            (Cow::Borrowed(raw_row.as_str()), 0)
        } else {
            self.horizontal_scroll_view(raw_row)
        };
        let visible_start = start_col.saturating_sub(skipped_chars);
        let visible_end = end_col.saturating_sub(skipped_chars);
        Some(highlight_visible_range(
            row,
            prefix_width.saturating_add(visible_start),
            prefix_width.saturating_add(visible_end),
        ))
    }

    fn horizontal_scroll_view<'a>(&self, row: &'a str) -> (Cow<'a, str>, usize) {
        let (first_end, second_start, second_end) = display::get_horizontal_scroll_bounds(
            row,
            self.cols,
            self.left_mark,
            self.line_numbers.is_on(),
            self.screen.line_count(),
        );
        let skipped_chars = self.horizontal_scroll_char_offset(row);

        if self.left_mark < row.len() {
            if self.line_numbers.is_on() {
                (
                    format!("{}{}", &row[..first_end], &row[second_start..second_end]).into(),
                    skipped_chars,
                )
            } else {
                (row[second_start..second_end].into(), skipped_chars)
            }
        } else {
            (Cow::Borrowed(""), skipped_chars)
        }
    }

    fn horizontal_scroll_char_offset(&self, row: &str) -> usize {
        let visible = strip_ansi(row);
        let visible_len = visible.chars().count();
        let content_start = self.line_number_padding().min(visible_len);
        let content = slice_chars(&visible, content_start, visible_len);
        let mut byte_offset = self.left_mark.min(content.len());
        while byte_offset < content.len() && !content.is_char_boundary(byte_offset) {
            byte_offset += 1;
        }
        content[..byte_offset].chars().count()
    }

    pub(crate) const fn line_number_padding(&self) -> usize {
        if self.line_numbers.is_on() {
            minus_core::utils::digits(self.screen.line_count()) + LineNumbers::EXTRA_PADDING + 2
        } else {
            0
        }
    }

    fn selection_bounds_for_row(&self, absolute_row: usize) -> Option<(usize, usize)> {
        let (start, end) = self.normalized_selection()?;

        if absolute_row < start.absolute_row || absolute_row > end.absolute_row {
            return None;
        }

        let start_col = if absolute_row == start.absolute_row {
            start.col
        } else {
            0
        };
        let end_col = if absolute_row == end.absolute_row {
            end.col.saturating_add(1)
        } else {
            usize::MAX
        };
        Some((start_col, end_col))
    }

    fn normalized_selection(&self) -> Option<(Selection, Selection)> {
        let s_start = self.selection_anchor?;
        let s_end = self.selection?;

        Some(
            if s_start.absolute_row > s_end.absolute_row
                || (s_start.absolute_row == s_end.absolute_row && s_start.col > s_end.col)
            {
                (s_end, s_start)
            } else {
                (s_start, s_end)
            },
        )
    }

    fn selection_col_in_line(&self, selection: Selection, line_idx: usize, line: &str) -> usize {
        if !self.screen.line_wrapping {
            return selection.col;
        }

        let Some(&line_start_row) = self.lines_to_row_map.get(line_idx) else {
            return selection.col;
        };
        let row_in_line = selection.absolute_row.saturating_sub(line_start_row);
        let cols_avail = self.wrapped_cols_available();
        let wrapped_rows = textwrap::wrap(line, cols_avail.max(1));
        let preceding_chars = wrapped_rows
            .iter()
            .take(row_in_line)
            .map(|row| row.chars().count())
            .sum::<usize>();

        preceding_chars.saturating_add(selection.col)
    }

    const fn wrapped_cols_available(&self) -> usize {
        if self.line_numbers.is_on() {
            self.cols
                .saturating_sub(self.line_number_padding().saturating_add(1))
        } else {
            self.cols
        }
    }

    pub(crate) fn append_str(&mut self, text: &str) -> Result<AppendStyle, PromptError> {
        let old_lc = self.screen.line_count();
        let old_lc_dgts = minus_core::utils::digits(old_lc);
        let mut append_result = self.screen.push_screen_buf(
            text,
            self.line_numbers,
            self.cols.try_into().unwrap(),
            #[cfg(feature = "search")]
            self.search_state.search_term.as_ref(),
        );
        let new_lc = self.screen.line_count();
        let new_lc_dgts = minus_core::utils::digits(new_lc);
        #[cfg(feature = "search")]
        {
            let mut append_search_idx = append_result.append_search_idx;
            self.search_state.search_idx.append(&mut append_search_idx);
        }
        self.lines_to_row_map.append(
            &mut append_result.lines_to_row_map,
            append_result.clean_append,
        );

        if self.line_numbers.is_on() && (new_lc_dgts != old_lc_dgts && old_lc_dgts != 0) {
            self.reformat_display()?;
            return Ok(AppendStyle::FullRedraw);
        }

        let total_rows = self.screen.formatted_lines_count();
        self.format_prompt()?;
        Ok(AppendStyle::PartialUpdate((
            total_rows - append_result.rows_formatted,
            total_rows,
        )))
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
mod tests {
    use super::{PagerState, Selection};
    use crate::selection::highlight_visible_range;
    use crate::{LineNumbers, PromptLine};
    use std::{borrow::Cow, fmt::Write, sync::Arc};

    #[test]
    fn prompt_renderer_receives_stable_context() {
        let mut ps = PagerState::new().unwrap();
        ps.prompt = "base prompt".to_string();
        ps.cols = 42;
        ps.rows = 12;
        ps.upper_mark = 7;
        ps.left_mark = 3;
        ps.screen.orig_text = (0..30).fold(String::new(), |mut text, line| {
            writeln!(text, "line {line}").expect("writing to String cannot fail");
            text
        });
        ps.reformat_display().unwrap();
        ps.prompt_renderer = Some(Arc::new(|context| {
            PromptLine::plain(format!(
                "{}:{}:{}:{}:{}:{}:{}",
                context.prompt(),
                context.columns(),
                context.rows(),
                context.upper_mark(),
                context.left_mark(),
                context.formatted_lines(),
                context.scroll_percentage(),
            ))
        }));

        ps.format_prompt().unwrap();

        assert_eq!(
            ps.displayed_prompt,
            format!("{:<42}\x1b[0m", "base prompt:42:12:7:3:30:35")
        );
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn selected_text_handles_line_numbers_and_unicode_horizontal_scroll() {
        let mut ps = PagerState::new().unwrap();
        ps.line_numbers = LineNumbers::Enabled;
        ps.screen.line_wrapping = false;
        ps.left_mark = "é".len();
        ps.screen.orig_text = "éabcdefghij\nklmnopqrst\nuvwxyz\n".to_string();
        ps.reformat_display().unwrap();

        let padding = ps.line_number_padding() as u16;
        ps.selection_anchor = ps.selection_from_coordinates(padding, 0);
        ps.selection = ps.selection_from_coordinates(padding + 1, 2);

        assert_eq!(
            ps.selected_text().as_deref(),
            Some("abcdefghij\nklmnopqrst\nuvwx")
        );
    }

    #[test]
    fn selected_text_spans_wrapped_rows() {
        let mut ps = PagerState::new().unwrap();
        ps.cols = 6;
        ps.screen.orig_text = "abcdefghi\njklmnop\n".to_string();
        ps.reformat_display().unwrap();
        ps.selection_anchor = Some(Selection {
            absolute_row: 0,
            col: 2,
        });
        ps.selection = Some(Selection {
            absolute_row: 2,
            col: 3,
        });

        assert_eq!(ps.selected_text().as_deref(), Some("cdefghi\njklm"));
    }

    #[test]
    fn selection_ignores_ansi_and_uses_display_width() {
        let mut ps = PagerState::new().unwrap();
        ps.screen.line_wrapping = false;
        ps.screen.orig_text = "\x1b[31ma界b\x1b[0m".to_string();
        ps.reformat_display().unwrap();

        assert_eq!(
            ps.selection_from_coordinates(2, 0),
            Some(Selection {
                absolute_row: 0,
                col: 1,
            })
        );
        assert_eq!(
            ps.selection_from_coordinates(3, 0),
            Some(Selection {
                absolute_row: 0,
                col: 2,
            })
        );
        ps.selection_anchor = ps.selection_from_coordinates(2, 0);
        ps.selection = ps.selection_from_coordinates(3, 0);
        assert_eq!(ps.selected_text().as_deref(), Some("界b"));
    }

    #[test]
    fn selection_preserves_complete_graphemes() {
        let mut ps = PagerState::new().unwrap();
        ps.cols = 10;
        ps.screen.line_wrapping = false;
        ps.screen.orig_text = "e\u{301}x".to_string();
        ps.reformat_display().unwrap();
        ps.selection_anchor = ps.selection_from_coordinates(0, 0);
        ps.selection = ps.selection_anchor;

        assert_eq!(ps.selected_text().as_deref(), Some("e\u{301}"));
        assert!(
            ps.render_rows_for_display(0, 1)[0]
                .contains("\x1b[0;38;2;143;147;162;48;2;31;34;51me\u{301}\x1b[0m")
        );
    }

    #[test]
    fn selection_highlight_overrides_and_restores_sgr_styles() {
        let rendered = highlight_visible_range(
            Cow::Borrowed("\x1b[31mred\x1b[0m plain"),
            0,
            "red plain".chars().count(),
        );

        assert!(rendered.contains("\x1b[0m\x1b[0;38;2;143;147;162;48;2;31;34;51m plain"));
        assert!(!rendered.contains("\x1b[7m"));

        let rendered = highlight_visible_range(Cow::Borrowed("\x1b[31mred plain\x1b[0m"), 0, 3);
        assert!(rendered.contains("red\x1b[0m\x1b[31m plain"));
    }
}
