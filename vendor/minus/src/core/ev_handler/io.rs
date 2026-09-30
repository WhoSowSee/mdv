use super::{AppendStyle, CommandQueue, Hook, IoCommand, MinusError, PagerState, Write, display};
#[cfg(feature = "search")]
use super::{
    Arc, Condvar, Mutex, Pager, apply_line_navigation_result, apply_search_result, line_navigation,
    search, with_general_input_paused,
};

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
        #[cfg(feature = "search")]
        IoCommand::RedrawToc => display::draw_toc_update(out, p)?,
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
            let search_result =
                with_general_input_paused(user_input_active, || search::fetch_input(&mut out, p))?;

            apply_search_result(p, pager, command_queue, search_result)?;
        }
        #[cfg(feature = "search")]
        IoCommand::FetchLineNumber => {
            let input = with_general_input_paused(user_input_active, || {
                line_navigation::fetch_line_number(&mut out, p)
            })?;

            apply_line_navigation_result(p, pager, command_queue, input)?;
        }
    }
    Ok(())
}
