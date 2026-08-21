#![allow(clippy::shadow_unrelated)]
#![allow(clippy::cast_possible_truncation)]
use super::{
    draw_for_change, draw_full, draw_selection_rows, write_from_pagerstate, write_prompt_view,
};
use crate::{LineNumbers, PagerState, state::Selection};
use crossterm::cursor::MoveTo;
use std::fmt::Write;

// PagerState uses an 80×10 viewport by default in tests.

#[test]
fn short_no_line_numbers() {
    let lines = "A line\nAnother line";
    let mut pager = PagerState::new().unwrap();

    pager.screen.orig_text = lines.to_string();
    pager.reformat_display().unwrap();

    let mut out = Vec::with_capacity(lines.len());

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\rA line\n\rAnother line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark += 1;

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\rA line\n\rAnother line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 0);
}

#[test]
fn selection_redraw_updates_rows_without_clearing_the_screen() {
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = "zero\none\ntwo".to_string();
    pager.reformat_display().unwrap();
    pager.selection_anchor = Some(Selection {
        absolute_row: 1,
        col: 0,
    });
    pager.selection = Some(Selection {
        absolute_row: 1,
        col: 2,
    });
    let mut out = Vec::new();

    draw_selection_rows(&mut out, &pager, 1, 1).unwrap();

    let rendered = String::from_utf8(out).unwrap();
    assert!(rendered.contains(&MoveTo(0, 1).to_string()));
    assert!(rendered.contains("\x1b[48;2;46;49;59mone\x1b[0m"));
    assert!(
        !rendered
            .contains(&crossterm::terminal::Clear(crossterm::terminal::ClearType::All).to_string())
    );
}

#[test]
fn prompt_view_resets_inherited_attributes_before_content() {
    let mut pager = PagerState::new().unwrap();
    pager.displayed_prompt = "status".to_string();
    let mut out = crossterm::style::Attribute::Italic.to_string().into_bytes();

    write_prompt_view(&mut out, &pager).unwrap();

    let rendered = String::from_utf8(out).unwrap();
    let prompt = format!("{}\r\x1b[0mstatus", MoveTo(0, pager.prompt_row() as u16));
    assert!(rendered.contains(&prompt));
}

#[test]
fn long_no_line_numbers() {
    let lines = "A line\nAnother line\nThird line\nFourth line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.rows = 4;
    pager.screen.orig_text = lines.to_string();
    pager.reformat_display().unwrap();

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\rA line\n\rAnother line\n\rThird line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.screen.orig_text = "Another line\nThird line\nFourth line\nFifth line\n".to_string();
    pager.upper_mark = 1;
    pager.reformat_display().unwrap();

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\rThird line\n\rFourth line\n\rFifth line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 1);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 2;

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\rFourth line\n\rFifth line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 2);
}

#[test]
fn short_with_line_numbers() {
    let lines = "A line\nAnother line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = lines.to_string();
    pager.line_numbers = LineNumbers::Enabled;
    pager.reformat_display().unwrap();

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\r     1. A line\n\r     2. Another line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 1;
    pager.line_numbers = LineNumbers::AlwaysOn;

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\r     1. A line\n\r     2. Another line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 0);
}

#[test]
fn long_with_line_numbers() {
    let lines = "A line\nAnother line\nThird line\nFourth line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.rows = 4;
    pager.screen.orig_text = lines.to_string();
    pager.line_numbers = LineNumbers::Enabled;
    pager.reformat_display().unwrap();

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\r     1. A line\n\r     2. Another line\n\r     3. Third line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 1;

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\r     2. Another line\n\r     3. Third line\n\r     4. Fourth line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 1);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 2;

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\r     3. Third line\n\r     4. Fourth line\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 2);
}

