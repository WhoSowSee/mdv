use super::*;

#[test]
fn mapped_replacement_preserves_source_position_and_clears_selection() {
    let mut pager = PagerState::new().unwrap();
    pager.rows = 3;
    pager.cols = 80;
    pager
        .replace_mapped_text(
            "first\nsecond\nthird\nfourth\nfifth".into(),
            Some(LineNavigation::from_current(vec![
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
            ])),
        )
        .unwrap();
    pager.upper_mark = 1;
    pager.selection = Some(crate::state::Selection {
        absolute_row: 1,
        col: 1,
    });
    pager.selection_anchor = pager.selection;
    pager.begin_line_navigation().unwrap();
    pager.finish_line_navigation(Some(2)).unwrap();
    pager.selection = Some(crate::state::Selection {
        absolute_row: 1,
        col: 1,
    });
    pager.selection_anchor = pager.selection;
    pager
        .replace_mapped_text(
            "1 first\n  continuation\n2 second\n3 third\n4 fourth\n5 fifth".into(),
            Some(LineNavigation::from_current(vec![
                Some(1),
                None,
                Some(2),
                Some(3),
                Some(4),
                Some(5),
            ])),
        )
        .unwrap();
    assert_eq!(pager.upper_mark, 2);
    assert!(pager.selection.is_none());
    assert!(pager.selection_anchor.is_none());
    assert!(!pager.line_navigation_is_active());
    assert!(pager.begin_line_navigation().unwrap());
    assert!(pager.finish_line_navigation(Some(4)).unwrap());
    assert_eq!(pager.upper_mark, 4);
}

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
