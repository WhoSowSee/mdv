use minus::{PromptColor, PromptContext, PromptError, PromptLine, PromptSpan, PromptStyle};
use std::path::Path;

const BRAND_TEXT: &str = " MDV ";
const HELP_TEXT: &str = " ? Help ";

const ACCENT_BACKGROUND: PromptColor = PromptColor::Rgb {
    r: 50,
    g: 50,
    b: 50,
};
const MAIN_FOREGROUND: PromptColor = PromptColor::Rgb {
    r: 125,
    g: 125,
    b: 125,
};
const MAIN_BACKGROUND: PromptColor = PromptColor::Rgb {
    r: 36,
    g: 36,
    b: 36,
};
const PROGRESS_FOREGROUND: PromptColor = PromptColor::Rgb {
    r: 90,
    g: 90,
    b: 90,
};

pub(super) struct PagerFooter {
    title: String,
    transparent: bool,
}

impl PagerFooter {
    pub(super) fn new(title: Option<&str>, file: Option<&Path>, transparent: bool) -> Self {
        let title = title
            .map(str::to_owned)
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                file.and_then(Path::file_name)
                    .map(|name| name.to_string_lossy().into_owned())
                    .filter(|name| !name.trim().is_empty())
            })
            .unwrap_or_else(|| "stdin".to_string());

        Self { title, transparent }
    }

    pub(super) fn render(&self, context: &PromptContext<'_>) -> Result<PromptLine, PromptError> {
        let content = context.message().unwrap_or(&self.title);
        build_footer(
            content,
            context.scroll_percentage(),
            context.search_position(),
            self.transparent,
        )
    }
}

fn build_footer(
    content: &str,
    percentage: u8,
    search_position: Option<(usize, usize)>,
    transparent: bool,
) -> Result<PromptLine, PromptError> {
    if transparent {
        build_transparent_footer(content, percentage, search_position)
    } else {
        build_opaque_footer(content, percentage, search_position)
    }
}

fn add_progress(
    mut footer: PromptLine,
    percentage: u8,
    search_position: Option<(usize, usize)>,
    style: PromptStyle,
) -> Result<PromptLine, PromptError> {
    if let Some((current, total)) = search_position {
        footer = footer.right(PromptSpan::new(format!(" {current}/{total}"), style)?);
    }
    Ok(footer.right(PromptSpan::new(format!(" {percentage:>3}% "), style)?))
}

fn build_opaque_footer(
    content: &str,
    percentage: u8,
    search_position: Option<(usize, usize)>,
) -> Result<PromptLine, PromptError> {
    let brand_style = PromptStyle::default()
        .foreground(MAIN_FOREGROUND)
        .background(ACCENT_BACKGROUND);
    let main_style = PromptStyle::default()
        .foreground(MAIN_FOREGROUND)
        .background(MAIN_BACKGROUND);
    let progress_style = main_style.foreground(PROGRESS_FOREGROUND);
    let help_style = main_style.background(ACCENT_BACKGROUND);

    let footer = PromptLine::new()
        .left(PromptSpan::new(BRAND_TEXT, brand_style)?)
        .left(PromptSpan::new(format!(" {content}"), main_style)?);
    let footer = add_progress(footer, percentage, search_position, progress_style)?;

    Ok(footer
        .right(PromptSpan::new(HELP_TEXT, help_style)?)
        .fill_style(main_style)
        .truncation_indicator(PromptSpan::new("…", main_style)?))
}

fn build_transparent_footer(
    content: &str,
    percentage: u8,
    search_position: Option<(usize, usize)>,
) -> Result<PromptLine, PromptError> {
    let main_style = PromptStyle::default().foreground(MAIN_FOREGROUND);
    let progress_style = main_style.foreground(PROGRESS_FOREGROUND);

    let footer = PromptLine::new()
        .left(PromptSpan::new(BRAND_TEXT, main_style)?)
        .left(PromptSpan::new("|", main_style)?)
        .left(PromptSpan::new(format!(" {content}"), main_style)?);
    let footer = add_progress(footer, percentage, search_position, progress_style)?;

    Ok(footer
        .right(PromptSpan::new("| ? Help ", main_style)?)
        .fill_style(main_style)
        .truncation_indicator(PromptSpan::new("…", main_style)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn footer_layout_contains_all_sections() {
        let plain = build_footer("AGENTS.md", 22, None, false)
            .unwrap()
            .render_plain(80);

        assert_eq!(plain.width(), 80);
        assert!(plain.starts_with(" MDV  AGENTS.md"));
        assert!(plain.ends_with("  22%  ? Help "));
    }

    #[test]
    fn search_position_appears_before_document_progress() {
        let footer = build_footer("AGENTS.md", 22, Some((2, 5)), false).unwrap();
        let plain = footer.render_plain(80);
        let rendered = footer.render(80);
        let transparent = build_footer("AGENTS.md", 22, Some((2, 5)), true)
            .unwrap()
            .render_plain(80);

        assert!(plain.ends_with(" 2/5  22%  ? Help "));
        assert!(transparent.ends_with(" 2/5  22% | ? Help "));
        assert!(
            rendered.matches("38;2;90;90;90").count() >= 2,
            "{}",
            rendered.escape_debug()
        );
    }

    #[test]
    fn explicit_title_overrides_the_file_name() {
        let footer = PagerFooter::new(Some("Help"), Some(Path::new("README.md")), false);

        assert_eq!(footer.title, "Help");
    }

    #[test]
    fn footer_uses_expected_colors() {
        let rendered = build_footer("AGENTS.md", 22, None, false)
            .unwrap()
            .render(80);

        assert!(rendered.contains("38;2;125;125;125"));
        assert!(rendered.contains("38;2;90;90;90"));
        assert!(rendered.contains("48;2;36;36;36"));
        assert!(rendered.matches("48;2;50;50;50").count() >= 2);
        assert!(rendered.ends_with("\x1b[0m"));
    }

    #[test]
    fn transparent_footer_uses_separators_without_background() {
        let footer = build_footer("AGENTS.md", 22, None, true).unwrap();
        let plain = footer.render_plain(80);

        assert!(plain.starts_with(" MDV | AGENTS.md"));
        assert!(plain.ends_with("  22% | ? Help "));
        assert!(!plain.contains("|  22%"));
        assert_eq!(plain.matches('|').count(), 2);
        assert!(!footer.render(80).contains("\x1b[48;"));
    }

    #[test]
    fn long_unicode_file_name_is_truncated_to_terminal_width() {
        let plain = build_footer("очень-длинный-файл-📚.md", 7, None, false)
            .unwrap()
            .render_plain(32);

        assert_eq!(plain.width(), 32);
        assert!(plain.contains('…'));
    }

    #[test]
    fn narrow_footer_never_exceeds_terminal_width() {
        let footer = build_footer("README.md", 100, None, false).unwrap();
        for columns in 0..20 {
            assert_eq!(footer.render_plain(columns).width(), columns);
        }
    }
}
