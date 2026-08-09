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
    file_name: String,
}

impl PagerFooter {
    pub(super) fn new(file: Option<&Path>) -> Self {
        let file_name = file
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "stdin".to_string());

        Self { file_name }
    }

    pub(super) fn render(&self, context: &PromptContext<'_>) -> Result<PromptLine, PromptError> {
        let content = context.message().unwrap_or(&self.file_name);
        build_footer(content, context.scroll_percentage())
    }
}

fn build_footer(content: &str, percentage: u8) -> Result<PromptLine, PromptError> {
    let brand_style = PromptStyle::default()
        .foreground(MAIN_FOREGROUND)
        .background(ACCENT_BACKGROUND);
    let main_style = PromptStyle::default()
        .foreground(MAIN_FOREGROUND)
        .background(MAIN_BACKGROUND);
    let progress_style = main_style.foreground(PROGRESS_FOREGROUND);
    let help_style = main_style.background(ACCENT_BACKGROUND);

    Ok(PromptLine::new()
        .left(PromptSpan::new(BRAND_TEXT, brand_style)?)
        .left(PromptSpan::new(format!(" {content}"), main_style)?)
        .right(PromptSpan::new(
            format!(" {percentage:>3}% "),
            progress_style,
        )?)
        .right(PromptSpan::new(HELP_TEXT, help_style)?)
        .fill_style(main_style)
        .truncation_indicator(PromptSpan::new("…", main_style)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn footer_layout_matches_glow_structure() {
        let plain = build_footer("AGENTS.md", 22).unwrap().render_plain(80);

        assert_eq!(plain.width(), 80);
        assert!(plain.starts_with(" MDV  AGENTS.md"));
        assert!(plain.ends_with("  22%  ? Help "));
    }

    #[test]
    fn footer_uses_glow_colors() {
        let rendered = build_footer("AGENTS.md", 22).unwrap().render(80);

        assert!(rendered.contains("38;2;125;125;125"));
        assert!(rendered.contains("38;2;90;90;90"));
        assert!(rendered.contains("48;2;36;36;36"));
        assert!(rendered.matches("48;2;50;50;50").count() >= 2);
        assert!(rendered.ends_with("\x1b[0m"));
    }

    #[test]
    fn long_unicode_file_name_is_truncated_to_terminal_width() {
        let plain = build_footer("очень-длинный-файл-📚.md", 7)
            .unwrap()
            .render_plain(32);

        assert_eq!(plain.width(), 32);
        assert!(plain.contains('…'));
    }

    #[test]
    fn narrow_footer_never_exceeds_terminal_width() {
        let footer = build_footer("README.md", 100).unwrap();
        for columns in 0..20 {
            assert_eq!(footer.render_plain(columns).width(), columns);
        }
    }
}
