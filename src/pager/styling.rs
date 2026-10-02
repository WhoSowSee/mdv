use crate::theme::{Color, InlineTheme, PagerTheme};
use minus::{HighlightColors, HighlightStyles, PromptColor, PromptStyle};

pub(super) fn foreground(style: PromptStyle, color: Option<&Color>) -> PromptStyle {
    color.map_or(style, |color| style.foreground(prompt_color(color)))
}

pub(super) fn highlight_styles(theme: &PagerTheme) -> HighlightStyles {
    fn colors(theme: &InlineTheme<Option<Color>>) -> HighlightColors {
        HighlightColors {
            foreground: theme.text.as_ref().map(prompt_color),
            background: theme.background.as_ref().map(prompt_color),
        }
    }
    HighlightStyles {
        selection: colors(&theme.selection),
        search: colors(&theme.search),
        search_current: colors(&theme.search_current),
    }
}

fn prompt_color(color: &Color) -> PromptColor {
    match color {
        Color::Black => PromptColor::Black,
        Color::DarkRed => PromptColor::DarkRed,
        Color::DarkGreen => PromptColor::DarkGreen,
        Color::DarkYellow => PromptColor::DarkYellow,
        Color::DarkBlue => PromptColor::DarkBlue,
        Color::DarkMagenta => PromptColor::DarkMagenta,
        Color::DarkCyan => PromptColor::DarkCyan,
        Color::Grey => PromptColor::Grey,
        Color::DarkGrey => PromptColor::DarkGrey,
        Color::Red => PromptColor::Red,
        Color::Green => PromptColor::Green,
        Color::Yellow => PromptColor::Yellow,
        Color::Blue => PromptColor::Blue,
        Color::Magenta => PromptColor::Magenta,
        Color::Cyan => PromptColor::Cyan,
        Color::White => PromptColor::White,
        Color::AnsiValue(value) => PromptColor::AnsiValue(*value),
        Color::Rgb { r, g, b } => PromptColor::Rgb {
            r: *r,
            g: *g,
            b: *b,
        },
        Color::Reset => PromptColor::Reset,
    }
}
