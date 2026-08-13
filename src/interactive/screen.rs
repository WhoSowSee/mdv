use super::app::App;
use super::browser::{BrowserSection, BrowserState, FilterState};
use crate::terminal::AnsiStyle;
use crate::utils::display_width;
use anyhow::{Context, Result};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::style::{Color, Print, ResetColor};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{execute, queue};
use std::io::{Stdout, Write, stdout};
use std::time::{Duration, SystemTime};
use unicode_width::UnicodeWidthChar;

const ELLIPSIS: &str = "…";
const BROWSER_ACCENT: Color = Color::Rgb {
    r: 126,
    g: 156,
    b: 216,
};
const BROWSER_SELECTED_DATE: Color = Color::Rgb {
    r: 92,
    g: 107,
    b: 135,
};
const BROWSER_FILTER_INPUT: Color = Color::Rgb {
    r: 184,
    g: 197,
    b: 223,
};
const BROWSER_LOGO_FOREGROUND: Color = Color::Rgb {
    r: 31,
    g: 35,
    b: 53,
};
const BROWSER_HELP_KEY: Color = Color::Rgb {
    r: 97,
    g: 97,
    b: 97,
};
const BROWSER_HELP_LABEL: Color = Color::Rgb {
    r: 73,
    g: 73,
    b: 73,
};
const BROWSER_HELP_SEPARATOR: Color = Color::Rgb {
    r: 60,
    g: 60,
    b: 60,
};
const BROWSER_MINI_HELP: &[(&str, &str)] = &[
    ("h/l ←/→", "page"),
    ("/", "find"),
    ("r", "refresh"),
    ("e", "edit"),
    ("q", "quit"),
    ("?", "more"),
];
const BROWSER_FILTERED_MINI_HELP: &[(&str, &str)] = &[
    ("tab", "section"),
    ("/", "edit search"),
    ("esc", "clear filter"),
    ("r", "refresh"),
    ("e", "edit"),
    ("q", "quit"),
    ("?", "more"),
];
const BROWSER_FULL_HELP_ROWS: [[Option<(&str, &str)>; 4]; 4] = [
    [
        Some(("enter", "open")),
        Some(("/", "find")),
        Some(("e", "edit")),
        Some(("r", "refresh")),
    ],
    [
        Some(("j/k ↑/↓", "choose")),
        Some(("esc", "clear")),
        Some(("!", "errors")),
        Some(("q", "quit")),
    ],
    [
        Some(("h/l ←/→", "page")),
        Some(("tab", "section")),
        Some(("?", "close help")),
        None,
    ],
    [
        Some(("g/home", "first")),
        Some(("G/end", "last")),
        None,
        None,
    ],
];

pub(super) struct TerminalSession {
    stdout: Stdout,
    active: bool,
}

impl TerminalSession {
    pub(super) fn enter() -> Result<Self> {
        let mut session = Self {
            stdout: stdout(),
            active: false,
        };
        session.resume()?;
        Ok(session)
    }

    pub(super) fn draw(&mut self, app: &App) -> Result<()> {
        queue!(self.stdout, Hide, MoveTo(0, 0), Clear(ClearType::All))?;
        draw_browser(&mut self.stdout, app)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub(super) fn suspend(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let restore_result = execute!(
            self.stdout,
            DisableBracketedPaste,
            ResetColor,
            Show,
            LeaveAlternateScreen
        );
        let raw_result = disable_raw_mode();
        self.active = false;
        restore_result?;
        raw_result?;
        Ok(())
    }

    pub(super) fn pause_for_pager(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let display_result = write_pager_pause(&mut self.stdout);
        let raw_result = disable_raw_mode();
        display_result?;
        raw_result?;
        Ok(())
    }

    pub(super) fn resume_after_pager(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        enable_raw_mode()?;
        if let Err(error) = write_pager_resume(&mut self.stdout) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        Ok(())
    }

    pub(super) fn resume(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }
        enable_raw_mode()?;
        if let Err(error) = execute!(
            self.stdout,
            EnterAlternateScreen,
            EnableBracketedPaste,
            Hide
        ) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        self.active = true;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.suspend();
    }
}

