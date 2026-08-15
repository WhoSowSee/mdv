use super::*;

pub(super) fn browser_mini_help(browser: &BrowserState, width: usize, no_colors: bool) -> String {
    let entries = if browser.filter_state() == FilterState::Applied {
        BROWSER_FILTERED_MINI_HELP
    } else {
        BROWSER_MINI_HELP
    };
    let max_width = width.saturating_sub(1);
    if max_width == 0 {
        return String::new();
    }

    let mut help = String::from("   ");
    let mut help_width = display_width(&help);
    for (index, &(key, label)) in entries.iter().enumerate() {
        let has_next = index + 1 < entries.len();
        let entry_width =
            display_width(key) + 1 + display_width(label) + if has_next { 3 } else { 0 };
        let truncation_width = usize::from(has_next);
        if help_width + entry_width + truncation_width > max_width {
            if help_width + display_width(ELLIPSIS) <= max_width {
                help.push_str(&styled(
                    ELLIPSIS,
                    Some(BROWSER_HELP_SEPARATOR),
                    None,
                    false,
                    no_colors,
                ));
            }
            break;
        }

        help.push_str(&styled(key, Some(BROWSER_HELP_KEY), None, false, no_colors));
        help.push(' ');
        help.push_str(&styled(
            label,
            Some(BROWSER_HELP_LABEL),
            None,
            false,
            no_colors,
        ));
        if has_next {
            help.push(' ');
            help.push_str(&styled(
                "•",
                Some(BROWSER_HELP_SEPARATOR),
                None,
                false,
                no_colors,
            ));
            help.push(' ');
        }
        help_width += entry_width;
    }
    help
}

pub(super) fn browser_filter_help(no_colors: bool) -> String {
    let segments = [
        ("enter", BROWSER_HELP_KEY),
        ("confirm", BROWSER_HELP_LABEL),
        ("•", BROWSER_HELP_SEPARATOR),
        ("esc", BROWSER_HELP_KEY),
        ("cancel", BROWSER_HELP_LABEL),
        ("•", BROWSER_HELP_SEPARATOR),
        ("ctrl+j/ctrl+k ↑/↓", BROWSER_HELP_KEY),
        ("choose", BROWSER_HELP_LABEL),
    ];
    let mut help = String::from("   ");
    for (index, (text, color)) in segments.into_iter().enumerate() {
        if index > 0 {
            help.push(' ');
        }
        help.push_str(&styled(text, Some(color), None, false, no_colors));
    }
    help
}

pub(super) fn browser_filter_full_help(no_colors: bool) -> Vec<String> {
    let entries = [
        ("enter", "confirm"),
        ("esc", "cancel"),
        ("ctrl+j/ctrl+k ↑/↓", "choose"),
    ];
    let key_width = entries
        .iter()
        .map(|(key, _)| display_width(key))
        .max()
        .unwrap_or(0);
    entries
        .into_iter()
        .map(|(key, label)| {
            format!(
                "   {}{}{}",
                styled(key, Some(BROWSER_HELP_KEY), None, false, no_colors),
                " ".repeat(key_width - display_width(key) + 2),
                styled(label, Some(BROWSER_HELP_LABEL), None, false, no_colors)
            )
        })
        .collect()
}

pub(super) fn browser_full_help(no_colors: bool) -> Vec<String> {
    let column_widths: [(usize, usize); 4] = std::array::from_fn(|column| {
        BROWSER_FULL_HELP_ROWS
            .iter()
            .fold((0, 0), |(key_width, label_width), row| match row[column] {
                Some((key, label)) => (
                    key_width.max(display_width(key)),
                    label_width.max(display_width(label)),
                ),
                None => (key_width, label_width),
            })
    });

    BROWSER_FULL_HELP_ROWS
        .iter()
        .map(|row| {
            let last_column = row.iter().rposition(Option::is_some).unwrap_or(0);
            let mut line = String::from("   ");
            for column in 0..=last_column {
                if column > 0 {
                    line.push_str("    ");
                }
                let (key_width, label_width) = column_widths[column];
                match row[column] {
                    Some((key, label)) => {
                        line.push_str(&styled(key, Some(BROWSER_HELP_KEY), None, false, no_colors));
                        line.push_str(&" ".repeat(key_width - display_width(key) + 2));
                        line.push_str(&styled(
                            label,
                            Some(BROWSER_HELP_LABEL),
                            None,
                            false,
                            no_colors,
                        ));
                        if column < last_column {
                            line.push_str(&" ".repeat(label_width - display_width(label)));
                        }
                    }
                    None => line.push_str(&" ".repeat(key_width + 2 + label_width)),
                }
            }
            line
        })
        .collect()
}

pub(super) fn item_prefix(selected: bool, no_colors: bool) -> String {
    let gutter = if selected {
        styled("│", Some(BROWSER_ACCENT), None, false, no_colors)
    } else {
        " ".to_string()
    };
    format!(" {gutter} ")
}

pub(super) fn browser_help_rows(browser: &BrowserState) -> u16 {
    if browser.filter_state() == FilterState::Editing && browser.show_full_help() {
        3
    } else if browser.show_full_help() {
        4
    } else {
        1
    }
}

pub(super) fn browser_footer_rows(height: u16, browser: &BrowserState) -> (u16, u16) {
    let help_rows = browser_help_rows(browser);
    let help_y = height.saturating_sub(help_rows + 1);
    (help_y.saturating_sub(2), help_y)
}

pub(super) fn draw_browser_help(
    stdout: &mut Stdout,
    browser: &BrowserState,
    start_y: u16,
    width: usize,
    no_colors: bool,
) -> Result<()> {
    if browser.filter_state() != FilterState::Editing && !browser.show_full_help() {
        return write_line(
            stdout,
            start_y,
            &browser_mini_help(browser, width, no_colors),
        );
    }

    if browser.filter_state() == FilterState::Editing {
        let rows = if browser.show_full_help() {
            browser_filter_full_help(no_colors)
        } else {
            vec![browser_filter_help(no_colors)]
        };
        for (index, row) in rows.iter().enumerate() {
            write_line(stdout, start_y + index as u16, row)?;
        }
        return Ok(());
    }

    for (index, row) in browser_full_help(no_colors).iter().enumerate() {
        write_line(stdout, start_y + index as u16, row)?;
    }
    Ok(())
}
