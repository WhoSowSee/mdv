#[cfg(feature = "clipboard")]
use super::copy_selection;
use super::{
    AppendStyle, Arc, AtomicBool, Command, CommandQueue, ExitStrategy, Hook, InputEvent, IoCommand,
    PagerState, PromptError, queue_prompt_redraw, queue_selection_redraw,
};
#[cfg(feature = "search")]
use super::{
    deactivate_search, dismiss_timed_message, move_to_next_search_match,
    move_to_previous_search_match,
};

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
        #[cfg(feature = "search")]
        Command::SetLayoutRenderer(renderer) => {
            p.layout_renderer = Some(renderer);
            p.refresh_layout()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        #[cfg(feature = "search")]
        Command::UserInput(
            action @ (InputEvent::ToggleToc
            | InputEvent::MoveToc(_)
            | InputEvent::MoveTocEntry(_)
            | InputEvent::ScrollTocKeyboard(_)
            | InputEvent::SelectToc(_)
            | InputEvent::ScrollToc(_)
            | InputEvent::CycleToc(_)),
        ) => {
            p.handle_toc_action(action)?;
            let redraw = if matches!(
                action,
                InputEvent::ScrollToc(_) | InputEvent::ScrollTocKeyboard(_)
            ) {
                IoCommand::RedrawToc
            } else {
                IoCommand::RedrawDisplay
            };
            command_queue.push_back(Command::Io(redraw));
        }
        #[cfg(feature = "search")]
        Command::SetMappedData(text, navigation) => {
            p.replace_mapped_text(text, navigation)?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        #[cfg(feature = "search")]
        Command::RefreshLayout => {
            if p.refresh_layout()? {
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
            }
        }
        Command::SetData(text) => {
            #[cfg(feature = "search")]
            p.configure_line_navigation(None)?;
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
            #[cfg(feature = "search")]
            p.toc.reset_position();
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
        Command::UserInput(InputEvent::SelectAll) => {
            let previous_span = p.selection_row_span();
            p.select_all();
            queue_selection_redraw(command_queue, previous_span, p.selection_row_span());
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
            #[cfg(feature = "search")]
            if p.refresh_layout()? {
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
                return Ok(());
            }
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
            if p.toc_visible() {
                p.handle_toc_action(InputEvent::ToggleToc)?;
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
            }
            dismiss_timed_message(p);
            p.search_mode = m;
            p.search_state.search_mode = m;
            p.search_state.search_mark = 0;
            command_queue.push_back(Command::Io(IoCommand::FetchSearchQuery));
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::GoToLine) => {
            if p.toc_visible() {
                p.handle_toc_action(InputEvent::ToggleToc)?;
            }
            if p.begin_line_navigation()? {
                dismiss_timed_message(p);
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
                command_queue.push_back(Command::Io(IoCommand::FetchLineNumber));
            }
        }
        #[cfg(feature = "search")]
        Command::UserInput(InputEvent::ExitLineNavigation) => {
            if p.exit_line_navigation()? {
                command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
            }
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
            #[cfg(feature = "search")]
            p.configure_line_navigation(None)?;
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
        #[cfg(feature = "search")]
        Command::SetLineNavigation(navigation) => p.configure_line_navigation(navigation)?,
        Command::SetLineNumbers(ln) => {
            p.line_numbers = ln;
            p.reformat_display()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
        }
        Command::SetOutputStyling(enabled) => {
            p.output_styling = enabled;
            p.format_prompt()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::SetColorDepth(depth) => {
            p.color_depth = depth;
            p.format_prompt()?;
            command_queue.push_back(Command::Io(IoCommand::RedrawDisplay));
            queue_prompt_redraw(p, command_queue)?;
        }
        Command::SetHighlightStyles(styles) => {
            p.highlight_styles = styles;
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