fn write_pager_pause(output: &mut impl Write) -> std::io::Result<()> {
    execute!(output, DisableBracketedPaste, ResetColor, Show)
}

fn write_pager_resume(output: &mut impl Write) -> std::io::Result<()> {
    execute!(output, EnableBracketedPaste, Hide)
}

fn draw_browser(stdout: &mut Stdout, app: &App) -> Result<()> {
    let browser = &app.browser;
    let no_colors = app.config.no_colors;
    if browser.show_error() {
        return draw_browser_error(stdout, browser, app.width, app.height, no_colors);
    }

    if browser.filter_state() == FilterState::Editing {
        let filter_text = truncate_plain(
            &format!("Find: {}", browser.query()),
            app.width.saturating_sub(3) as usize,
        );
        let filter = browser_filter_prompt_text(&filter_text, no_colors);
        write_line(stdout, 1, &filter)?;
    } else {
        write_line(stdout, 1, &browser_logo_line(no_colors))?;
    }

    let header = browser_header(browser, no_colors);
    write_line(stdout, 3, &format!("   {header}"))?;

    let visible = browser.visible_indices();
    let page_start = browser.page() * browser.per_page();
    let page_end = (page_start + browser.per_page()).min(visible.len());
    let available_width = app.width.saturating_sub(8) as usize;
    if page_start == page_end {
        let message = if browser.is_loaded() {
            "No files found."
        } else {
            "Looking for local files..."
        };
        write_line(
            stdout,
            5,
            &format!(
                "   {}",
                styled(message, Some(rgb(98, 98, 98)), None, false, no_colors)
            ),
        )?;
    } else {
        for (row, document_index) in visible[page_start..page_end].iter().enumerate() {
            let document = &browser.documents()[*document_index];
            let y = 5 + (row * 3) as u16;
            let selected = browser_item_selected(browser, row);
            let title_color = if selected {
                BROWSER_ACCENT
            } else {
                rgb(221, 221, 221)
            };
            let date_color = if selected {
                BROWSER_SELECTED_DATE
            } else {
                rgb(98, 98, 98)
            };
            let title = truncate_plain(&sanitize_display(&document.relative_path), available_width);
            let title = if browser.filter_state() == FilterState::Editing
                || browser.section() == BrowserSection::Filter
            {
                filtered_title(document, &title, browser.query(), title_color, no_colors)
            } else {
                styled(&title, Some(title_color), None, false, no_colors)
            };
            let date_text = relative_time(document.modified)?;
            let date = styled(&date_text, Some(date_color), None, false, no_colors);
            let prefix = item_prefix(selected, no_colors);
            write_line(stdout, y, &format!("{prefix}{title}"))?;
            write_line(stdout, y + 1, &format!("{prefix}{date}"))?;
        }
    }

    let (pagination_y, help_y) = browser_footer_rows(app.height, browser);
    if browser.page_count() > 1 {
        write_line(
            stdout,
            pagination_y,
            &format!("   {}", pagination(browser, app.width as usize, no_colors)),
        )?;
    }
    draw_browser_help(stdout, browser, help_y, app.width as usize, no_colors)?;
    if browser.filter_state() == FilterState::Editing {
        let filter_text = truncate_plain(
            &format!("Find: {}", browser.query()),
            app.width.saturating_sub(3) as usize,
        );
        queue!(
            stdout,
            Show,
            MoveTo(browser_filter_cursor_x(&filter_text, app.width), 1)
        )?;
    }
    Ok(())
}

