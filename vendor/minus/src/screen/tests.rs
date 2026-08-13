mod unterminated {
    use crate::screen::{FormatOpts, Rows, format_text_block};

    const fn get_append_opts_template(text: &'_ str) -> FormatOpts<'_, Rows> {
        FormatOpts {
            buffer: Vec::new(),
            text,
            attachment: None,
            #[cfg(feature = "search")]
            search_term: None,
            lines_count: 0,
            formatted_lines_count: 0,
            cols: 80,
            line_numbers: crate::LineNumbers::Disabled,
            prev_unterminated: 0,
            line_wrapping: true,
        }
    }

    #[test]
    fn test_single_no_endline() {
        let append_style = format_text_block(get_append_opts_template("This is a line"));
        assert_eq!(1, append_style.num_unterminated);
    }

    #[test]
    fn test_single_endline() {
        let append_style = format_text_block(get_append_opts_template("This is a line\n"));
        assert_eq!(0, append_style.num_unterminated);
    }

    #[test]
    fn test_single_multi_newline() {
        let append_style = format_text_block(get_append_opts_template(
            "This is a line\nThis is another line\nThis is third line",
        ));
        assert_eq!(1, append_style.num_unterminated);
    }

    #[test]
    fn test_single_multi_endline() {
        let append_style = format_text_block(get_append_opts_template(
            "This is a line\nThis is another line\n",
        ));
        assert_eq!(0, append_style.num_unterminated);
    }

    #[test]
    fn test_single_line_wrapping() {
        let mut fs = get_append_opts_template("This is a quite lengthy line");
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(2, append_style.num_unterminated);
    }

    #[test]
    fn test_single_mid_newline_wrapping() {
        let mut fs = get_append_opts_template(
            "This is a quite lengthy line\nIt has three lines\nThis is
third line",
        );
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(1, append_style.num_unterminated);
    }

    #[test]
    fn test_single_endline_wrapping() {
        let mut fs = get_append_opts_template(
            "This is a quite lengthy line\nIt has three lines\nThis is
third line\n",
        );
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(0, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_no_endline() {
        let append_style = format_text_block(get_append_opts_template("This is a line. "));
        assert_eq!(1, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is another line");
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is a line. ");

        let append_style = format_text_block(fs);
        assert_eq!(1, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_endline() {
        let append_style = format_text_block(get_append_opts_template("This is a line. "));
        assert_eq!(1, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is another line\n");
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is a line. ");

        let append_style = format_text_block(fs);
        assert_eq!(0, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_multiple_newline() {
        let append_style = format_text_block(get_append_opts_template("This is a line\n"));
        assert_eq!(0, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is another line\n");
        fs.lines_count = 1;
        fs.formatted_lines_count = 1;
        fs.attachment = None;

        let append_style = format_text_block(fs);
        assert_eq!(0, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_wrapping() {
        let mut fs = get_append_opts_template("This is a line. This is second line. ");
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(2, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is another line\n");
        fs.cols = 20;
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is a line. This is second line");

        let append_style = format_text_block(fs);
        assert_eq!(0, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_wrapping_continued() {
        let mut fs = get_append_opts_template("This is a line. This is second line. ");
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(2, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is third line");
        fs.cols = 20;
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is a line. This is second line. ");

        let append_style = format_text_block(fs);
        assert_eq!(3, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_wrapping_last_continued() {
        let mut fs = get_append_opts_template("This is a line.\nThis is second line. ");
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(1, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is third line.");
        fs.cols = 20;
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is second line. ");
        fs.lines_count = 1;
        fs.formatted_lines_count = 2;

        let append_style = format_text_block(fs);

        assert_eq!(2, append_style.num_unterminated);
    }

    #[test]
    fn test_multi_wrapping_additive() {
        let mut fs = get_append_opts_template("This is a line. ");
        fs.cols = 20;
        let append_style = format_text_block(fs);
        assert_eq!(1, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is second line. ");
        fs.cols = 20;
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is a line. ");

        let append_style = format_text_block(fs);
        assert_eq!(2, append_style.num_unterminated);

        let mut fs = get_append_opts_template("This is third line");
        fs.cols = 20;
        fs.prev_unterminated = append_style.num_unterminated;
        fs.attachment = Some("This is a line. This is second line. ");
        let append_style = format_text_block(fs);

        assert_eq!(3, append_style.num_unterminated);
    }
}

mod wrapping {
    use super::super::format_line;
    use crate::LineNumbers;
    use crate::selection::strip_ansi;

    #[test]
    fn osc8_hyperlink_does_not_change_wrap_position() {
        let plain = "Короткая ссылка Длинная ссылка ссылка которая занимает почти всю строку sd";
        let linked = concat!(
            "Короткая \x1b]8;;https://example.com\x1b\\ссылка\x1b]8;;\x1b\\ ",
            "Длинная ссылка ",
            "\x1b]8;;https://very-long-url-that-should-be-truncated-when-using-cut-mode.exams\x1b\\",
            "ссылка которая занимает почти всю строку",
            "\x1b]8;;\x1b\\ sd"
        );

        for width in [10, 29, 30, 31, 80] {
            let plain_rows = format_line(plain, 1, 0, LineNumbers::Disabled, width, true)
                .map(|row| row.raw_row().to_string())
                .collect::<Vec<_>>();
            let linked_rows = format_line(linked, 1, 0, LineNumbers::Disabled, width, true)
                .map(|row| strip_ansi(row.raw_row()).into_owned())
                .collect::<Vec<_>>();

            assert_eq!(
                linked_rows, plain_rows,
                "mismatched wrap at {width} columns"
            );
        }

        let narrow_lines = [
            concat!(
                " Короткая \x1b]8;;https://example.com\x1b\\ссылка\x1b]8;;\x1b\\ ",
                "Длинная ссылк"
            ),
            concat!(
                " а \x1b]8;;https://very-long-url-that-should-be-truncated-when-using-cut-mode.exams\x1b\\",
                "ссылка которая занимает поч\x1b]8;;\x1b\\"
            ),
            concat!(
                " \x1b]8;;https://very-long-url-that-should-be-truncated-when-using-cut-mode.exams\x1b\\",
                "ти всю строку\x1b]8;;\x1b\\ sd"
            ),
        ];

        for line in narrow_lines {
            let rows = format_line(line, 1, 0, LineNumbers::Disabled, 30, true)
                .map(|row| row.raw_row().to_string())
                .collect::<Vec<_>>();

            assert_eq!(rows, vec![line.to_string()]);
        }
    }
}
