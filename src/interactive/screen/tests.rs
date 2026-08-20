use super::super::discovery::DocumentEntry;
use super::*;

#[test]
fn pagination_dots_are_adjacent() {
    let documents = (0..3)
        .map(|index| DocumentEntry::for_test(&format!("document-{index}.md")))
        .collect();
    let browser = BrowserState::for_test(documents, 14);

    assert_eq!(pagination(&browser, 80, true), "•••");
}

#[test]
fn recent_documents_keep_relative_time() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);
    let modified = now - Duration::from_secs(6 * 86_400);

    assert_eq!(document_time(modified, now).unwrap(), "6 days ago");
}

#[test]
fn week_old_documents_use_an_absolute_local_timestamp() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);
    let modified = now - Duration::from_secs(7 * 86_400);

    let timestamp = document_time(modified, now).unwrap();
    assert_eq!(timestamp, format_local_timestamp(modified).unwrap());
    assert!(timestamp.contains(':'));
    assert!(timestamp.split_whitespace().count() >= 5);
}

#[test]
fn document_titles_align_with_the_header() {
    assert_eq!(item_prefix(false, true), "   ");
    assert_eq!(item_prefix(true, true), " │ ");
}

#[test]
fn logo_block_aligns_with_the_header_without_a_spinner() {
    assert_eq!(browser_logo_line(None, true), "    MDV ");
}

#[test]
fn logo_block_delays_the_spinner_and_starts_from_the_first_frame() {
    assert_eq!(
        browser_logo_line(Some(Duration::from_millis(15)), true),
        "    MDV "
    );
    assert_eq!(
        browser_logo_line(Some(Duration::from_millis(16)), true),
        " |  MDV "
    );
    assert_eq!(
        browser_logo_line(Some(Duration::from_millis(116)), true),
        " /  MDV "
    );
}

#[test]
fn frame_diff_writes_only_changes_and_skips_an_unchanged_frame() {
    let mut previous = ScreenFrame::new(80, 4);
    previous.write_line(0, "unchanged");
    previous.write_line(1, "removed");
    let mut next = ScreenFrame::new(80, 4);
    next.write_line(0, "unchanged");
    next.write_line(2, "added");
    let mut output = String::new();

    encode_synchronized_frame(&mut output, Some(&previous), &next).unwrap();

    assert!(output.contains("added"));
    assert!(!output.contains("unchanged"));
    assert!(output.contains("\x1b[2;1H\x1b[K"));

    output.clear();
    encode_synchronized_frame(&mut output, Some(&next), &next).unwrap();
    assert!(output.is_empty());
}

#[test]
fn first_frame_is_a_full_synchronized_redraw() {
    let mut frame = ScreenFrame::new(80, 4);
    frame.write_line(1, "first frame");
    let mut output = String::new();

    encode_synchronized_frame(&mut output, None, &frame).unwrap();

    assert!(output.contains("\x1b[2J"));

    let begin = output.find("\x1b[?2026h").unwrap();
    let content = output.find("first frame").unwrap();
    let end = output.find("\x1b[?2026l").unwrap();
    assert!(begin < content && content < end);
}

#[test]
fn mini_help_matches_the_navigation_status() {
    assert_eq!(
        browser_mini_help(&BrowserState::for_test(Vec::new(), 24), 120, true),
        "   h/l ←/→ page • / find • r refresh • e edit • q quit • ? more"
    );
}

#[test]
fn collapsed_help_draws_only_the_mini_help() {
    let browser = BrowserState::for_test(Vec::new(), 24);
    let mut frame = ScreenFrame::new(120, 24);
    let (_, help_y) = browser_footer_rows(24, &browser);

    draw_browser_help(&mut frame, &browser, help_y, 120, true);

    let mut output = String::new();
    encode_synchronized_frame(&mut output, None, &frame).unwrap();
    assert!(output.contains("h/l ←/→ page • / find"));
    assert!(!output.contains("enter  open"));
    assert!(!output.contains("j/k ↑/↓  choose"));
}

#[test]
fn filter_prompt_aligns_with_browser_content_and_uses_the_blue_palette() {
    assert_eq!(browser_filter_cursor_x("Find: doc", 80), 12);
    assert_eq!(browser_filter_cursor_x("Find: document", 10), 9);
    assert_eq!(browser_filter_prompt_text("Find:", true), "   Find:");
    assert_eq!(
        browser_filter_prompt_text("Find: doc", true),
        "   Find: doc"
    );

    let prompt = browser_filter_prompt_text("Find: doc", false);
    assert!(prompt.starts_with("   "));
    assert!(prompt.contains(&styled("Find:", Some(BROWSER_ACCENT), None, false, false)));
    assert!(prompt.contains(&styled(
        "doc",
        Some(BROWSER_FILTER_INPUT),
        None,
        false,
        false
    )));
}

#[test]
fn filter_help_aligns_with_browser_content_and_has_compact_separators() {
    assert_eq!(
        browser_filter_help(true),
        "   enter confirm • esc cancel • ctrl+j/ctrl+k ↑/↓ choose"
    );
}