fn draw_browser_error(
    stdout: &mut Stdout,
    browser: &BrowserState,
    width: u16,
    height: u16,
    no_colors: bool,
) -> Result<()> {
    let title = styled(
        " ERROR ",
        Some(rgb(255, 253, 245)),
        Some(rgb(237, 86, 122)),
        false,
        no_colors,
    );
    write_line(stdout, 1, &format!("   {title}"))?;
    for (index, error) in browser
        .errors()
        .iter()
        .take(height.saturating_sub(6) as usize)
        .enumerate()
    {
        let error = truncate_plain(&sanitize_display(error), width.saturating_sub(6) as usize);
        write_line(stdout, 3 + index as u16, &format!("   {error}"))?;
    }
    let prompt = styled(
        "press any key to return",
        Some(rgb(92, 92, 92)),
        None,
        false,
        no_colors,
    );
    write_line(stdout, height.saturating_sub(2), &format!("   {prompt}"))?;
    Ok(())
}

fn browser_header(browser: &BrowserState, no_colors: bool) -> String {
    if browser.filter_state() == FilterState::Editing {
        return styled(
            &format!("{} local", browser.documents().len()),
            Some(rgb(98, 98, 98)),
            None,
            false,
            no_colors,
        );
    }

    let documents = format!("{} documents", browser.documents().len());
    if browser.filter_state() != FilterState::Applied {
        return styled(&documents, Some(rgb(98, 98, 98)), None, false, no_colors);
    }
    let filtered = format!("{} “{}”", browser.filtered_count(), browser.query());
    let documents_color = if browser.section() == BrowserSection::Documents {
        rgb(151, 151, 151)
    } else {
        rgb(98, 98, 98)
    };
    let filtered_color = if browser.section() == BrowserSection::Filter {
        rgb(151, 151, 151)
    } else {
        rgb(98, 98, 98)
    };
    format!(
        "{} {} {}",
        styled(&documents, Some(documents_color), None, false, no_colors),
        styled("│", Some(rgb(60, 60, 60)), None, false, no_colors),
        styled(&filtered, Some(filtered_color), None, false, no_colors)
    )
}