#[test]
fn big_line_numbers_are_padded() {
    let lines = {
        let mut l = String::with_capacity(450);
        for i in 0..110 {
            writeln!(&mut l, "L{i}").unwrap();
        }
        l
    };

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 95;
    pager.rows = 11;
    pager.screen.orig_text = lines;
    pager.line_numbers = LineNumbers::AlwaysOn;
    pager.reformat_display().unwrap();

    assert!(write_from_pagerstate(&mut out, &mut pager).is_ok());

    assert_eq!(
        "\r      96. L95\n\r      97. L96\n\r      98. L97\n\r      99. L98\n\r     100. \
         L99\n\r     101. L100\n\r     102. L101\n\r     103. L102\n\r     104. L103\n\r     105. L104\n",
        String::from_utf8(out).expect("Should have written valid UTF-8")
    );
    assert_eq!(pager.upper_mark, 95);
}

#[test]
fn line_numbers_not() {
    #[allow(clippy::enum_glob_use)]
    use LineNumbers::*;

    assert_eq!(AlwaysOn, !AlwaysOn);
    assert_eq!(AlwaysOff, !AlwaysOff);
    assert_eq!(Enabled, !Disabled);
    assert_eq!(Disabled, !Enabled);
}

#[test]
fn line_numbers_invertible() {
    #[allow(clippy::enum_glob_use)]
    use LineNumbers::*;

    assert!(!AlwaysOn.is_invertible());
    assert!(!AlwaysOff.is_invertible());
    assert!(Enabled.is_invertible());
    assert!(Disabled.is_invertible());
}

#[test]
fn draw_short_no_line_numbers() {
    let lines = "A line\nAnother line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = lines.to_string();
    pager.line_numbers = LineNumbers::AlwaysOff;
    pager.reformat_display().unwrap();

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\rA line\n\rAnother line")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 1;

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\rA line\n\rAnother line")
    );
    assert_eq!(pager.upper_mark, 0);
}

#[test]
fn draw_long_no_line_numbers() {
    let lines = "A line\nAnother line\nThird line\nFourth line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.rows = 3;
    pager.screen.orig_text = lines.to_string();
    pager.reformat_display().unwrap();

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\rA line\n\rAnother line")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 1;

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\rAnother line\n\rThird line")
    );
    assert_eq!(pager.upper_mark, 1);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 3;

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\rFourth line\n")
    );
    assert_eq!(pager.upper_mark, 3);
}

#[test]
fn draw_short_with_line_numbers() {
    let lines = "A line\nAnother line";
    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = lines.to_string();
    pager.line_numbers = LineNumbers::Enabled;
    pager.reformat_display().unwrap();

    assert!(draw_full(&mut out, &mut pager).is_ok());
    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\r     1. A line\n\r     2. Another line")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 1;

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\r     1. A line\n\r     2. Another line")
    );
    assert_eq!(pager.upper_mark, 0);
}

#[test]
fn draw_long_with_line_numbers() {
    let lines = "A line\nAnother line\nThird line\nFourth line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.rows = 3;
    pager.screen.orig_text = lines.to_string();
    pager.line_numbers = LineNumbers::Enabled;
    pager.reformat_display().unwrap();

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\r     1. A line\n\r     2. Another line")
    );
    assert_eq!(pager.upper_mark, 0);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 1;

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\r     2. Another line\n\r     3. Third line")
    );
    assert_eq!(pager.upper_mark, 1);

    let mut out = Vec::with_capacity(lines.len());
    pager.upper_mark = 3;

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains("\r     4. Fourth line\n")
    );
    assert_eq!(pager.upper_mark, 3);
}

#[test]
fn draw_big_line_numbers_are_padded() {
    let lines = {
        let mut l = String::with_capacity(450);
        for i in 0..110 {
            writeln!(&mut l, "L{i}").unwrap();
        }
        l
    };

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.upper_mark = 95;
    pager.screen.orig_text = lines;
    pager.line_numbers = LineNumbers::Enabled;
    pager.reformat_display().unwrap();

    assert!(draw_full(&mut out, &mut pager).is_ok());

    assert!(String::from_utf8(out)
        .expect("Should have written valid UTF-8")
        .contains(
            "\r      96. L95\n\r      97. L96\n\r      98. L97\n\r      99. L98\n\r     100. L99\n\r     101. L100\n\r     102. L101\n\r     103. L102\n\r     104. L103",
        )
    );
    assert_eq!(pager.upper_mark, 95);
}

