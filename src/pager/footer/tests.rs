use super::*;

fn theme(transparent: bool) -> PagerTheme {
    PagerTheme {
        transparent,
        ..PagerTheme::default()
    }
}

use unicode_width::UnicodeWidthStr;

fn progress(percentage: u8) -> FooterProgress {
    FooterProgress {
        percentage,
        search_position: None,
        line_navigation_position: None,
    }
}

#[test]
fn footer_layout_contains_all_sections() {
    let plain = build_footer("AGENTS.md", progress(22), &theme(false))
        .unwrap()
        .render_plain(80);

    assert_eq!(plain.width(), 80);
    assert!(plain.starts_with(" MDV  AGENTS.md"));
    assert!(plain.ends_with("  22%  ? Help "));
    for transparent in [false, true] {
        for width in [60, 80, 120] {
            let hint =
                build_toc_footer("README.md", progress(22), &theme(transparent), width).unwrap();
            let text = hint.render_plain(width);
            assert_eq!(text.width(), width);
            assert!(text.contains("MDV | README.md") && text.contains("? keys"));
            assert!(!text.contains("TOC:"));
            assert!(text.ends_with("22% | ? Help "));
            if width >= 72 {
                assert!(text.contains("Alt+↑↓ move · ? keys"));
            }
            if transparent {
                assert!(!hint.render(width).contains("48;"));
            }
        }
    }
}

#[test]
fn search_and_line_positions_appear_before_document_progress() {
    let progress = FooterProgress {
        search_position: Some((2, 5)),
        line_navigation_position: Some(50),
        ..progress(22)
    };
    let footer = build_footer("AGENTS.md", progress, &theme(false)).unwrap();
    let plain = footer.render_plain(80);
    let rendered = footer.render(80);
    let transparent = build_footer("AGENTS.md", progress, &theme(true))
        .unwrap()
        .render_plain(80);

    assert!(plain.ends_with(" 2/5 :50  22%  ? Help "));
    assert!(transparent.ends_with(" 2/5 :50  22% | ? Help "));
    assert!(
        rendered.matches("38;2;90;90;90").count() >= 2,
        "{}",
        rendered.escape_debug()
    );
}

#[test]
fn explicit_title_overrides_the_file_name() {
    let footer = PagerFooter::new(Some("Help"), Some(Path::new("README.md")), &theme(false));

    assert_eq!(footer.title, "Help");
}

#[test]
fn footer_uses_expected_colors() {
    let rendered = build_footer("AGENTS.md", progress(22), &theme(false))
        .unwrap()
        .render(80);

    assert!(rendered.contains("38;2;125;125;125"));
    assert!(rendered.contains("38;2;90;90;90"));
    assert!(rendered.contains("48;2;36;36;36"));
    assert!(rendered.matches("48;2;50;50;50").count() >= 2);
    assert!(rendered.ends_with("\x1b[0m"));
    for transparent in [false, true] {
        let theme = PagerTheme {
            transparent,
            title: Some(Color::Red),
            help: Some(Color::Green),
            file_name: Some(Color::Blue),
            progress: Some(Color::Magenta),
            matches: Some(Color::Cyan),
            ..PagerTheme::default()
        };
        let progress = FooterProgress {
            search_position: Some((2, 5)),
            ..progress(22)
        };
        for (toc, footer) in [
            (false, build_footer("AGENTS.md", progress, &theme).unwrap()),
            (
                true,
                build_toc_footer("AGENTS.md", progress, &theme, 100).unwrap(),
            ),
        ] {
            let output = footer.render(100);
            for (code, text, accent) in [
                (91, " MDV ", true),
                (92, "? Help", true),
                (94, "AGENTS.md", false),
                (95, "22%", false),
                (96, "2/5", false),
            ] {
                let prefix = format!("\x1b[{code}m");
                let section = output
                    .match_indices(&prefix)
                    .map(|(start, _)| output[start..].split("\x1b[0m").next().unwrap())
                    .find(|section| section.contains(text))
                    .unwrap_or_else(|| panic!("{text}: {output:?}"));
                if !transparent {
                    let background = if accent && !toc { 50 } else { 36 };
                    assert!(
                        section.contains(&format!("48;2;{background};{background};{background}")),
                        "{output:?}"
                    );
                }
            }
            assert_eq!(output.contains("48;"), !transparent);
        }
    }
}

#[test]
fn transparent_footer_uses_separators_without_background() {
    let footer = build_footer("AGENTS.md", progress(22), &theme(true)).unwrap();
    let plain = footer.render_plain(80);

    assert!(plain.starts_with(" MDV | AGENTS.md"));
    assert!(plain.ends_with("  22% | ? Help "));
    assert!(!plain.contains("|  22%"));
    assert_eq!(plain.matches('|').count(), 2);
    assert!(!footer.render(80).contains("\x1b[48;"));
}

#[test]
fn long_unicode_file_name_is_truncated_to_terminal_width() {
    let plain = build_footer("very-long-file-name-📚.md", progress(7), &theme(false))
        .unwrap()
        .render_plain(32);

    assert_eq!(plain.width(), 32);
    assert!(plain.contains('…'));
}

#[test]
fn narrow_footer_never_exceeds_terminal_width() {
    let footer = build_footer("README.md", progress(100), &theme(false)).unwrap();
    for columns in 0..20 {
        assert_eq!(footer.render_plain(columns).width(), columns);
    }
}
