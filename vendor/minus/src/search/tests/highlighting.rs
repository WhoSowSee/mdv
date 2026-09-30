#![allow(clippy::trivial_regex)]
use crate::search::{INVERT, NORMAL, highlight_line_matches};
use crossterm::style::Attribute;
use regex::Regex;

const ESC: &str = "\x1b[34m";
const NONE: &str = "\x1b[0m";

mod consistent {
    use super::*;

    #[test]
    fn test_highlight_matches() {
        let line = "Integer placerat tristique nisl. placerat non mollis, magna orci dolor, placerat at vulputate neque nulla lacinia eros.".to_string();
        let pat = Regex::new(r"\W\w+t\W").unwrap();
        let result = format!(
            "Integer{inverse} placerat {noinverse}tristique nisl.\
{inverse} placerat {noinverse}non mollis, magna orci dolor,\
{inverse} placerat {noinverse}at vulputate neque nulla lacinia \
eros.",
            inverse = Attribute::Reverse,
            noinverse = Attribute::NoReverse
        );

        assert_eq!(highlight_line_matches(&line, &pat, false).0, result);
    }

    #[test]
    fn no_match() {
        let orig = "no match";
        let res = highlight_line_matches(orig, &Regex::new("test").unwrap(), false);
        assert_eq!(res.0, orig.to_string());
    }

    #[test]
    fn single_match_no_esc() {
        let res = highlight_line_matches("this is a test", &Regex::new(" a ").unwrap(), false);
        assert_eq!(res.0, format!("this is{} a {}test", *INVERT, *NORMAL));
    }

    #[test]
    fn multi_match_no_esc() {
        let res = highlight_line_matches("test another test", &Regex::new("test").unwrap(), false);
        assert_eq!(
            res.0,
            format!("{i}test{n} another {i}test{n}", i = *INVERT, n = *NORMAL)
        );
    }

    #[test]
    fn esc_pair_outside_match() {
        let res = highlight_line_matches(
            &format!("{ESC}color{NONE} and test"),
            &Regex::new("test").unwrap(),
            false,
        );
        assert_eq!(
            res.0,
            format!("{}color{} and {}test{}", ESC, NONE, *INVERT, *NORMAL)
        );
    }

    #[test]
    fn esc_pair_end_in_match() {
        let orig = format!("this {ESC}is a te{NONE}st");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
        assert_eq!(
            res.0,
            format!("this {}is a {}test{}{}", ESC, *INVERT, *NORMAL, NONE)
        );
    }

    #[test]
    fn esc_pair_start_in_match() {
        let orig = format!("this is a te{ESC}st again{NONE}");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
        assert_eq!(
            res.0,
            format!("this is a {}test{}{ESC} again{}", *INVERT, *NORMAL, NONE)
        );
    }

    #[test]
    fn esc_pair_around_match() {
        let orig = format!("this is {ESC}a test again{NONE}");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
        assert_eq!(
            res.0,
            format!("this is {}a {}test{} again{}", ESC, *INVERT, *NORMAL, NONE)
        );
    }

    #[test]
    fn esc_pair_within_match() {
        let orig = format!("this is a t{ESC}es{NONE}t again");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
        assert_eq!(
            res.0,
            format!("this is a {}test{}{ESC}{NONE} again", *INVERT, *NORMAL)
        );
    }

    #[test]
    fn multi_escape_match() {
        let orig = format!("this {ESC}is a te{NONE}st again {ESC}yeah{NONE} test");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), false);
        assert_eq!(
            res.0,
            format!(
                "this {e}is a {i}test{n}{nn} again {e}yeah{nn} {i}test{n}",
                e = ESC,
                i = *INVERT,
                n = *NORMAL,
                nn = NONE
            )
        );
    }
}
mod accurate {
    use super::*;
    #[test]
    fn correct_ascii_sequence_placement() {
        let orig = format!("{ESC}test{NONE} this {ESC}is a te{NONE}st again {ESC}yeah{NONE} test",);

        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
        assert_eq!(
            res.0,
            format!(
                "{i}{e}test{n}{nn} this {e}is a {i}te{NONE}st{n} again {e}yeah{nn} {i}test{n}",
                e = ESC,
                i = *INVERT,
                n = *NORMAL,
                nn = NONE
            )
        );
    }

    #[test]
    fn esc_pair_outside_match() {
        let res = highlight_line_matches(
            &format!("{ESC}color{NONE} and test"),
            &Regex::new("test").unwrap(),
            true,
        );
        assert_eq!(
            res.0,
            format!("{}color{} and {}test{}", ESC, NONE, *INVERT, *NORMAL)
        );
    }

    #[test]
    fn esc_pair_end_in_match() {
        let orig = format!("this {ESC}is a te{NONE}st");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
        assert_eq!(
            res.0,
            format!("this {ESC}is a {}te{NONE}st{}", *INVERT, *NORMAL)
        );
    }

    #[test]
    fn esc_pair_start_in_match() {
        let orig = format!("this is a te{ESC}st again{NONE}");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
        assert_eq!(
            res.0,
            format!("this is a {}te{ESC}st{} again{NONE}", *INVERT, *NORMAL)
        );
    }

    #[test]
    fn esc_pair_around_match() {
        let orig = format!("this is {ESC}a test again{NONE}");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
        assert_eq!(
            res.0,
            format!("this is {ESC}a {}test{} again{NONE}", *INVERT, *NORMAL)
        );
    }

    #[test]
    fn esc_pair_within_match() {
        let orig = format!("this is a t{ESC}es{NONE}t again");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
        assert_eq!(
            res.0,
            format!("this is a {}t{ESC}es{NONE}t{} again", *INVERT, *NORMAL)
        );
    }

    #[test]
    fn multi_escape_match() {
        let orig = format!("this {ESC}is a te{NONE}st again {ESC}yeah{NONE} test");
        let res = highlight_line_matches(&orig, &Regex::new("test").unwrap(), true);
        assert_eq!(
            res.0,
            format!(
                "this {e}is a {i}te{nn}st{n} again {e}yeah{nn} {i}test{n}",
                e = ESC,
                i = *INVERT,
                n = *NORMAL,
                nn = NONE
            )
        );
    }
}
