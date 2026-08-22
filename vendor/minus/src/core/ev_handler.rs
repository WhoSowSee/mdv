use std::io::Write;
use std::sync::{Arc, atomic::AtomicBool};
#[cfg(feature = "search")]
use std::time::Duration;

#[cfg(feature = "search")]
use parking_lot::{Condvar, Mutex};

use super::CommandQueue;
use super::commands::{Command, IoCommand};
use super::utils::display::{self, AppendStyle};
use crate::ExitStrategy;
#[cfg(feature = "search")]
use crate::{Pager, search};
use crate::{PagerState, PromptError, error::MinusError, hooks::Hook, input::InputEvent};

#[cfg(feature = "search")]
const NO_SEARCH_MATCH_MESSAGE: &str = "No matches found";
#[cfg(feature = "search")]
const NO_SEARCH_MATCH_DURATION: Duration = Duration::from_secs(2);

#[cfg_attr(not(feature = "search"), allow(unused_mut))]
#[allow(clippy::too_many_lines)]
// Deprecated input variants remain accepted for compatibility.
#[allow(deprecated)]
pub fn handle_event(
    ev: Command,
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
    is_exited: &Arc<AtomicBool>,
) -> Result<(), PromptError> {
    match ev {
        Command::SetData(text) => {
            p.screen.orig_text = text;
            p.screen.line_count = p.screen.orig_text.lines().count();
            p.reformat_display()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        Command::UserInput(InputEvent::Exit) => {
            p.run_hooks(Hook::PrePagerExit);
            p.exit();
            is_exited.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Command::UserInput(InputEvent::UpdateUpperMark(um)) => {
            command_queue.push_back(Command::Io(IoCommand::SetUpperMark(um)));
        }
        Command::UserInput(InputEvent::UpdateLeftMark(lm)) if !p.screen.line_wrapping => {
            let max_scrollable = p
                .screen
                .get_max_line_length()
                .saturating_add(p.line_number_padding());
            if lm.saturating_add(p.cols) > max_scrollable && lm > p.left_mark {
                return Ok(());
            }
            p.left_mark = lm;
            p.format_prompt()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        Command::UserInput(InputEvent::StartSelection { x, y }) => {
            if let Some(selection) = p.selection_from_coordinates(x, y) {
                let previous_span = p.selection_row_span();
                p.selection_anchor = Some(selection);
                p.selection = Some(selection);
                queue_selection_redraw(command_queue, previous_span, p.selection_row_span());
            }
        }
        Command::UserInput(InputEvent::UpdateSelection { x, y }) => {
            if p.selection_anchor.is_none() {
                return Ok(());
            }

            let writable_rows = p.content_rows();
            if writable_rows == 0 {
                return Ok(());
            }

            let row_count = p.screen.formatted_lines_count();
            let max_upper_mark = row_count.saturating_sub(writable_rows);
            let previous_span = p.selection_row_span();
            let mut scrolled = false;
            let mut selection_y = usize::from(y);

            if y == 0 {
                let next_upper_mark = p.upper_mark.saturating_sub(1);
                if next_upper_mark != p.upper_mark {
                    p.upper_mark = next_upper_mark;
                    scrolled = true;
                }
                selection_y = 0;
            } else if selection_y >= writable_rows {
                let next_upper_mark = p.upper_mark.saturating_add(1).min(max_upper_mark);
                if next_upper_mark != p.upper_mark {
                    p.upper_mark = next_upper_mark;
                    scrolled = true;
                }
                selection_y = writable_rows.saturating_sub(1);
            }

            #[allow(clippy::cast_possible_truncation)]
            let selection_y = selection_y as u16;
            let selection_changed = if let Some(selection) =
                p.selection_from_coordinates(x, selection_y)
                && p.selection != Some(selection)
            {
                p.selection = Some(selection);
                true
            } else {
                false
            };

            if scrolled {
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
            } else if selection_changed {
                queue_selection_redraw(command_queue, previous_span, p.selection_row_span());
            }
        }
        Command::UserInput(InputEvent::ClearSelection) => {
            if p.selection.is_some() || p.selection_anchor.is_some() {
                let previous_span = p.selection_row_span();
                p.clear_selection();
                queue_selection_redraw(command_queue, previous_span, None);
            }
        }

        #[cfg(feature = "clipboard")]
        Command::UserInput(InputEvent::CopySelection) => {
            copy_selection(p);
            if p.selection.is_some() || p.selection_anchor.is_some() {
                let previous_span = p.selection_row_span();
                p.clear_selection();
                queue_selection_redraw(command_queue, previous_span, None);
            }
        }
        #[cfg(feature = "clipboard")]
        Command::UserInput(InputEvent::FinalizeSelection) => copy_selection(p),
        Command::UserInput(InputEvent::RestorePrompt) => {
            p.message = None;
            p.message_id = None;
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::UserInput(InputEvent::UpdateTermArea(c, r)) => {
            p.rows = r;
            p.cols = c;
            p.reformat_display()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        Command::UserInput(InputEvent::UpdateLineNumber(l)) => {
            p.line_numbers = l;
            p.reformat_display()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        Command::UserInput(InputEvent::Number(n)) => {
            p.prefix_num.push(n);
            queue_prompt_redraw(p, command_queue)?;
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::Search(m)) => {
            if p.message_id.take().is_some() {
                p.message = None;
                queue_prompt_redraw(p, command_queue)?;
            }
            p.search_mode = m;
            p.search_state.search_mode = m;
            p.search_state.search_mark = 0;
            command_queue.push_back(Command::Io(IoCommand::FetchSearchQuery));
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::CancelSearch) => {
            deactivate_search(p)?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::NextMatch | InputEvent::MoveToNextMatch(1))
            if p.search_state.search_term.is_some() =>
        {
            move_to_next_search_match(p, command_queue, 1)?;
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::PrevMatch | InputEvent::MoveToPrevMatch(1))
            if p.search_state.search_term.is_some() =>
        {
            move_to_previous_search_match(p, command_queue, 1)?;
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::MoveToNextMatch(n))
            if p.search_state.search_term.is_some() =>
        {
            move_to_next_search_match(p, command_queue, n)?;
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::MoveToPrevMatch(n))
            if p.search_state.search_term.is_some() =>
        {
            move_to_previous_search_match(p, command_queue, n)?;
        }

        Command::UserInput(InputEvent::HorizontalScroll(val)) => {
            p.screen.line_wrapping = val;
            p.reformat_display()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }

        Command::AppendData(text) => {
            let prev_unterminated = p.screen.unterminated;
            let prev_fmt_lines_count = p.screen.formatted_lines_count();
            let append_style = p.append_str(text.as_str())?;

            if append_style == AppendStyle::FullRedraw {
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
                return Ok(());
            }

            command_queue.push_back(Command::Io(IoCommand::DrawAppendedText(
                prev_unterminated,
                prev_fmt_lines_count,
                append_style,
            )));

            if p.follow_output {
                command_queue.push_back(Command::Io(IoCommand::SetUpperMark(
                    p.screen.formatted_lines_count(),
                )));
            }
        }

        Command::SetPrompt(text) => {
            p.prompt = text;
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::SendMessage(text) => {
            p.message = Some(text);
            p.message_id = None;
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::SetTimedMessage { text, id } => {
            p.message = Some(text);
            p.message_id = Some(id);
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::ClearMessage(id) if p.message_id == Some(id) => {
            p.message = None;
            p.message_id = None;
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::ClearMessage(_) => {}
        Command::SetPromptRenderer(renderer) => {
            p.prompt_renderer = renderer;
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::SetPromptPanel(panel) => {
            let was_at_bottom = p.upper_mark >= p.max_upper_mark();
            p.prompt_panel = panel;
            let max_upper_mark = p.max_upper_mark();
            if was_at_bottom || p.upper_mark > max_upper_mark {
                p.upper_mark = max_upper_mark;
            }
            p.format_prompt()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        #[cfg(feature = "search")]
        Command::SetSearchPrompt(prompt) => p.search_prompt = prompt,
        Command::SetLineNumbers(ln) => {
            p.line_numbers = ln;
            p.reformat_display()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        Command::SetExitStrategy(es) => {
            p.hooks.remove_callback(Hook::PostPagerExit, 1);
            if es == ExitStrategy::ProcessQuit {
                p.hooks.add_callback(
                    Hook::PostPagerExit,
                    1,
                    Box::new(|_| {
                        std::process::exit(1);
                    }),
                );
            } else {
                p.hooks
                    .add_callback(Hook::PostPagerExit, 1, Box::new(|_| {}));
            }
        }
        Command::LineWrapping(lw) => {
            p.screen.line_wrapping = lw;
            p.reformat_display()?;
        }
        #[cfg(feature = "static_output")]
        Command::SetRunNoOverflow(val) => p.run_no_overflow = val,
        #[cfg(feature = "search")]
        Command::IncrementalSearchCondition(cb) => p.search_state.incremental_search_condition = cb,
        Command::SetInputClassifier(clf) => p.input_classifier = clf,
        Command::AddExitCallback(cb) => p.exit_callbacks.push(cb),
        Command::AddHook(hook, id, cb) => p.hooks.add_callback(hook, id, cb),
        Command::RemoveHook(hook, id) => {
            p.hooks.remove_callback(hook, id);
        }
        Command::ShowPrompt(show) => p.show_prompt = show,
        Command::FollowOutput(follow_output)
        | Command::UserInput(InputEvent::FollowOutput(follow_output)) => {
            p.follow_output = follow_output;
            command_queue.push_back(Command::UserInput(InputEvent::UpdateUpperMark(
                p.screen.formatted_lines_count(),
            )));
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::UserInput(_) => {}
        Command::Io(_) => unreachable!(),
    }
    Ok(())
}

fn queue_prompt_redraw(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
) -> Result<(), PromptError> {
    p.format_prompt()?;
    command_queue.push_back(Command::Io(IoCommand::RedrawPrompt));
    Ok(())
}

#[cfg(feature = "search")]
fn queue_current_search_match(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
) -> Result<(), PromptError> {
    let Some(search_match) = p
        .search_state
        .search_matches
        .get(p.search_state.search_mark)
    else {
        return Ok(());
    };
    let viewport_rows = p.content_rows().max(1);
    let viewport_end = p.upper_mark.saturating_add(viewport_rows);
    let next_upper_mark = if search_match.row < p.upper_mark {
        search_match.row
    } else if search_match.row >= viewport_end {
        search_match
            .row
            .saturating_add(1)
            .saturating_sub(viewport_rows)
    } else {
        p.upper_mark
    };
    if next_upper_mark == p.upper_mark {
        command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
    } else {
        command_queue.push_back(Command::Io(IoCommand::SetUpperMark(next_upper_mark)));
    }
    queue_prompt_redraw(p, command_queue)
}

#[cfg(feature = "search")]
fn move_to_next_search_match(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
    count: usize,
) -> Result<(), PromptError> {
    let total = p.search_state.search_matches.len();
    if total == 0 {
        return Ok(());
    }
    p.search_state.search_mark = (p.search_state.search_mark + count % total) % total;
    queue_current_search_match(p, command_queue)
}

#[cfg(feature = "search")]
fn move_to_previous_search_match(
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
    count: usize,
) -> Result<(), PromptError> {
    if p.search_state.search_matches.is_empty() {
        return Ok(());
    }
    p.search_state.search_mark = p.search_state.search_mark.saturating_sub(count);
    queue_current_search_match(p, command_queue)
}

fn queue_selection_redraw(
    command_queue: &mut CommandQueue,
    previous: Option<(usize, usize)>,
    current: Option<(usize, usize)>,
) {
    let span = match (previous, current) {
        (Some(previous), Some(current)) => {
            Some((previous.0.min(current.0), previous.1.max(current.1)))
        }
        (Some(span), None) | (None, Some(span)) => Some(span),
        (None, None) => None,
    };
    if let Some((start, end)) = span {
        command_queue.push_back(Command::Io(IoCommand::RedrawSelection(start, end)));
    }
}

#[cfg(feature = "clipboard")]
fn copy_selection(p: &PagerState) {
    if let Some(text) = p.selected_text()
        && let Ok(mut clipboard) = arboard::Clipboard::new()
    {
        let _ = clipboard.set_text(text);
    }
}

#[cfg(feature = "search")]
fn set_search_position(p: &mut PagerState, pager: &Pager) -> Result<(), MinusError> {
    let Some(upper_mark) = p
        .search_state
        .search_matches
        .get(p.search_state.search_mark)
        .map(|search_match| search_match.row)
    else {
        pager.send_message_for(NO_SEARCH_MATCH_MESSAGE, NO_SEARCH_MATCH_DURATION)?;
        return Ok(());
    };
    p.upper_mark = upper_mark;
    Ok(())
}

#[cfg(feature = "search")]
fn deactivate_search(p: &mut PagerState) -> Result<(), PromptError> {
    p.search_mode = search::SearchMode::Unknown;
    p.search_state.search_mode = search::SearchMode::Unknown;
    p.search_state.search_term = None;
    p.search_state.search_mark = 0;
    p.reformat_display()?;
    Ok(())
}

#[cfg(feature = "search")]
fn apply_search_result(
    p: &mut PagerState,
    pager: &Pager,
    command_queue: &mut CommandQueue,
    search_result: search::FetchInputResult,
) -> Result<(), MinusError> {
    let search::FetchInputResult {
        string,
        compiled_regex,
        input_status,
        preview_upper_mark,
    } = search_result;

    if string.is_empty() {
        p.search_state.last_search_query.clear();
        deactivate_search(p)?;
        command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        return Ok(());
    }

    if input_status == search::InputStatus::Cancelled {
        p.search_state.last_search_query = string;
        if let Some(upper_mark) = preview_upper_mark {
            p.upper_mark = upper_mark;
        }
        command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        return Ok(());
    }

    let Some(search_term) = compiled_regex.or_else(|| regex::Regex::new(&string).ok()) else {
        p.search_state.last_search_query = string;
        command_queue.push_back(Command::SendMessage(
            "Invalid regular expression. Press Enter".to_string(),
        ));
        return Ok(());
    };
    p.search_state.last_search_query = string;
    p.search_state.search_term = Some(search_term);

    p.reformat_display()?;
    set_search_position(p, pager)?;
    command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
    command_queue.push_back(Command::Io(IoCommand::RedrawPrompt));
    Ok(())
}

#[cfg_attr(
    not(feature = "search"),
    allow(unused_variables),
    allow(clippy::needless_pass_by_ref_mut)
)]
pub fn handle_io_command(
    internal_command: IoCommand,
    mut out: &mut impl Write,
    p: &mut PagerState,
    command_queue: &mut CommandQueue,
    #[cfg(feature = "search")] pager: &Pager,
    #[cfg(feature = "search")] user_input_active: &Arc<(Mutex<bool>, Condvar)>,
) -> Result<(), MinusError> {
    if p.running.lock().is_uninitialized() {
        return Ok(());
    }
    match internal_command {
        IoCommand::RedrawPrompt => {
            display::write_prompt_view(out, p)?;
        }
        IoCommand::RedrawDisplay => {
            display::draw_full(&mut out, p)?;
        }
        IoCommand::RedrawSelection(start, end) => {
            display::draw_selection_rows(&mut out, p, start, end)?;
        }
        IoCommand::SetUpperMark(mut um) => {
            display::draw_for_change(out, p, &mut um)?;
            let line_count = p.screen.formatted_lines_count();
            if um >= line_count.saturating_sub(p.content_rows()) && line_count > p.content_rows() {
                p.run_hooks(Hook::EofReached);
            }
            p.upper_mark = um;
        }
        IoCommand::DrawAppendedText(prev_unterminated, prev_fmt_lines_count, append_style) => {
            let AppendStyle::PartialUpdate(bounds) = append_style else {
                unreachable!();
            };
            let fmt_lines = p.render_rows_for_display(bounds.0, bounds.1);
            display::draw_append_text(
                out,
                p.rows.saturating_sub(p.prompt_panel_rows()),
                prev_unterminated,
                prev_fmt_lines_count,
                &fmt_lines,
            )?;
            if p.show_prompt {
                display::write_prompt_view(out, p)?;
            }
        }
        #[cfg(feature = "search")]
        IoCommand::FetchSearchQuery => {
            // Suspend the general reader while the search loop owns terminal input.
            let (lock, cvar) = (&user_input_active.0, &user_input_active.1);
            let mut active = lock.lock();
            *active = false;
            drop(active);
            cvar.notify_one();
            let search_result = search::fetch_input(&mut out, p)?;
            let mut active = lock.lock();
            *active = true;
            drop(active);
            cvar.notify_one();

            apply_search_result(p, pager, command_queue, search_result)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::commands::{Command, IoCommand};
    use super::handle_event;
    use crate::{
        Pager, PagerState, PromptLine, PromptSpan, PromptStyle, input::InputEvent,
        minus_core::CommandQueue, state::Selection,
    };
    use std::fmt::Write;
    use std::sync::{Arc, atomic::AtomicBool};

    const TEST_STR: &str = "This is some sample text";

    #[cfg(feature = "search")]
    #[allow(clippy::trivial_regex)]
    fn pager_with_active_search() -> PagerState {
        let mut ps = PagerState::new().unwrap();
        ps.screen.orig_text = "pager\nother".to_string();
        ps.search_mode = crate::search::SearchMode::Forward;
        ps.search_state.search_mode = crate::search::SearchMode::Forward;
        ps.search_state.last_search_query = "pager".to_string();
        ps.search_state.search_term = Some(regex::Regex::new("pager").unwrap());
        ps.reformat_display().unwrap();
        ps
    }

    #[cfg(feature = "search")]
    #[test]
    fn cancel_search_clears_highlights_but_keeps_the_query_for_reuse() {
        let mut ps = pager_with_active_search();
        let mut command_queue = CommandQueue::new_zero();
        let is_exited = Arc::new(AtomicBool::new(false));
        assert!(!ps.search_state.search_matches.is_empty());

        handle_event(
            Command::UserInput(InputEvent::CancelSearch),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert!(ps.search_state.search_term.is_none());
        assert!(ps.search_state.search_matches.is_empty());
        assert_eq!(ps.search_mode, crate::search::SearchMode::Unknown);
        assert_eq!(
            ps.search_state.search_mode,
            crate::search::SearchMode::Unknown
        );
        assert!(!is_exited.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawDisplay))
        );

        handle_event(
            Command::UserInput(InputEvent::Search(crate::search::SearchMode::Forward)),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();
        let search_opts = crate::search::SearchOpts::from(&ps);

        assert_eq!(search_opts.string, "pager");
    }

    #[cfg(feature = "search")]
    #[test]
    fn manually_cleared_search_input_forgets_the_saved_query() {
        let mut ps = pager_with_active_search();
        let pager = Pager::new();
        let mut command_queue = CommandQueue::new_zero();

        super::apply_search_result(
            &mut ps,
            &pager,
            &mut command_queue,
            crate::search::FetchInputResult {
                string: String::new(),
                compiled_regex: None,
                input_status: crate::search::InputStatus::Cancelled,
                preview_upper_mark: None,
            },
        )
        .unwrap();

        assert!(ps.search_state.last_search_query.is_empty());
        assert!(ps.search_state.search_term.is_none());
        assert!(ps.search_state.search_matches.is_empty());
        assert_eq!(ps.search_mode, crate::search::SearchMode::Unknown);
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawDisplay))
        );

        ps.search_state.search_mode = crate::search::SearchMode::Forward;
        assert!(crate::search::SearchOpts::from(&ps).string.is_empty());
    }

    #[cfg(feature = "search")]
    #[test]
    fn cancelled_search_input_preserves_the_new_draft() {
        let mut ps = pager_with_active_search();
        let pager = Pager::new();
        let mut command_queue = CommandQueue::new_zero();

        super::apply_search_result(
            &mut ps,
            &pager,
            &mut command_queue,
            crate::search::FetchInputResult {
                string: "[draft".to_string(),
                compiled_regex: None,
                input_status: crate::search::InputStatus::Cancelled,
                preview_upper_mark: None,
            },
        )
        .unwrap();

        assert_eq!(
            ps.search_state
                .search_term
                .as_ref()
                .map(regex::Regex::as_str),
            Some("pager")
        );
        assert_eq!(crate::search::SearchOpts::from(&ps).string, "[draft");
    }

    #[cfg(feature = "search")]
    #[test]
    fn cancelled_search_input_keeps_incremental_preview_position() {
        let mut ps = pager_with_active_search();
        let pager = Pager::new();
        let mut command_queue = CommandQueue::new_zero();

        super::apply_search_result(
            &mut ps,
            &pager,
            &mut command_queue,
            crate::search::FetchInputResult {
                string: "pager".to_string(),
                compiled_regex: None,
                input_status: crate::search::InputStatus::Cancelled,
                preview_upper_mark: Some(1),
            },
        )
        .unwrap();

        assert_eq!(ps.upper_mark, 1);
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawDisplay))
        );
    }

    #[cfg(feature = "search")]
    #[test]
    fn empty_search_notification_is_dismissed_by_next_search() {
        let mut ps = PagerState::new().unwrap();
        let pager = Pager::new();
        let mut command_queue = CommandQueue::new_zero();
        let is_exited = Arc::new(AtomicBool::new(false));
        ps.screen.orig_text = TEST_STR.to_string();
        ps.search_state.search_term = Some(regex::Regex::new(r"dasdas\s+das").unwrap());
        ps.reformat_display().unwrap();
        ps.upper_mark = 3;

        assert!(ps.search_state.search_matches.is_empty());
        super::set_search_position(&mut ps, &pager).unwrap();
        assert_eq!(ps.upper_mark, 3);
        assert_eq!(
            super::NO_SEARCH_MATCH_DURATION,
            std::time::Duration::from_secs(2)
        );
        let notification = pager.rx.try_recv().unwrap();
        assert_eq!(
            notification,
            Command::SetTimedMessage {
                text: "No matches found".to_string(),
                id: 1,
            }
        );

        handle_event(notification, &mut ps, &mut command_queue, &is_exited).unwrap();
        assert_eq!(ps.message.as_deref(), Some("No matches found"));
        assert_eq!(ps.message_id, Some(1));
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawPrompt))
        );

        handle_event(
            Command::UserInput(InputEvent::Search(crate::search::SearchMode::Forward)),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.message, None);
        assert_eq!(ps.message_id, None);
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawPrompt))
        );
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::FetchSearchQuery))
        );
    }

    #[test]
    fn prompt_renderer_tracks_prompt_updates_and_horizontal_scroll() {
        let mut ps = PagerState::new().unwrap();
        ps.cols = 5;
        ps.screen.line_wrapping = false;
        ps.screen.orig_text = "0123456789".to_string();
        ps.reformat_display().unwrap();
        let mut command_queue = CommandQueue::new_zero();
        let is_exited = Arc::new(AtomicBool::new(false));

        handle_event(
            Command::SetPromptRenderer(Some(Arc::new(|context| {
                Ok(PromptLine::new()
                    .left(PromptSpan::new(context.prompt(), PromptStyle::default())?)
                    .right(PromptSpan::new(
                        context.left_mark().to_string(),
                        PromptStyle::default(),
                    )?)
                    .truncation_indicator(PromptSpan::new("…", PromptStyle::default())?))
            }))),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();
        handle_event(
            Command::SetPrompt("updated".to_string()),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();
        handle_event(
            Command::UserInput(InputEvent::UpdateLeftMark(1)),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.displayed_prompt, "upd…1\x1b[0m");

        ps.cols = 80;
        handle_event(
            Command::SetPromptRenderer(None),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();
        assert!(ps.displayed_prompt.contains("updated"));
    }

    #[test]
    fn prompt_panel_reserves_rows_and_preserves_bottom_position() {
        let mut ps = PagerState::new().unwrap();
        ps.rows = 10;
        ps.screen.orig_text = (0..30)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        ps.reformat_display().unwrap();
        ps.upper_mark = ps.max_upper_mark();
        let mut command_queue = CommandQueue::new_zero();
        let is_exited = Arc::new(AtomicBool::new(false));
        let panel = vec![
            PromptLine::plain("help one").unwrap(),
            PromptLine::plain("help two").unwrap(),
        ];

        handle_event(
            Command::SetPromptPanel(panel),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.content_rows(), 7);
        assert_eq!(ps.upper_mark, 24);
        assert_eq!(ps.displayed_prompt_panel.len(), 2);
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawDisplay))
        );

        handle_event(
            Command::SetPromptPanel(Vec::new()),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.content_rows(), 9);
        assert_eq!(ps.upper_mark, 22);
    }

    #[test]
    #[cfg(any(feature = "dynamic_output", feature = "static_output"))]
    fn set_data() {
        let mut ps = PagerState::new().unwrap();
        let ev = Command::SetData(TEST_STR.to_string());
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            ev,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();

        assert_eq!(ps.screen.formatted_lines, vec![TEST_STR.to_string()]);
    }

    #[test]
    fn append_str() {
        let mut ps = PagerState::new().unwrap();
        let ev1 = Command::AppendData(format!("{TEST_STR}\n"));
        let ev2 = Command::AppendData(TEST_STR.to_string());
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            ev1,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        handle_event(
            ev2,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(
            ps.screen.formatted_lines,
            vec![TEST_STR.to_string(), TEST_STR.to_string()]
        );
    }

    #[test]
    #[cfg(any(feature = "dynamic_output", feature = "static_output"))]
    fn set_prompt() {
        let mut ps = PagerState::new().unwrap();
        let ev = Command::SetPrompt(TEST_STR.to_string());
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            ev,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(ps.prompt, TEST_STR.to_string());
    }

    #[test]
    #[cfg(any(feature = "dynamic_output", feature = "static_output"))]
    fn send_message() {
        let mut ps = PagerState::new().unwrap();
        let ev = Command::SendMessage(TEST_STR.to_string());
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            ev,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(ps.message.unwrap(), TEST_STR.to_string());
    }

    #[test]
    fn timed_message_clear_is_generation_guarded() {
        let mut ps = PagerState::new().unwrap();
        let mut command_queue = CommandQueue::new_zero();
        let is_exited = Arc::new(AtomicBool::new(false));

        handle_event(
            Command::SetTimedMessage {
                text: "older".to_string(),
                id: 7,
            },
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();
        handle_event(
            Command::SetTimedMessage {
                text: "newer".to_string(),
                id: 8,
            },
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();
        assert_eq!(ps.message.as_deref(), Some("newer"));
        assert_eq!(ps.message_id, Some(8));
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawPrompt))
        );
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawPrompt))
        );

        handle_event(
            Command::ClearMessage(7),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.message.as_deref(), Some("newer"));
        assert_eq!(ps.message_id, Some(8));
        assert!(command_queue.is_empty());

        handle_event(
            Command::ClearMessage(8),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.message, None);
        assert_eq!(ps.message_id, None);
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawPrompt))
        );
    }

    #[test]
    #[cfg(feature = "static_output")]
    fn set_run_no_overflow() {
        let mut ps = PagerState::new().unwrap();
        let ev = Command::SetRunNoOverflow(true);
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            ev,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert!(ps.run_no_overflow);
    }

    #[test]
    fn add_exit_callback() {
        let mut ps = PagerState::new().unwrap();
        let ev = Command::AddExitCallback(Box::new(|| println!("Hello World")));
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            ev,
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(ps.exit_callbacks.len(), 1);
    }

    #[test]
    fn update_selection_scrolls_up_at_top_edge() {
        let mut ps = PagerState::new().unwrap();
        ps.rows = 5;
        ps.screen.orig_text = (0..10).fold(String::new(), |mut t, idx| {
            let _ = writeln!(t, "line {idx}");
            t
        });
        ps.reformat_display().unwrap();
        ps.upper_mark = 3;
        ps.selection_anchor = Some(Selection {
            absolute_row: 3,
            col: 0,
        });
        ps.selection = ps.selection_anchor;
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            Command::UserInput(InputEvent::UpdateSelection { x: 0, y: 0 }),
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();

        assert_eq!(ps.upper_mark, 2);
        assert_eq!(
            ps.selection,
            Some(Selection {
                absolute_row: 2,
                col: 0,
            })
        );
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawDisplay))
        );
    }

    #[test]
    fn update_selection_scrolls_down_at_bottom_edge() {
        let mut ps = PagerState::new().unwrap();
        ps.rows = 5;
        ps.screen.orig_text = (0..10).fold(String::new(), |mut t, idx| {
            let _ = writeln!(t, "line {idx}");
            t
        });
        ps.reformat_display().unwrap();
        ps.upper_mark = 3;
        ps.selection_anchor = Some(Selection {
            absolute_row: 3,
            col: 0,
        });
        ps.selection = ps.selection_anchor;
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            Command::UserInput(InputEvent::UpdateSelection { x: 0, y: 4 }),
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();

        assert_eq!(ps.upper_mark, 4);
        assert_eq!(
            ps.selection,
            Some(Selection {
                absolute_row: 7,
                col: 0,
            })
        );
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawDisplay))
        );
    }

    #[test]
    fn update_selection_clamps_scroll_at_bottom_bound() {
        let mut ps = PagerState::new().unwrap();
        ps.rows = 5;
        ps.screen.orig_text = (0..6).fold(String::new(), |mut t, idx| {
            let _ = writeln!(t, "line {idx}");
            t
        });
        ps.reformat_display().unwrap();
        ps.upper_mark = 2;
        ps.selection_anchor = Some(Selection {
            absolute_row: 2,
            col: 0,
        });
        ps.selection = ps.selection_anchor;
        let mut command_queue = CommandQueue::new_zero();

        handle_event(
            Command::UserInput(InputEvent::UpdateSelection { x: 0, y: 10 }),
            &mut ps,
            &mut command_queue,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();

        assert_eq!(ps.upper_mark, 2);
        assert_eq!(
            ps.selection,
            Some(Selection {
                absolute_row: 5,
                col: 0,
            })
        );
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawSelection(2, 5)))
        );
    }

    #[test]
    #[cfg(feature = "clipboard")]
    fn clipboard_events_preserve_then_clear_selection() {
        let mut ps = PagerState::new().unwrap();
        ps.selection_anchor = Some(Selection {
            absolute_row: 0,
            col: 0,
        });
        ps.selection = ps.selection_anchor;
        let mut command_queue = CommandQueue::new_zero();
        let is_exited = Arc::new(AtomicBool::new(false));

        handle_event(
            Command::UserInput(InputEvent::FinalizeSelection),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert!(ps.selection.is_some());
        assert!(ps.selection_anchor.is_some());
        assert!(command_queue.is_empty());

        handle_event(
            Command::UserInput(InputEvent::CopySelection),
            &mut ps,
            &mut command_queue,
            &is_exited,
        )
        .unwrap();

        assert_eq!(ps.selection, None);
        assert_eq!(ps.selection_anchor, None);
        assert_eq!(
            command_queue.pop_front(),
            Some(Command::Io(IoCommand::RedrawSelection(0, 0)))
        );
    }
}

#[cfg(all(test, feature = "search"))]
#[path = "ev_handler/search_tests.rs"]
mod search_tests;