#[test]
fn draw_wrapping_line_numbers() {
    let lines = (0..3)
        .map(|l| format!("Line {l}: This is the line who is {l}"))
        .collect::<Vec<String>>()
        .join("\n");

    let mut out = Vec::new();
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = lines;
    pager.cols = 30;
    pager.upper_mark = 2;
    pager.line_numbers = LineNumbers::Enabled;
    pager.reformat_display().unwrap();

    assert!(draw_full(&mut out, &mut pager).is_ok());

    let written = String::from_utf8(out).expect("Should have written valid UTF-8");
    let expected = "     2. Line 1: This is the\n\r        line who is 1\n\r     3. Line 2: This is the\n\r        line who is 2";
    assert!(written.contains(expected));
}

#[test]
fn draw_help_message() {
    let lines = "A line\nAnother line";

    let mut out = Vec::with_capacity(lines.len());
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = lines.to_string();
    pager.line_numbers = LineNumbers::AlwaysOff;
    pager.format_prompt().unwrap();

    draw_full(&mut out, &mut pager).expect("Should have written");

    let res = String::from_utf8(out).expect("Should have written valid UTF-8");
    assert!(res.contains("minus"));
}

#[test]
fn test_draw_no_overflow() {
    const TEXT: &str = "This is a line of text to the pager";
    let mut out = Vec::with_capacity(TEXT.len());
    let mut pager = PagerState::new().unwrap();
    pager.screen.orig_text = TEXT.to_string();
    pager.reformat_display().unwrap();
    draw_full(&mut out, &mut pager).unwrap();
    assert!(
        String::from_utf8(out)
            .expect("Should have written valid UTF-8")
            .contains(TEXT)
    );
}

#[cfg(test)]
mod draw_for_change_tests {
    use super::draw_for_change;
    use crate::{PromptLine, state::PagerState};
    use crossterm::{
        cursor::MoveTo,
        terminal::{Clear, ClearType, ScrollDown, ScrollUp},
    };
    use std::fmt::Write as FmtWrite;
    use std::io::Write as IOWrite;
    use std::sync::Arc;

    fn create_pager_state() -> PagerState {
        let lines = {
            let mut l = String::with_capacity(450);
            for i in 0..100 {
                writeln!(&mut l, "L{i}").unwrap();
            }
            l
        };
        let mut ps = PagerState::new().unwrap();
        ps.upper_mark = 0;
        ps.screen.orig_text = lines;
        ps.reformat_display().unwrap();
        ps.format_prompt().unwrap();
        ps
    }

    fn write_expected_prompt(out: &mut Vec<u8>, ps: &PagerState) {
        write!(
            out,
            "{}\r{}{}",
            MoveTo(0, ps.prompt_row().try_into().unwrap()),
            crossterm::style::Attribute::Reset,
            ps.displayed_prompt
        )
        .unwrap();
    }

    fn atomic_body(out: &[u8]) -> &[u8] {
        out.strip_prefix(b"\x1b[?2026h")
            .and_then(|body| body.strip_suffix(b"\x1b[?2026l"))
            .expect("scroll output should be synchronized")
    }

    #[test]
    fn custom_prompt_is_reformatted_after_scroll() {
        let mut ps = create_pager_state();
        ps.prompt_renderer = Some(Arc::new(|context| {
            crate::PromptLine::plain(format!("position:{}", context.upper_mark()))
        }));
        ps.format_prompt().unwrap();
        let mut out = Vec::new();
        let mut upper_mark = 3;

        draw_for_change(&mut out, &mut ps, &mut upper_mark).unwrap();

        assert!(ps.displayed_prompt.starts_with("position:3"));
        assert!(ps.displayed_prompt.ends_with("\x1b[0m"));
        assert!(
            String::from_utf8(out)
                .expect("Should have written valid UTF-8")
                .contains("position:3")
        );
    }

