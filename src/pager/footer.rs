use super::styling::foreground;
use crate::theme::{Color, PagerTheme};
use minus::{PromptColor, PromptContext, PromptError, PromptLine, PromptSpan, PromptStyle};
use std::path::Path;

const BRAND_TEXT: &str = " MDV ";
const HELP_TEXT: &str = " ? Help ";
const ACCENT_BACKGROUND: PromptColor = PromptColor::Rgb {
    r: 50,
    g: 50,
    b: 50,
};
const MAIN_BACKGROUND: PromptColor = PromptColor::Rgb {
    r: 36,
    g: 36,
    b: 36,
};

pub(super) struct PagerFooter {
    title: String,
    theme: PagerTheme,
}

#[derive(Clone, Copy)]
struct FooterProgress {
    percentage: u8,
    search_position: Option<(usize, usize)>,
    line_navigation_position: Option<usize>,
}

impl FooterProgress {
    fn from_context(context: &PromptContext<'_>) -> Self {
        Self {
            percentage: context.scroll_percentage(),
            search_position: context.search_position(),
            line_navigation_position: context.line_navigation_position(),
        }
    }
}

impl PagerFooter {
    pub(super) fn new(title: Option<&str>, file: Option<&Path>, theme: &PagerTheme) -> Self {
        let title = title
            .map(str::to_owned)
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                file.and_then(Path::file_name)
                    .map(|name| name.to_string_lossy().into_owned())
                    .filter(|name| !name.trim().is_empty())
            })
            .unwrap_or_else(|| "stdin".to_string());
        Self {
            title,
            theme: theme.clone(),
        }
    }

    pub(super) fn render(&self, context: &PromptContext<'_>) -> Result<PromptLine, PromptError> {
        let progress = FooterProgress::from_context(context);
        if context.toc_hint_visible() && context.message().is_none() {
            return build_toc_footer(&self.title, progress, &self.theme, context.columns());
        }
        build_footer(
            context.message().unwrap_or(&self.title),
            progress,
            &self.theme,
        )
    }
}

fn style(theme: &PagerTheme, color: Option<&Color>, background: PromptColor) -> PromptStyle {
    let style = foreground(PromptStyle::default(), color);
    if theme.transparent {
        style
    } else {
        style.background(background)
    }
}

fn add_progress(
    mut footer: PromptLine,
    progress: FooterProgress,
    theme: &PagerTheme,
) -> Result<PromptLine, PromptError> {
    let progress_style = style(theme, theme.progress.as_ref(), MAIN_BACKGROUND);
    if let Some((current, total)) = progress.search_position {
        footer = footer.right(PromptSpan::new(
            format!(" {current}/{total}"),
            style(theme, theme.matches.as_ref(), MAIN_BACKGROUND),
        )?);
    }
    if let Some(source_line) = progress.line_navigation_position {
        footer = footer.right(PromptSpan::new(format!(" :{source_line}"), progress_style)?);
    }
    Ok(footer.right(PromptSpan::new(
        format!(" {:>3}% ", progress.percentage),
        progress_style,
    )?))
}

fn build_footer(
    content: &str,
    progress: FooterProgress,
    theme: &PagerTheme,
) -> Result<PromptLine, PromptError> {
    let main_style = style(theme, theme.file_name.as_ref(), MAIN_BACKGROUND);
    let mut footer = PromptLine::new().left(PromptSpan::new(
        BRAND_TEXT,
        style(theme, theme.title.as_ref(), ACCENT_BACKGROUND),
    )?);
    if theme.transparent {
        footer = footer.left(PromptSpan::new("|", main_style)?);
    }
    let footer = footer.left(PromptSpan::new(format!(" {content}"), main_style)?);
    let help = if theme.transparent {
        "| ? Help "
    } else {
        HELP_TEXT
    };
    Ok(add_progress(footer, progress, theme)?
        .right(PromptSpan::new(
            help,
            style(theme, theme.help.as_ref(), ACCENT_BACKGROUND),
        )?)
        .fill_style(main_style)
        .truncation_indicator(PromptSpan::new("…", main_style)?))
}

fn build_toc_footer(
    title: &str,
    progress: FooterProgress,
    theme: &PagerTheme,
    columns: usize,
) -> Result<PromptLine, PromptError> {
    use unicode_width::UnicodeWidthStr;
    let main_style = style(theme, theme.file_name.as_ref(), MAIN_BACKGROUND);
    let title_style = style(theme, theme.title.as_ref(), MAIN_BACKGROUND);
    let hint = if columns >= 72 {
        "Alt+↑↓ move · ? keys"
    } else {
        "? keys"
    };
    let footer = add_progress(PromptLine::new(), progress, theme)?.right(PromptSpan::new(
        "| ? Help ",
        style(theme, theme.help.as_ref(), MAIN_BACKGROUND),
    )?);
    let right = footer.render_plain(columns);
    let available = columns.saturating_sub(right.trim_start().width());
    let hint_start = ((columns.saturating_sub(hint.width())) / 2)
        .min(available.saturating_sub(hint.width() + 1));
    let label = PromptLine::new()
        .left(PromptSpan::new(format!(" MDV | {title}"), main_style)?)
        .truncation_indicator(PromptSpan::new("…", main_style)?)
        .render_plain(hint_start.saturating_sub(1));
    let brand: String = label.chars().take(BRAND_TEXT.len()).collect();
    let rest: String = label.chars().skip(BRAND_TEXT.len()).collect();
    Ok(footer
        .left(PromptSpan::new(brand, title_style)?)
        .left(PromptSpan::new(format!("{rest} {hint}"), main_style)?)
        .fill_style(main_style)
        .truncation_indicator(PromptSpan::new("…", main_style)?))
}

#[cfg(test)]
mod tests;
