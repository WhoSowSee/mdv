fn theme(transparent: bool) -> crate::theme::PagerTheme {
    crate::theme::PagerTheme {
        transparent,
        ..Default::default()
    }
}
use super::*;

#[test]
fn help_panel_contains_and_orders_expected_shortcuts() {
    let lines = build_help_panel(
        PagerCapabilities {
            editor_enabled: true,
            reload_enabled: true,
            line_navigation_enabled: true,
            line_number_toggle_enabled: true,
        },
        &theme(false),
    )
    .unwrap();
    let rendered_lines = lines
        .iter()
        .map(|line| line.render_plain(80))
        .collect::<Vec<_>>();
    let text = rendered_lines.join("\n");

    for shortcut in [
        "k/↑      up",
        "j/↓      down",
        "b/pgup   page up",
        "f/pgdn   page down",
        "u        ½ page up",
        "d        ½ page down",
        "g/home   go to top",
        "G/end    go to bottom",
        "c       copy contents",
        "e       edit document",
        "r       reload document",
        "/        search",
        "q       quit",
        "?       toggle help",
        "esc     close help",
        ":        source line",
        "l        line numbers",
    ] {
        assert!(text.contains(shortcut), "missing shortcut: {shortcut}");
    }
    assert!(!text.contains("q/esc"));
    assert!(!text.contains("space"));
    let lowercase = text.to_ascii_lowercase();
    assert!(!lowercase.contains("ctrl+f"));
    assert!(!lowercase.contains("c-f"));
    assert!(!lowercase.contains("ctrl+l"));
    let first_row = &rendered_lines[1];
    assert_eq!(display_column(first_row, "b/pgup"), 2);
    assert_eq!(display_column(first_row, "g/home"), 2 + HELP_COLUMN_WIDTH);
    assert_eq!(
        display_column(first_row, "esc     close help"),
        2 + HELP_COLUMN_WIDTH * 2
    );
    assert!(rendered_lines[4].contains("r       reload document"));
    assert!(!text.contains("TOC entries"));
    let toc_rows = build_toc_help_panel(80, &theme(true)).unwrap();
    let toc = toc_rows
        .iter()
        .map(|line| line.render_plain(80))
        .collect::<Vec<_>>()
        .join("\n");
    for keys in [
        "1–9",
        "Alt+↑/↓",
        "Shift+↑/↓ or J/K",
        "Alt+Shift+↑/↓ or U/D",
        "? / Esc",
    ] {
        assert!(toc.contains(keys), "{toc}");
    }
    assert!(!toc.contains("reload document") && !toc.contains("copy contents"));
    let first = toc_rows[1].render_plain(80);
    assert_eq!(toc_rows.len(), 6);
    assert!(first.contains("1–9") && first.contains("Alt+Shift+↑/↓"));
    for (index, left, right) in [
        (1, "select / cycle", "scroll"),
        (2, "entries", "scroll"),
        (3, "sections", "close contents"),
        (4, "select", "close help"),
    ] {
        let row = toc_rows[index].render_plain(80);
        assert_eq!(
            display_column(&row, left),
            display_column(&first, "select / cycle")
        );
        assert_eq!(
            display_column(&row, right),
            display_column(&first, "scroll")
        );
    }
}

#[test]
fn help_panel_has_symmetric_vertical_padding() {
    let lines = build_help_panel(
        PagerCapabilities {
            editor_enabled: true,
            reload_enabled: true,
            line_navigation_enabled: true,
            line_number_toggle_enabled: true,
        },
        &theme(false),
    )
    .unwrap();

    assert!(lines.first().unwrap().render_plain(80).trim().is_empty());
    assert!(lines.last().unwrap().render_plain(80).trim().is_empty());
}

#[test]
fn help_panel_omits_unavailable_actions() {
    let lines = build_help_panel(
        PagerCapabilities {
            editor_enabled: false,
            reload_enabled: false,
            line_navigation_enabled: false,
            line_number_toggle_enabled: false,
        },
        &theme(false),
    )
    .unwrap();
    let text = lines
        .iter()
        .map(|line| line.render_plain(100))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!text.contains("edit document"));
    assert!(!text.contains("reload document"));
    assert!(!text.contains("source line"));
    assert!(!text.contains("line numbers"));
}

#[test]
fn help_panel_fills_the_terminal_width() {
    let lines = build_help_panel(
        PagerCapabilities {
            editor_enabled: true,
            reload_enabled: true,
            line_navigation_enabled: true,
            line_number_toggle_enabled: true,
        },
        &theme(false),
    )
    .unwrap();

    for columns in [20, 80, 120] {
        let fitted = fit_help_panel(&lines, columns, &theme(false)).unwrap();
        assert!(
            fitted
                .iter()
                .all(|line| line.render_plain(columns).width() == columns)
        );
        let text = fitted
            .iter()
            .map(|line| line.render_plain(columns))
            .collect::<Vec<_>>()
            .join("\n");
        for shortcut in [
            "b/pgup", "g/home", "esc", "c       ", "e       ", "q       ",
        ] {
            assert!(
                text.contains(shortcut),
                "missing {shortcut} at width {columns}"
            );
        }
    }
}

fn display_column(line: &str, text: &str) -> usize {
    let byte_index = line.find(text).expect("help item");
    line[..byte_index].width()
}

#[test]
fn help_colors_survive_layout_changes_and_theme_overrides() {
    for transparent in [false, true] {
        let themes = [
            (theme(transparent), "38;2;125;125;125"),
            (
                crate::theme::PagerTheme {
                    help: Some(crate::theme::Color::Rgb {
                        r: 12,
                        g: 34,
                        b: 56,
                    }),
                    ..theme(transparent)
                },
                "38;2;12;34;56",
            ),
        ];
        for (theme, key_color) in themes {
            let lines = build_help_panel(
                PagerCapabilities {
                    editor_enabled: true,
                    reload_enabled: true,
                    line_navigation_enabled: true,
                    line_number_toggle_enabled: true,
                },
                &theme,
            )
            .unwrap();
            for width in [20, 60, 80] {
                let rendered = fit_help_panel(&lines, width, &theme)
                    .unwrap()
                    .iter()
                    .map(|line| line.render(width))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert_foreground(&rendered, "b/pgup", key_color);
                assert_foreground(&rendered, "page up", "38;2;73;73;73");
                if transparent {
                    assert!(!rendered.contains("\x1b[48;"));
                } else {
                    assert!(rendered.contains("48;2;27;27;27"));
                }
            }
        }
    }
}

#[test]
fn toc_help_keeps_distinct_description_colors_when_wrapped() {
    for width in [8, 20, 48, 80] {
        let lines = build_toc_help_panel(width, &theme(true)).unwrap();
        let rendered = lines
            .iter()
            .map(|line| line.render(width))
            .collect::<Vec<_>>()
            .join("\n");
        assert_foreground(&rendered, "1–9", "38;2;125;125;125");
        assert_foreground(&rendered, "select", "38;2;73;73;73");
        assert_foreground(&rendered, "scroll", "38;2;73;73;73");
    }
}

fn assert_foreground(rendered: &str, text: &str, expected: &str) {
    let prefix = &rendered[..rendered.find(text).expect(text)];
    let mut foreground = None;
    for sequence in prefix.split("\x1b[").skip(1) {
        let Some((parameters, _)) = sequence.split_once('m') else {
            continue;
        };
        if parameters == "0" {
            foreground = None;
        } else if parameters.starts_with("38;") {
            foreground = Some(parameters);
        }
    }
    assert_eq!(foreground, Some(expected), "wrong color for {text}");
}
