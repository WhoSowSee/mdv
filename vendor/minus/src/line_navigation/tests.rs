use super::*;

#[test]
fn navigation_preserves_state_across_wrapped_views() {
    let mut pager = PagerState::new().unwrap();
    pager.cols = 6;
    pager.screen.orig_text = "first long line\nsecond\nthird".to_string();
    pager.screen.line_count = 3;
    pager.reformat_display().unwrap();
    pager.upper_mark = pager.lines_to_row_map.get(1).copied().unwrap();
    pager.left_mark = 4;
    pager.line_navigation = Some(LineNavigation::new(
        "1 first long line\n7 second\n9 third",
        vec![Some(1), Some(7), Some(9)],
        vec![Some(1), Some(7), Some(9)],
    ));

    assert!(pager.begin_line_navigation().unwrap());
    let target_row = pager.row_for_source_line(7, true).unwrap();
    assert_eq!(
        pager.screen.orig_text,
        "1 first long line\n7 second\n9 third"
    );
    assert_eq!(pager.left_mark, 0);
    assert!(pager.finish_line_navigation(Some(7)).unwrap());

    assert_eq!(pager.upper_mark, target_row);
    assert_eq!(pager.left_mark, 0);
    assert!(pager.line_navigation_is_active());
    assert_eq!(pager.line_navigation_position(), Some(7));
    assert!(pager.source_line_is_highlighted(target_row));

    pager.upper_mark = 0;
    assert!(pager.begin_line_navigation().unwrap());
    assert!(!pager.finish_line_navigation(None).unwrap());
    assert_eq!(
        pager.screen.orig_text,
        "1 first long line\n7 second\n9 third"
    );
    assert_eq!(pager.upper_mark, 0);
    assert_eq!(pager.line_navigation_position(), Some(7));

    assert!(pager.exit_line_navigation().unwrap());
    assert_eq!(pager.screen.orig_text, "first long line\nsecond\nthird");
    assert_eq!(pager.upper_mark, 0);
    assert_eq!(pager.left_mark, 4);
    assert!(!pager.line_navigation_is_active());
}
