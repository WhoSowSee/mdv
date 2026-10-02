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

#[test]
fn help_panel_uses_expected_colors() {
    let rendered = build_help_panel(
        PagerCapabilities {
            editor_enabled: true,
            reload_enabled: true,
            line_navigation_enabled: true,
            line_number_toggle_enabled: true,
        },
        &theme(false),
    )
    .unwrap()[1]
        .render(80);

    assert!(rendered.contains("38;2;125;125;125"));
    assert!(rendered.contains("48;2;27;27;27"));
    assert!(rendered.ends_with("\x1b[0m"));
}

#[test]
fn transparent_help_panel_does_not_set_a_background() {
    let rendered = build_help_panel(
        PagerCapabilities {
            editor_enabled: true,
            reload_enabled: true,
            line_navigation_enabled: true,
            line_number_toggle_enabled: true,
        },
        &theme(true),
    )
    .unwrap()[1]
        .render(80);

    assert!(rendered.contains("38;2;125;125;125"));
    assert!(!rendered.contains("\x1b[48;"));
    assert!(rendered.ends_with("\x1b[0m"));
}

fn display_column(line: &str, text: &str) -> usize {
    let byte_index = line.find(text).expect("help item");
    line[..byte_index].width()
}
