use super::*;
use crate::{Pager, PromptColor, PromptLine, PromptSpan, PromptStyle};

#[cfg(feature = "search")]
#[test]
fn limited_depth_covers_prompts_search_selection_and_navigation() {
    let handle = Pager::new();
    handle.set_color_depth(crate::ColorDepth::Ansi16).unwrap();
    handle
        .set_mapped_text(
            "\x1b[38;2;100;170;255mtarget one\x1b[0m\nsecond row\n".into(),
            Some(LineNavigation::from_current(vec![Some(1), Some(2)])),
        )
        .unwrap();
    let mut state = PagerState::generate_initial_state(&handle.rx).unwrap();
    let style = PromptStyle::default()
        .foreground(PromptColor::AnsiValue(102))
        .background(PromptColor::AnsiValue(235));
    state.prompt_renderer = Some(Arc::new(move |_| {
        Ok(PromptLine::new().left(PromptSpan::new("footer", style)?))
    }));
    state.prompt_panel = vec![PromptLine::new().left(PromptSpan::new("help", style).unwrap())];
    state.search_state.search_term = Some(regex::Regex::new("target").unwrap());
    state.reformat_display().unwrap();
    state.begin_line_navigation().unwrap();
    state.finish_line_navigation(Some(2)).unwrap();
    state.selection_anchor = Some(Selection {
        absolute_row: 0,
        col: 7,
    });
    state.selection = Some(Selection {
        absolute_row: 0,
        col: 9,
    });
    let rows = state.render_rows_for_display(0, 2);
    assert!(rows[0].contains("\x1b[30;103m"));
    assert!(rows[0].contains("\x1b[30;107m"));
    assert!(rows[1].contains("\x1b[30;47m"));
    for output in [
        &state.displayed_prompt,
        &state.displayed_prompt_panel[0],
        rows[0].as_ref(),
        rows[1].as_ref(),
    ] {
        assert!(!output.contains("38;"));
        assert!(!output.contains("48;"));
    }
}

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

#[cfg(feature = "search")]
#[test]
#[allow(clippy::trivial_regex)]
fn custom_highlight_palette_reaches_rows_and_color_conversion() {
    let handle = Pager::new();
    handle
        .set_highlight_styles(crate::HighlightStyles {
            selection: crate::HighlightColors {
                foreground: Some(PromptColor::White),
                background: Some(PromptColor::Blue),
            },
            search: crate::HighlightColors {
                foreground: Some(PromptColor::Black),
                background: Some(PromptColor::Green),
            },
            search_current: crate::HighlightColors {
                foreground: Some(PromptColor::Reset),
                background: Some(PromptColor::Red),
            },
        })
        .unwrap();
    handle.push_str("target other target\n").unwrap();
    let mut state = PagerState::generate_initial_state(&handle.rx).unwrap();
    state.search_state.search_term = Some(regex::Regex::new("target").unwrap());
    state.reformat_display().unwrap();
    state.selection_anchor = Some(Selection {
        absolute_row: 0,
        col: 7,
    });
    state.selection = Some(Selection {
        absolute_row: 0,
        col: 11,
    });
    let row = state.render_rows_for_display(0, 1);
    for style in ["\x1b[97;104m", "\x1b[30;102m", "\x1b[39;101m"] {
        assert!(row[0].contains(style), "{row:?}");
    }
    assert_eq!(state.selected_text().as_deref(), Some("other"));
    state.color_depth = crate::ColorDepth::Ansi16;
    assert!(!state.render_rows_for_display(0, 1)[0].contains("48;"));
    state.output_styling = false;
    assert_eq!(
        state.render_rows_for_display(0, 1)[0],
        "target other target"
    );
}
