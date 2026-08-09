mod fmt_write {
    use crate::{Pager, minus_core::commands::Command};
    use std::fmt::Write;

    #[test]
    fn pager_writeln() {
        const TEST: &str = "This is a line";
        let mut pager = Pager::new();
        writeln!(pager, "{TEST}").unwrap();
        while let Ok(Command::AppendData(text)) = pager.rx.try_recv() {
            if text != "\n" {
                assert_eq!(text, TEST.to_string());
            }
        }
    }

    #[test]
    fn test_write() {
        const TEST: &str = "This is a line";
        let mut pager = Pager::new();
        write!(pager, "{TEST}").unwrap();
        while let Ok(Command::AppendData(text)) = pager.rx.try_recv() {
            assert_eq!(text, TEST.to_string());
        }
    }
}

mod pager_append_str {
    use crate::PagerState;

    #[test]
    fn sequential_append_str() {
        const TEXT1: &str = "This is a line.";
        const TEXT2: &str = " This is a follow up line";
        let mut ps = PagerState::new().unwrap();
        ps.append_str(TEXT1).unwrap();
        ps.append_str(TEXT2).unwrap();
        assert_eq!(ps.screen.formatted_lines, vec![format!("{TEXT1}{TEXT2}")]);
        assert_eq!(ps.screen.orig_text, TEXT1.to_string() + TEXT2);
    }

    #[test]
    fn append_sequential_lines() {
        const TEXT1: &str = "This is a line.";
        const TEXT2: &str = " This is a follow up line";
        let mut ps = PagerState::new().unwrap();
        ps.append_str(&(TEXT1.to_string() + "\n")).unwrap();
        ps.append_str(&(TEXT2.to_string() + "\n")).unwrap();

        assert_eq!(
            ps.screen.formatted_lines,
            vec![TEXT1.to_string(), TEXT2.to_string()]
        );
    }

    #[test]
    fn crlf_write() {
        const LINES: [&str; 4] = [
            "hello,\n",
            "this is ",
            "a test\r\n",
            "of weird line endings",
        ];

        let mut ps = PagerState::new().unwrap();

        for line in LINES {
            ps.append_str(line).unwrap();
        }

        assert_eq!(
            ps.screen.formatted_lines,
            vec![
                "hello,".to_string(),
                "this is a test".to_string(),
                "of weird line endings".to_string()
            ]
        );
    }

    #[test]
    fn unusual_whitespace() {
        const LINES: [&str; 4] = [
            "This line has trailing whitespace      ",
            "     This has leading whitespace\n",
            "   This has whitespace on both sides   ",
            "Andthishasnone",
        ];

        let mut ps = PagerState::new().unwrap();

        for line in LINES {
            ps.append_str(line).unwrap();
        }

        assert_eq!(
            ps.screen.formatted_lines,
            vec![
                "This line has trailing whitespace           This has leading whitespace",
                "   This has whitespace on both sides   Andthishasnone"
            ]
        );
    }

    #[test]
    fn appendstr_with_newlines() {
        const LINES: [&str; 3] = [
            "this is a normal line with no newline",
            "this is an appended line with a newline\n",
            "and this is a third line",
        ];

        let mut ps = PagerState::new().unwrap();
        ps.cols = 15;

        for line in LINES {
            ps.append_str(line).unwrap();
        }

        assert_eq!(
            ps.screen.formatted_lines,
            vec![
                "this is a",
                "normal line",
                "with no",
                "newlinethis is",
                "an appended",
                "line with a",
                "newline",
                "and this is a",
                "third line"
            ]
        );
    }