    #[test]
    fn scrolling_with_a_prompt_panel_uses_an_atomic_full_redraw() {
        let mut ps = create_pager_state();
        ps.rows = 10;
        ps.cols = 20;
        ps.prompt_renderer = Some(Arc::new(|_| PromptLine::plain("status")));
        ps.prompt_panel = vec![
            PromptLine::plain("help one").unwrap(),
            PromptLine::plain("help two").unwrap(),
        ];
        ps.format_prompt().unwrap();
        let mut out = Vec::new();
        let mut upper_mark = 3;

        draw_for_change(&mut out, &mut ps, &mut upper_mark).unwrap();

        assert!(out.windows(8).any(|window| window == b"\x1b[?2026h"));
        assert!(out.windows(4).any(|window| window == b"\x1b[2J"));
        assert!(!out.windows(4).any(|window| window == b"\x1b[3S"));
        let written = String::from_utf8(out).unwrap();
        assert!(written.contains(&MoveTo(0, 7).to_string()));
        let status = written.find("status").unwrap();
        let first_help = written.find("help one").unwrap();
        let second_help = written.find("help two").unwrap();
        assert!(status < first_help && first_help < second_help);
        assert_eq!(written.matches("help one").count(), 1);
        assert_eq!(written.matches("help two").count(), 1);
    }

    #[cfg(feature = "search")]
    #[test]
    #[allow(clippy::trivial_regex)]
    fn scrolling_with_active_search_uses_a_full_redraw() {
        let mut pager = create_pager_state();
        pager.search_state.search_term = Some(regex::Regex::new("L").unwrap());
        let mut out = Vec::new();
        let mut upper_mark = 3;

        draw_for_change(&mut out, &mut pager, &mut upper_mark).unwrap();

        assert!(out.windows(4).any(|window| window == b"\x1b[2J"));
        assert!(!out.windows(4).any(|window| window == b"\x1b[3S"));
    }

    #[test]
    fn incremental_scrolling_is_atomic_in_both_directions() {
        for (initial_upper_mark, requested_upper_mark, expected_scroll) in [
            (0, 3, ScrollUp(3).to_string()),
            (60, 50, ScrollDown(9).to_string()),
        ] {
            let mut ps = create_pager_state();
            ps.upper_mark = initial_upper_mark;
            let mut upper_mark = requested_upper_mark;
            let mut out = Vec::new();

            draw_for_change(&mut out, &mut ps, &mut upper_mark).unwrap();

            let written = String::from_utf8(out).unwrap();
            assert!(written.starts_with("\x1b[?2026h"));
            assert!(written.contains(&expected_scroll));
            assert!(written.ends_with("\x1b[?2026l"));
        }
    }

    #[test]
    fn small_scrolldown() {
        let mut ps = create_pager_state();
        let mut out = Vec::with_capacity(100);

        let mut res = Vec::new();
        write!(
            res,
            "{}{}{}",
            ScrollUp(3),
            MoveTo(0, ps.rows as u16 - 4),
            Clear(ClearType::CurrentLine)
        )
        .unwrap();
        for line in &ps.screen.formatted_lines[9..12] {
            writeln!(res, "\r{line}").unwrap();
        }
        write_expected_prompt(&mut res, &ps);

        draw_for_change(&mut out, &mut ps, &mut 3).unwrap();

        assert_eq!(atomic_body(&out), res.as_slice());
    }

