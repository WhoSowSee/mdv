use minus::{PromptColor, PromptError, PromptLine, PromptSpan, PromptStyle};

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
const LEFT_COLUMN_WIDTH: usize = 30;

pub(super) fn build_help_panel(
    editor_enabled: bool,
    reload_enabled: bool,
) -> Result<Vec<PromptLine>, PromptError> {
    let style = PromptStyle::default()
        .foreground(HELP_FOREGROUND)
        .background(HELP_BACKGROUND);
    let edit = editor_enabled.then_some("e/E     edit this document");
    let reload = reload_enabled.then_some("r       reload this document");

    [
        None,
        Some(("k/↑      up", Some("g/home  go to top"))),
        Some(("j/↓      down", Some("G/end   go to bottom"))),
        Some(("b/pgup   page up", Some("c       copy contents"))),
        Some(("f/pgdn   page down", edit)),
        Some(("u        ½ page up", reload)),
        Some(("d        ½ page down", Some("/       search"))),
        Some(("q        quit", Some("esc/?   close help"))),
        None,
    ]
    .into_iter()
    .map(|columns| help_line(columns, style))
    .collect()
}

fn help_line(
    columns: Option<(&str, Option<&str>)>,
    style: PromptStyle,
) -> Result<PromptLine, PromptError> {
    let text = columns.map_or_else(String::new, |(left, right)| {
        right.map_or_else(
            || format!("  {left}"),
            |right| format!("  {left:<LEFT_COLUMN_WIDTH$}{right}"),
        )
    });

    Ok(PromptLine::new()
        .left(PromptSpan::new(text, style)?)
        .fill_style(style))
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn help_panel_contains_mdv_and_glow_shortcuts() {
        let lines = build_help_panel(true, true).unwrap();
        let text = lines
            .iter()
            .map(|line| line.render_plain(100))
            .collect::<Vec<_>>()
            .join("\n");

        for shortcut in [
            "k/↑      up",
            "j/↓      down",
            "b/pgup   page up",
            "f/pgdn   page down",
            "u        ½ page up",
            "d        ½ page down",
            "g/home  go to top",
            "G/end   go to bottom",
            "c       copy contents",
            "e/E     edit this document",
            "r       reload this document",
            "/       search",
            "q        quit",
            "esc/?   close help",
        ] {
            assert!(text.contains(shortcut), "missing shortcut: {shortcut}");
        }
        assert!(!text.contains("q/esc"));
        assert!(!text.contains("space"));
        let lowercase = text.to_ascii_lowercase();
        assert!(!lowercase.contains("ctrl+f"));
        assert!(!lowercase.contains("c-f"));
    }

    #[test]
    fn help_panel_has_symmetric_vertical_padding() {
        let lines = build_help_panel(true, true).unwrap();

        assert!(lines.first().unwrap().render_plain(80).trim().is_empty());
        assert!(lines.last().unwrap().render_plain(80).trim().is_empty());
    }

    #[test]
    fn help_panel_omits_unavailable_file_actions() {
        let lines = build_help_panel(false, false).unwrap();
        let text = lines
            .iter()
            .map(|line| line.render_plain(100))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(!text.contains("edit this document"));
        assert!(!text.contains("reload this document"));
    }

    #[test]
    fn help_panel_fills_the_terminal_width() {
        let lines = build_help_panel(true, true).unwrap();

        for columns in [20, 80, 120] {
            assert!(
                lines
                    .iter()
                    .all(|line| line.render_plain(columns).width() == columns)
            );
        }
    }

    #[test]
    fn help_panel_uses_glow_colors() {
        let rendered = build_help_panel(true, true).unwrap()[1].render(80);

        assert!(rendered.contains("38;2;125;125;125"));
        assert!(rendered.contains("48;2;27;27;27"));
        assert!(rendered.ends_with("\x1b[0m"));
    }
}