    #[test]
    fn incremental_append() {
        const LINES: [&str; 4] = [
            "this is a line",
            " and this is another",
            " and this is yet another\n",
            "and this should be on a newline",
        ];

        let mut ps = PagerState::new().unwrap();

        ps.append_str(LINES[0]).unwrap();

        assert_eq!(ps.screen.orig_text, LINES[0].to_owned());
        assert_eq!(ps.screen.formatted_lines, vec![LINES[0].to_owned()]);

        ps.append_str(LINES[1]).unwrap();

        let line = LINES[..2].join("");
        assert_eq!(ps.screen.orig_text, line);
        assert_eq!(ps.screen.formatted_lines, vec![line]);

        ps.append_str(LINES[2]).unwrap();

        let mut line = LINES[..3].join("");
        assert_eq!(ps.screen.orig_text, line);

        line.pop();
        assert_eq!(ps.screen.formatted_lines, vec![line]);

        ps.append_str(LINES[3]).unwrap();

        let joined = LINES.join("");
        assert_eq!(ps.screen.orig_text, joined);
        assert_eq!(
            ps.screen.formatted_lines,
            joined
                .lines()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn multiple_newlines() {
        const TEST: &str = "This\n\n\nhas many\n newlines\n";

        let mut ps = PagerState::new().unwrap();

        ps.append_str(TEST).unwrap();

        assert_eq!(ps.screen.orig_text, TEST.to_owned());
        assert_eq!(
            ps.screen.formatted_lines,
            TEST.lines()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        );

        ps.screen.orig_text = TEST.to_string();
        ps.reformat_display().unwrap();

        assert_eq!(ps.screen.orig_text, TEST.to_owned());
        assert_eq!(
            ps.screen.formatted_lines,
            TEST.lines()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn append_floating_newline() {
        const TEST: &str = "This is a line with a bunch of\nin between\nbut not at the end";
        let mut ps = PagerState::new().unwrap();
        ps.append_str(TEST).unwrap();
        assert_eq!(
            ps.screen.formatted_lines,
            vec![
                "This is a line with a bunch of".to_string(),
                "in between".to_string(),
                "but not at the end".to_owned()
            ]
        );
        assert_eq!(ps.screen.orig_text, TEST.to_string());
    }

    #[test]
    fn format_lines_reuses_existing_rows_when_shrinking() {
        let mut ps = PagerState::new().unwrap();
        ps.screen.formatted_lines = vec!["x".repeat(64), "y".repeat(64), "z".repeat(64)];
        let vec_ptr = ps.screen.formatted_lines.as_ptr();
        let row0_ptr = ps.screen.formatted_lines[0].as_ptr();
        let row1_ptr = ps.screen.formatted_lines[1].as_ptr();

        ps.screen.orig_text = "short\nlines".to_string();
        ps.reformat_display().unwrap();

        assert_eq!(ps.screen.formatted_lines, vec!["short", "lines"]);
        assert_eq!(ps.screen.formatted_lines.as_ptr(), vec_ptr);
        assert_eq!(ps.screen.formatted_lines[0].as_ptr(), row0_ptr);
        assert_eq!(ps.screen.formatted_lines[1].as_ptr(), row1_ptr);
    }

    #[test]
    fn format_lines_reuses_existing_rows_when_growing_within_capacity() {
        let mut ps = PagerState::new().unwrap();
        let mut formatted_lines = Vec::with_capacity(4);
        formatted_lines.push("x".repeat(64));
        formatted_lines.push("y".repeat(64));
        ps.screen.formatted_lines = formatted_lines;

        let vec_ptr = ps.screen.formatted_lines.as_ptr();
        let row0_ptr = ps.screen.formatted_lines[0].as_ptr();
        let row1_ptr = ps.screen.formatted_lines[1].as_ptr();

        ps.screen.orig_text = "one\ntwo\nthree".to_string();
        ps.reformat_display().unwrap();

        assert_eq!(ps.screen.formatted_lines, vec!["one", "two", "three"]);
        assert_eq!(ps.screen.formatted_lines.as_ptr(), vec_ptr);
        assert_eq!(ps.screen.formatted_lines[0].as_ptr(), row0_ptr);
        assert_eq!(ps.screen.formatted_lines[1].as_ptr(), row1_ptr);
    }
}

#[cfg(feature = "dynamic_output")]
#[test]
fn exit_callback() {
    use crate::PagerState;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, atomic::AtomicBool};

    let mut ps = PagerState::new().unwrap();
    let exited = Arc::new(AtomicBool::new(false));
    let exited_within_callback = exited.clone();
    ps.exit_callbacks.push(Box::new(move || {
        exited_within_callback.store(true, Ordering::Relaxed);
    }));
    ps.exit();

    assert!(exited.load(Ordering::Relaxed));
}

mod emit_events {
    use crate::{Pager, minus_core::commands::Command};

    const TEST_STR: &str = "This is sample text";

    #[test]
    fn prompt_text_validation_returns_an_error() {
        let pager = Pager::new();

        assert!(pager.set_prompt("first\nsecond").is_err());
        assert!(pager.send_message("first\rsecond").is_err());
        #[cfg(feature = "search")]
        assert!(pager.set_search_prompt("Find:\n").is_err());
        assert!(pager.rx.try_recv().is_err());
    }

    #[test]
    fn send_message_for_emits_a_guarded_clear() {
        let pager = Pager::new();

        pager
            .send_message_for(TEST_STR, std::time::Duration::from_millis(1))
            .unwrap();
        assert_eq!(
            Command::SetTimedMessage {
                text: TEST_STR.to_string(),
                id: 1,
            },
            pager.rx.try_recv().unwrap()
        );
        assert_eq!(
            Command::ClearMessage(1),
            pager
                .rx
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap()
        );
    }
}