#[test]
fn expanded_filter_help_uses_three_aligned_rows() {
    assert_eq!(
        browser_filter_full_help(true),
        [
            "   enter              confirm",
            "   esc                cancel",
            "   ctrl+j/ctrl+k ↑/↓  choose",
        ]
    );
}

#[test]
fn filter_help_controls_the_reserved_footer_height() {
    let documents = vec![DocumentEntry::for_test("README.md")];
    let mut browser = BrowserState::for_test(documents, 24);
    browser.toggle_help();
    browser.begin_filter();

    assert_eq!(browser_footer_rows(24, &browser), (18, 20));
    assert_eq!(browser_help_rows(&browser), 3);
}

#[test]
fn applied_filter_omits_page_navigation_from_mini_help() {
    let documents = (0..12)
        .map(|index| DocumentEntry::for_test(&format!("docs/{index}.md")))
        .collect();
    let mut browser = BrowserState::for_test(documents, 24);
    browser.begin_filter();
    browser.set_filter("docs");
    browser.confirm_filter();

    assert!(browser.page_count() > 1);
    assert_eq!(
        browser_mini_help(&browser, 120, true),
        "   tab section • / edit search • esc clear filter • r refresh • e edit • q quit • ? more"
    );
}

#[test]
fn mini_help_truncates_at_a_segment_boundary() {
    let documents = [
        DocumentEntry::for_test("docs/one.md"),
        DocumentEntry::for_test("docs/two.md"),
    ];
    let mut browser = BrowserState::for_test(documents.into(), 24);
    browser.begin_filter();
    browser.set_filter("docs");
    browser.confirm_filter();

    assert_eq!(
        browser_mini_help(&browser, 64, true),
        "   tab section • / edit search • esc clear filter • …"
    );
}

#[test]
fn filtering_underlines_only_the_fuzzy_match_characters() {
    let document = DocumentEntry::for_test("test_codeblock_indent.md");
    let title = filtered_title(
        &document,
        &document.relative_path,
        "doc",
        rgb(221, 221, 221),
        false,
    );

    assert_eq!(title.matches("\x1b[4m").count(), 3);
    assert_eq!(crate::utils::strip_ansi(&title), "test_codeblock_indent.md");
}

#[test]
fn filtering_without_colors_keeps_the_title_plain() {
    assert_eq!(
        filtered_title(
            &DocumentEntry::for_test("docs/résumé.md"),
            "docs/résumé.md",
            "RESUME",
            rgb(221, 221, 221),
            true,
        ),
        "docs/résumé.md"
    );
}

#[test]
fn filter_editing_never_marks_a_result_as_selected() {
    let documents = [
        DocumentEntry::for_test("DOC.md"),
        DocumentEntry::for_test("docs/sample.md"),
    ];
    let mut browser = BrowserState::for_test(documents.into(), 20);
    browser.begin_filter();
    browser.set_filter("doc");

    assert!(!browser_item_selected(&browser, 0));

    browser.set_filter("sample");

    assert!(!browser_item_selected(&browser, 0));
}

#[test]
fn full_help_uses_aligned_columns_from_browser_column_three() {
    let rows = browser_full_help(true);
    let visual_column = |row: &str, text: &str| {
        let byte_index = row.find(text).unwrap();
        display_width(&row[..byte_index])
    };

    assert_eq!(rows.len(), 4);
    assert_eq!(
        [
            visual_column(&rows[0], "enter"),
            visual_column(&rows[1], "j/k"),
            visual_column(&rows[2], "h/l"),
            visual_column(&rows[3], "g/home"),
        ],
        [3; 4]
    );
    assert_eq!(
        [
            visual_column(&rows[0], "/"),
            visual_column(&rows[1], "esc"),
            visual_column(&rows[2], "tab"),
            visual_column(&rows[3], "G/end"),
        ],
        [22; 4]
    );
    assert_eq!(
        [
            visual_column(&rows[0], "e  edit"),
            visual_column(&rows[1], "!  errors"),
            visual_column(&rows[2], "?  close help"),
        ],
        [40; 3]
    );
    assert_eq!(
        [
            visual_column(&rows[0], "r  refresh"),
            visual_column(&rows[1], "q  quit"),
        ],
        [57; 2]
    );
}

#[test]
fn mini_help_reserves_a_bottom_row() {
    let mut browser = BrowserState::for_test(vec![DocumentEntry::for_test("README.md")], 24);
    assert_eq!(browser_footer_rows(24, &browser), (20, 22));
    browser.toggle_help();
    assert_eq!(browser_footer_rows(24, &browser), (17, 19));
}

#[test]
fn pager_handoff_keeps_the_current_screen_buffer() {
    let mut output = Vec::new();

    write_pager_pause(&mut output).unwrap();
    write_pager_resume(&mut output).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("\x1b[?1049h"));
    assert!(!output.contains("\x1b[?1049l"));
}
