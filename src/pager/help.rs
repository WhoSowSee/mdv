use minus::{PromptColor, PromptError, PromptLine, PromptSpan, PromptStyle};
use std::cmp::Reverse;
use unicode_width::UnicodeWidthStr;

const HELP_BACKGROUND: PromptColor = PromptColor::Rgb {
    r: 27,
    g: 27,
    b: 27,
};
const HELP_COLUMN_WIDTH: usize = 26;

fn help_style(theme: &crate::theme::PagerTheme) -> PromptStyle {
    let style = super::styling::foreground(PromptStyle::default(), theme.help.as_ref());
    if theme.transparent {
        style
    } else {
        style.background(HELP_BACKGROUND)
    }
}

pub(super) fn build_toc_help_panel(
    width: usize,
    theme: &crate::theme::PagerTheme,
) -> Result<Vec<PromptLine>, PromptError> {
    let style = help_style(theme);
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
    theme: &crate::theme::PagerTheme,
) -> Result<Vec<PromptLine>, PromptError> {
    if width >= 78 {
        return Ok(lines.to_vec());
    }
    let style = help_style(theme);
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

pub(super) struct PagerCapabilities {
    pub(super) editor_enabled: bool,
    pub(super) reload_enabled: bool,
    pub(super) line_navigation_enabled: bool,
    pub(super) line_number_toggle_enabled: bool,
}

pub(super) fn build_help_panel(
    capabilities: PagerCapabilities,
    theme: &crate::theme::PagerTheme,
) -> Result<Vec<PromptLine>, PromptError> {
    let PagerCapabilities {
        editor_enabled,
        reload_enabled,
        line_navigation_enabled,
        line_number_toggle_enabled,
    } = capabilities;
    let style = help_style(theme);
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
mod tests;
