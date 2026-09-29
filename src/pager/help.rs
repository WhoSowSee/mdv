use minus::{PromptColor, PromptError, PromptLine, PromptSpan, PromptStyle};
use std::cmp::Reverse;
use unicode_width::UnicodeWidthStr;

const HELP_FOREGROUND: PromptColor = PromptColor::Rgb {
    r: 125,
    g: 125,
    b: 125,
};
const HELP_BACKGROUND: PromptColor = PromptColor::Rgb {
    r: 27,
    g: 27,
    b: 27,
};
const HELP_COLUMN_WIDTH: usize = 26;

fn help_style(transparent: bool) -> PromptStyle {
    let style = PromptStyle::default().foreground(HELP_FOREGROUND);
    if transparent {
        style
    } else {
        style.background(HELP_BACKGROUND)
    }
}

pub(super) fn build_toc_help_panel(
    width: usize,
    transparent: bool,
) -> Result<Vec<PromptLine>, PromptError> {
    let style = help_style(transparent);
    let groups = [
        [
            ("1–9", "select / cycle"),
            ("Alt+↑/↓", "entries"),
            ("Shift+↑/↓ or J/K", "sections"),
            ("Mouse click", "select"),
        ],
        [
            ("Alt+Shift+↑/↓ or U/D", "scroll"),
            ("Mouse wheel", "scroll"),
            ("t", "close contents"),
            ("? / Esc", "close help"),
        ],
    ];
    let groups = groups.map(|group| {
        let key_width = group
            .iter()
            .map(|(keys, _)| keys.width())
            .max()
            .unwrap_or(0);
        group.map(|(keys, description)| {
            format!(
                "{keys}{}{description}",
                " ".repeat(key_width - keys.width() + 2)
            )
        })
    });
    let mut lines = vec![help_line(None, style)?];
    let required_width = groups
        .iter()
        .flatten()
        .map(|text| text.width())
        .max()
        .unwrap_or(0)
        * 2
        + 4;
    let texts = if width >= required_width {
        let column = width.saturating_sub(4) / 2;
        groups[0]
            .iter()
            .zip(groups[1].iter())
            .map(|(left, right)| {
                format!(
                    "  {left}{}  {right}",
                    " ".repeat(column.saturating_sub(left.width()))
                )
            })
            .collect::<Vec<_>>()
    } else {
        groups
            .into_iter()
            .flatten()
            .map(|text| format!("  {text}"))
            .collect()
    };
    for text in texts {
        for part in
            crate::utils::wrap_text_with_mode(&text, width.max(1), crate::utils::WrapMode::Word)
                .lines()
        {
            lines.push(
                PromptLine::new()
                    .left(PromptSpan::new(part, style)?)
                    .fill_style(style),
            );
        }
    }
    lines.push(help_line(None, style)?);
    Ok(lines)
}

pub(super) fn fit_help_panel(
    lines: &[PromptLine],
    width: usize,
    transparent: bool,
) -> Result<Vec<PromptLine>, PromptError> {
    if width >= 78 {
        return Ok(lines.to_vec());
    }
    let style = help_style(transparent);
    let columns = if width >= 52 { 2 } else { 1 };
    let mut items = Vec::new();
    for column in 0..3 {
        for line in lines {
            let text = line.render_plain(80);
            let item = text
                .chars()
                .skip(2 + column * HELP_COLUMN_WIDTH)
                .take(HELP_COLUMN_WIDTH)
                .collect::<String>();
            if !item.trim().is_empty() {
                items.push(item.trim_end().to_owned());
            }
        }
    }
    let mut result = vec![help_line(None, style)?];
    for row in items.chunks(columns) {
        let text = row
            .iter()
            .map(|item| format!("{item:<HELP_COLUMN_WIDTH$}"))
            .collect::<String>();
        result.push(
            PromptLine::new()
                .left(PromptSpan::new(format!("  {text}"), style)?)
                .fill_style(style),
        );
    }
    result.push(help_line(None, style)?);
    Ok(result)
}