fn pagination(browser: &BrowserState, width: usize, no_colors: bool) -> String {
    let pages = browser.page_count();
    let dots_width = pages;
    if dots_width + 6 > width {
        return styled(
            &format!("{}/{}", browser.page() + 1, pages),
            Some(rgb(92, 92, 92)),
            None,
            false,
            no_colors,
        );
    }
    (0..pages)
        .map(|page| {
            let color = if page == browser.page() {
                rgb(132, 122, 133)
            } else {
                rgb(60, 60, 60)
            };
            styled("•", Some(color), None, false, no_colors)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn browser_filter_cursor_x(filter_text: &str, width: u16) -> u16 {
    display_width(filter_text)
        .saturating_add(3)
        .min(width.saturating_sub(1) as usize) as u16
}

fn browser_item_selected(browser: &BrowserState, row: usize) -> bool {
    browser.filter_state() != FilterState::Editing && row == browser.selected_index_on_page()
}

fn browser_filter_prompt_text(text: &str, no_colors: bool) -> String {
    let (label, query) = text
        .split_once(' ')
        .map_or((text, None), |(label, query)| (label, Some(query)));
    let mut prompt = format!(
        "   {}",
        styled(label, Some(BROWSER_ACCENT), None, false, no_colors)
    );
    if let Some(query) = query {
        prompt.push(' ');
        prompt.push_str(&styled(
            query,
            Some(BROWSER_FILTER_INPUT),
            None,
            false,
            no_colors,
        ));
    }
    prompt
}

fn filtered_title(
    document: &super::discovery::DocumentEntry,
    title: &str,
    query: &str,
    color: Color,
    no_colors: bool,
) -> String {
    if query.is_empty() || no_colors {
        return styled(title, Some(color), None, false, no_colors);
    }

    let indices = document.match_indices(query);
    let mut output = String::new();
    for (index, character) in title.chars().enumerate() {
        let mut style = AnsiStyle::new().fg(color);
        if indices.binary_search(&index).is_ok() {
            style = style.underline();
        }
        output.push_str(&style.apply(&character.to_string(), false));
    }
    output
}

fn browser_logo_line(no_colors: bool) -> String {
    let logo = styled(
        " MDV ",
        Some(BROWSER_LOGO_FOREGROUND),
        Some(BROWSER_ACCENT),
        true,
        no_colors,
    );
    format!("   {logo}")
}

fn browser_mini_help(browser: &BrowserState, width: usize, no_colors: bool) -> String {
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

fn browser_filter_help(no_colors: bool) -> String {
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

fn browser_filter_full_help(no_colors: bool) -> Vec<String> {
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

fn browser_full_help(no_colors: bool) -> Vec<String> {
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

fn item_prefix(selected: bool, no_colors: bool) -> String {
    let gutter = if selected {
        styled("│", Some(BROWSER_ACCENT), None, false, no_colors)
    } else {
        " ".to_string()
    };
    format!(" {gutter} ")
}

fn browser_help_rows(browser: &BrowserState) -> u16 {
    if browser.filter_state() == FilterState::Editing && browser.show_full_help() {
        3
    } else if browser.show_full_help() {
        4
    } else {
        1
    }
}

fn browser_footer_rows(height: u16, browser: &BrowserState) -> (u16, u16) {
    let help_rows = browser_help_rows(browser);
    let help_y = height.saturating_sub(help_rows + 1);
    (help_y.saturating_sub(2), help_y)
}

fn draw_browser_help(
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

fn write_line(stdout: &mut Stdout, y: u16, text: &str) -> Result<()> {
    queue!(
        stdout,
        MoveTo(0, y),
        Print(text),
        Clear(ClearType::UntilNewLine)
    )?;
    Ok(())
}

fn styled(
    text: &str,
    foreground: Option<Color>,
    background: Option<Color>,
    bold: bool,
    no_colors: bool,
) -> String {
    let mut style = AnsiStyle::new();
    if let Some(foreground) = foreground {
        style = style.fg(foreground);
    }
    if let Some(background) = background {
        style = style.bg(background);
    }
    if bold {
        style = style.bold();
    }
    style.apply(text, no_colors)
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

fn sanitize_display(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn truncate_plain(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if display_width(text) <= width {
        return text.to_string();
    }
    let content_width = width.saturating_sub(display_width(ELLIPSIS));
    let mut output = String::new();
    let mut used = 0;
    for character in text.chars() {
        let character_width = character.width().unwrap_or(0);
        if used + character_width > content_width {
            break;
        }
        output.push(character);
        used += character_width;
    }
    output.push_str(ELLIPSIS);
    output
}

fn relative_time(modified: SystemTime) -> Result<String> {
    document_time(modified, SystemTime::now())
}

fn document_time(modified: SystemTime, now: SystemTime) -> Result<String> {
    let age = now.duration_since(modified).unwrap_or(Duration::ZERO);
    Ok(match age {
        age if age < Duration::from_secs(60) => "just now".to_string(),
        age if age < Duration::from_secs(120) => "1 minute ago".to_string(),
        age if age < Duration::from_secs(3_600) => {
            format!("{} minutes ago", age.as_secs() / 60)
        }
        age if age < Duration::from_secs(7_200) => "1 hour ago".to_string(),
        age if age < Duration::from_secs(86_400) => {
            format!("{} hours ago", age.as_secs() / 3_600)
        }
        age if age < Duration::from_secs(172_800) => "1 day ago".to_string(),
        age if age < Duration::from_secs(604_800) => {
            format!("{} days ago", age.as_secs() / 86_400)
        }
        _ => return format_local_timestamp(modified),
    })
}

fn format_local_timestamp(modified: SystemTime) -> Result<String> {
    let timestamp = jiff::Zoned::try_from(modified)
        .context("document modification time is outside the supported range")?;
    Ok(timestamp.strftime("%d %b %Y %H:%M %Z").to_string())
}

#[cfg(test)]
mod tests {
    use super::super::discovery::DocumentEntry;
    use super::*;

    #[test]
    fn pagination_dots_are_adjacent() {
        let documents = (0..3)
            .map(|index| DocumentEntry::for_test(&format!("document-{index}.md")))
            .collect();
        let browser = BrowserState::for_test(documents, 14);

        assert_eq!(pagination(&browser, 80, true), "•••");
    }

    #[test]
    fn recent_documents_keep_relative_time() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);
        let modified = now - Duration::from_secs(6 * 86_400);

        assert_eq!(document_time(modified, now).unwrap(), "6 days ago");
    }

    #[test]
    fn week_old_documents_use_an_absolute_local_timestamp() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);
        let modified = now - Duration::from_secs(7 * 86_400);

        let timestamp = document_time(modified, now).unwrap();
        assert_eq!(timestamp, format_local_timestamp(modified).unwrap());
        assert!(timestamp.contains(':'));
        assert!(timestamp.split_whitespace().count() >= 5);
    }

    #[test]
    fn document_titles_align_with_the_header() {
        assert_eq!(item_prefix(false, true), "   ");
        assert_eq!(item_prefix(true, true), " │ ");
    }

    #[test]
    fn logo_block_aligns_with_the_header_without_a_spinner() {
        assert_eq!(browser_logo_line(true), "    MDV ");
    }

    #[test]
    fn mini_help_matches_the_navigation_status() {
        assert_eq!(
            browser_mini_help(&BrowserState::for_test(Vec::new(), 24), 120, true),
            "   h/l ←/→ page • / find • r refresh • e edit • q quit • ? more"
        );
    }

    #[test]
    fn filter_prompt_aligns_with_browser_content_and_uses_the_blue_palette() {
        assert_eq!(browser_filter_cursor_x("Find: doc", 80), 12);
        assert_eq!(browser_filter_cursor_x("Find: document", 10), 9);
        assert_eq!(browser_filter_prompt_text("Find:", true), "   Find:");
        assert_eq!(
            browser_filter_prompt_text("Find: doc", true),
            "   Find: doc"
        );

        let prompt = browser_filter_prompt_text("Find: doc", false);
        assert!(prompt.starts_with("   "));
        assert!(prompt.contains(&styled("Find:", Some(BROWSER_ACCENT), None, false, false)));
        assert!(prompt.contains(&styled(
            "doc",
            Some(BROWSER_FILTER_INPUT),
            None,
            false,
            false
        )));
    }

    #[test]
    fn filter_help_aligns_with_browser_content_and_has_compact_separators() {
        assert_eq!(
            browser_filter_help(true),
            "   enter confirm • esc cancel • ctrl+j/ctrl+k ↑/↓ choose"
        );
    }

    #[test]
    fn expanded_filter_help_uses_three_aligned_rows() {
        assert_eq!(
            browser_filter_full_help(true),
            [
                "   enter              confirm",
                "   esc                cancel",
                "   ctrl+j/ctrl+k ↑/↓  choose",
            ]
        );
    }

    #[test]
    fn filter_help_controls_the_reserved_footer_height() {
        let documents = vec![DocumentEntry::for_test("README.md")];
        let mut browser = BrowserState::for_test(documents, 24);
        browser.toggle_help();
        browser.begin_filter();

        assert_eq!(browser_footer_rows(24, &browser), (18, 20));
        assert_eq!(browser_help_rows(&browser), 3);
    }

    #[test]
    fn applied_filter_omits_page_navigation_from_mini_help() {
        let documents = (0..12)
            .map(|index| DocumentEntry::for_test(&format!("docs/{index}.md")))
            .collect();
        let mut browser = BrowserState::for_test(documents, 24);
        browser.begin_filter();
        browser.set_filter("docs");
        browser.confirm_filter();

        assert!(browser.page_count() > 1);
        assert_eq!(
            browser_mini_help(&browser, 120, true),
            "   tab section • / edit search • esc clear filter • r refresh • e edit • q quit • ? more"
        );
    }

    #[test]
    fn mini_help_truncates_at_a_segment_boundary() {
        let documents = [
            DocumentEntry::for_test("docs/one.md"),
            DocumentEntry::for_test("docs/two.md"),
        ];
        let mut browser = BrowserState::for_test(documents.into(), 24);
        browser.begin_filter();
        browser.set_filter("docs");
        browser.confirm_filter();

        assert_eq!(
            browser_mini_help(&browser, 64, true),
            "   tab section • / edit search • esc clear filter • …"
        );
    }

    #[test]
    fn filtering_underlines_only_the_fuzzy_match_characters() {
        let document = DocumentEntry::for_test("test_codeblock_indent.md");
        let title = filtered_title(
            &document,
            &document.relative_path,
            "doc",
            rgb(221, 221, 221),
            false,
        );

        assert_eq!(title.matches("\x1b[4m").count(), 3);
        assert_eq!(crate::utils::strip_ansi(&title), "test_codeblock_indent.md");
    }

    #[test]
    fn filtering_without_colors_keeps_the_title_plain() {
        assert_eq!(
            filtered_title(
                &DocumentEntry::for_test("docs/résumé.md"),
                "docs/résumé.md",
                "RESUME",
                rgb(221, 221, 221),
                true,
            ),
            "docs/résumé.md"
        );
    }

    #[test]
    fn filter_editing_never_marks_a_result_as_selected() {
        let documents = [
            DocumentEntry::for_test("DOC.md"),
            DocumentEntry::for_test("docs/sample.md"),
        ];
        let mut browser = BrowserState::for_test(documents.into(), 20);
        browser.begin_filter();
        browser.set_filter("doc");

        assert!(!browser_item_selected(&browser, 0));

        browser.set_filter("sample");

        assert!(!browser_item_selected(&browser, 0));
    }

    #[test]
    fn full_help_uses_aligned_columns_from_browser_column_three() {
        let rows = browser_full_help(true);
        let visual_column = |row: &str, text: &str| {
            let byte_index = row.find(text).unwrap();
            display_width(&row[..byte_index])
        };

        assert_eq!(rows.len(), 4);
        assert_eq!(
            [
                visual_column(&rows[0], "enter"),
                visual_column(&rows[1], "j/k"),
                visual_column(&rows[2], "h/l"),
                visual_column(&rows[3], "g/home"),
            ],
            [3; 4]
        );
        assert_eq!(
            [
                visual_column(&rows[0], "/"),
                visual_column(&rows[1], "esc"),
                visual_column(&rows[2], "tab"),
                visual_column(&rows[3], "G/end"),
            ],
            [22; 4]
        );
        assert_eq!(
            [
                visual_column(&rows[0], "e  edit"),
                visual_column(&rows[1], "!  errors"),
                visual_column(&rows[2], "?  close help"),
            ],
            [40; 3]
        );
        assert_eq!(
            [
                visual_column(&rows[0], "r  refresh"),
                visual_column(&rows[1], "q  quit"),
            ],
            [57; 2]
        );
    }

    #[test]
    fn mini_help_reserves_a_bottom_row() {
        let mut browser = BrowserState::for_test(vec![DocumentEntry::for_test("README.md")], 24);
        assert_eq!(browser_footer_rows(24, &browser), (20, 22));
        browser.toggle_help();
        assert_eq!(browser_footer_rows(24, &browser), (17, 19));
    }

    #[test]
    fn pager_handoff_keeps_the_current_screen_buffer() {
        let mut output = Vec::new();

        write_pager_pause(&mut output).unwrap();
        write_pager_resume(&mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains("\x1b[?1049h"));
        assert!(!output.contains("\x1b[?1049l"));
    }
}
