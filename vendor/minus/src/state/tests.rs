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
fn select_all_includes_offscreen_wrapped_unicode_text() {
    let mut ps = PagerState::new().unwrap();
    ps.cols = 6;
    ps.rows = 2;
    ps.screen.orig_text = "\x1b[31mabcdefghi\x1b[0m\n界e\u{301}\nlast".to_string();
    ps.reformat_display().unwrap();
    ps.upper_mark = 1;
    ps.select_all();
    assert_eq!(
        ps.selected_text().as_deref(),
        Some("abcdefghi\n界e\u{301}\nlast")
    );
    assert_eq!(ps.upper_mark, 1);
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
    assert!(ps.render_rows_for_display(0, 1)[0].contains("\x1b[48;2;46;49;59me\u{301}\x1b[0m"));
}

#[test]
fn selection_highlight_preserves_and_restores_sgr_styles() {
    const SELECTION_BACKGROUND: &str = "\x1b[48;2;46;49;59m";
    let rendered = highlight_visible_range(
        Cow::Borrowed("\x1b[31mred\x1b[0m plain"),
        0,
        "red plain".chars().count(),
        crate::ColorDepth::TrueColor,
    );

    assert!(rendered.contains(&format!("\x1b[31m{SELECTION_BACKGROUND}red")));
    assert!(!rendered.contains("\x1b[7m"));

    let rendered = highlight_visible_range(
        Cow::Borrowed("\x1b[31mred plain\x1b[0m"),
        0,
        3,
        crate::ColorDepth::TrueColor,
    );
    assert!(rendered.contains("red\x1b[0m\x1b[31m plain"));
}