pub(super) fn build_help_panel(
    editor_enabled: bool,
    reload_enabled: bool,
    line_navigation_enabled: bool,
    line_number_toggle_enabled: bool,
    transparent: bool,
) -> Result<Vec<PromptLine>, PromptError> {
    let style = help_style(transparent);
    let scrolling = longest_shortcuts_first([
        "k/↑      up",
        "j/↓      down",
        "b/pgup   page up",
        "f/pgdn   page down",
        "u        ½ page up",
        "d        ½ page down",
    ]);
    let navigation = longest_shortcuts_first(
        [
            Some("g/home   go to top"),
            Some("G/end    go to bottom"),
            Some("/        search"),
            line_navigation_enabled.then_some("t        contents"),
            line_navigation_enabled.then_some(":        source line"),
            line_number_toggle_enabled.then_some("l        line numbers"),
        ]
        .into_iter()
        .flatten(),
    );
    let actions = longest_shortcuts_first(
        [
            Some("c       copy contents"),
            editor_enabled.then_some("e       edit document"),
            reload_enabled.then_some("r       reload document"),
            Some("?       toggle help"),
            Some("esc     close help"),
            Some("q       quit"),
        ]
        .into_iter()
        .flatten(),
    );

    let row_count = scrolling.len().max(navigation.len()).max(actions.len());
    let mut rows = Vec::with_capacity(row_count + 2);
    rows.push(help_line(None, style)?);
    for index in 0..row_count {
        rows.push(help_line(
            Some([
                scrolling.get(index).copied(),
                navigation.get(index).copied(),
                actions.get(index).copied(),
            ]),
            style,
        )?);
    }
    rows.push(help_line(None, style)?);
    Ok(rows)
}

fn longest_shortcuts_first<'a>(items: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut items = items.into_iter().collect::<Vec<_>>();
    items.sort_by_key(|item| Reverse(shortcut_width(item)));
    items
}

fn shortcut_width(item: &str) -> usize {
    item.split_whitespace()
        .next()
        .map_or(0, UnicodeWidthStr::width)
}

fn help_line(
    columns: Option<[Option<&str>; 3]>,
    style: PromptStyle,
) -> Result<PromptLine, PromptError> {
    let text = columns.map_or_else(String::new, |[left, middle, right]| {
        format!(
            "  {:<HELP_COLUMN_WIDTH$}{:<HELP_COLUMN_WIDTH$}{}",
            left.unwrap_or_default(),
            middle.unwrap_or_default(),
            right.unwrap_or_default(),
        )
    });

    Ok(PromptLine::new()
        .left(PromptSpan::new(text, style)?)
        .fill_style(style))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_panel_contains_and_orders_expected_shortcuts() {
        let lines = build_help_panel(true, true, true, true, false).unwrap();
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
        let toc_rows = build_toc_help_panel(80, true).unwrap();
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
        let lines = build_help_panel(true, true, true, true, false).unwrap();

        assert!(lines.first().unwrap().render_plain(80).trim().is_empty());
        assert!(lines.last().unwrap().render_plain(80).trim().is_empty());
    }

    #[test]
    fn help_panel_omits_unavailable_actions() {
        let lines = build_help_panel(false, false, false, false, false).unwrap();
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
        let lines = build_help_panel(true, true, true, true, false).unwrap();

        for columns in [20, 80, 120] {
            let fitted = fit_help_panel(&lines, columns, false).unwrap();
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
        let rendered = build_help_panel(true, true, true, true, false).unwrap()[1].render(80);

        assert!(rendered.contains("38;2;125;125;125"));
        assert!(rendered.contains("48;2;27;27;27"));
        assert!(rendered.ends_with("\x1b[0m"));
    }

    #[test]
    fn transparent_help_panel_does_not_set_a_background() {
        let rendered = build_help_panel(true, true, true, true, true).unwrap()[1].render(80);

        assert!(rendered.contains("38;2;125;125;125"));
        assert!(!rendered.contains("\x1b[48;"));
        assert!(rendered.ends_with("\x1b[0m"));
    }

    fn display_column(line: &str, text: &str) -> usize {
        let byte_index = line.find(text).expect("help item");
        line[..byte_index].width()
    }
}
