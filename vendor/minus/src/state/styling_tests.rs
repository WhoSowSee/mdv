use super::*;
use crate::{Pager, PromptColor, PromptLine, PromptSpan, PromptStyle};

#[test]
fn output_styling_command_controls_default_and_custom_prompts() {
    let handle = Pager::new();
    handle.set_output_styling(false).unwrap();
    let mut state = PagerState::generate_initial_state(&handle.rx).unwrap();
    assert!(!state.displayed_prompt.contains('\x1b'));
    let style = PromptStyle::default().foreground(PromptColor::Red);
    state.prompt_panel = vec![PromptLine::new().left(PromptSpan::new("help", style).unwrap())];
    state.prompt_renderer = Some(Arc::new(move |_| {
        Ok(PromptLine::new().left(PromptSpan::new("footer", style)?))
    }));
    state.format_prompt().unwrap();
    assert!(!state.displayed_prompt.contains('\x1b'));
    assert!(!state.displayed_prompt_panel[0].contains('\x1b'));
    assert!(state.displayed_prompt.starts_with("footer"));
    assert!(state.displayed_prompt_panel[0].starts_with("help"));
    let mut output = Vec::new();
    crate::minus_core::utils::display::write_prompt_view(&mut output, &state).unwrap();
    assert!(!String::from_utf8(output).unwrap().contains("\x1b[0m"));
}

#[cfg(feature = "search")]
#[test]
fn disabled_styling_preserves_navigation_search_and_selection() {
    let handle = Pager::new();
    handle.set_output_styling(false).unwrap();
    handle
        .set_mapped_text(
            "target one\ntarget two\n".into(),
            Some(LineNavigation::from_current(vec![Some(1), Some(2)])),
        )
        .unwrap();
    let mut state = PagerState::generate_initial_state(&handle.rx).unwrap();
    state.search_state.search_term = Some(regex::Regex::new("target").unwrap());
    state.reformat_display().unwrap();
    assert_eq!(state.search_state.search_matches.len(), 2);
    state.begin_line_navigation().unwrap();
    assert!(state.finish_line_navigation(Some(2)).unwrap());
    assert_eq!(state.line_navigation_position(), Some(2));
    state.selection_anchor = Some(Selection {
        absolute_row: 1,
        col: 0,
    });
    state.selection = Some(Selection {
        absolute_row: 1,
        col: 5,
    });
    assert_eq!(state.selected_text().as_deref(), Some("target"));
    let rows = state.render_rows_for_display(0, 2);
    assert!(rows.iter().all(|row| !row.contains('\x1b')));
    assert_eq!(rows[1], "target two");
}