    #[test]
    fn scrolls_one_row_past_eof() {
        let mut ps = PagerState::new().unwrap();
        ps.rows = 4;
        ps.screen.orig_text = "First line\nSecond line\nThird line\nFourth line".to_string();
        ps.reformat_display().unwrap();
        ps.upper_mark = 1;
        let mut upper_mark = 2;
        let mut out = Vec::new();

        draw_for_change(&mut out, &mut ps, &mut upper_mark).unwrap();

        assert_eq!(upper_mark, 2);
        assert_eq!(ps.upper_mark, 2);
        assert!(atomic_body(&out).starts_with(ScrollUp(1).to_string().as_bytes()));
    }

    #[test]
    fn large_scrolldown() {
        let mut ps = create_pager_state();
        let mut out = Vec::with_capacity(100);

        let mut res = Vec::new();
        write!(
            res,
            "{}{}{}",
            ScrollUp(9),
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine)
        )
        .unwrap();
        for line in &ps.screen.formatted_lines[50..59] {
            writeln!(res, "\r{line}").unwrap();
        }
        write_expected_prompt(&mut res, &ps);

        draw_for_change(&mut out, &mut ps, &mut 50).unwrap();

        assert_eq!(atomic_body(&out), res.as_slice());
    }

    #[test]
    fn no_overflow_change() {
        let mut ps = create_pager_state();
        ps.screen.formatted_lines.truncate(5);
        let mut out = Vec::with_capacity(100);
        let mut new_upper_mark = 10;

        let res = Vec::new();

        draw_for_change(&mut out, &mut ps, &mut new_upper_mark).unwrap();

        assert_eq!(out, res);
    }

    #[test]
    fn large_scrollup() {
        let mut ps = create_pager_state();
        let mut out = Vec::with_capacity(100);
        ps.upper_mark = 80;

        let mut res = Vec::new();
        write!(res, "{}{}", ScrollDown(9), MoveTo(0, 0)).unwrap();
        for line in &ps.screen.formatted_lines[20..29] {
            writeln!(res, "\r{line}").unwrap();
        }
        write_expected_prompt(&mut res, &ps);

        draw_for_change(&mut out, &mut ps, &mut 20).unwrap();

        assert_eq!(atomic_body(&out), res.as_slice());
    }

    #[test]
    fn small_scrollup() {
        let mut ps = create_pager_state();
        let mut out = Vec::with_capacity(100);
        ps.upper_mark = 60;

        let mut res = Vec::new();
        write!(res, "{}{}", ScrollDown(9), MoveTo(0, 0)).unwrap();
        for line in &ps.screen.formatted_lines[50..59] {
            writeln!(res, "\r{line}").unwrap();
        }
        write_expected_prompt(&mut res, &ps);

        draw_for_change(&mut out, &mut ps, &mut 50).unwrap();

        assert_eq!(atomic_body(&out), res.as_slice());
    }
}

#[cfg(test)]
mod horizontal_scroll_bounds_tests {
    use super::super::get_horizontal_scroll_bounds;

    #[test]
    fn basic_bounds_no_line_numbers() {
        let line = "0123456789";
        let bounds = get_horizontal_scroll_bounds(line, 5, 2, false, 10);
        assert_eq!(bounds, (0, 2, 7));
    }

    #[test]
    fn basic_bounds_with_line_numbers() {
        let line = format!(
            "{:5}{}10.{} 0123456789",
            "",
            crossterm::style::Attribute::Bold,
            crossterm::style::Attribute::Reset
        );

        let bounds = get_horizontal_scroll_bounds(&line, 5, 2, true, 10);
        assert_eq!(bounds, (17, 19, 19));
    }

    #[test]
    fn non_ascii_bounds() {
        let line = "├── 0123456789";
        let bounds = get_horizontal_scroll_bounds(line, 5, 1, false, 10);
        assert_eq!(bounds, (0, 3, 6));
    }

    #[test]
    fn overflow_bounds() {
        let line = "0123";
        let bounds = get_horizontal_scroll_bounds(line, 10, 0, false, 10);
        assert_eq!(bounds, (0, 0, 4));
    }
}
