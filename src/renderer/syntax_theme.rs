use crate::terminal::{AnsiStyle, ansi256_to_rgb};
use crate::theme::{Color, Theme};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use syntect::highlighting::ScopeSelectors;
use syntect::highlighting::{
    Color as SyntectColor, FontStyle, Style, StyleModifier, Theme as SyntectTheme, ThemeItem,
    ThemeSet,
};

/// Global cache of themes
static DEFAULT_THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

pub(crate) fn default_theme_set() -> &'static ThemeSet {
    &DEFAULT_THEME_SET
}

/// Syntect theme plus a reverse map for palette-aware terminal escaping.
pub(crate) struct CodeHighlightTheme {
    pub syntect: SyntectTheme,
    pub palette_map: HashMap<(u8, u8, u8), Color>,
}

impl CodeHighlightTheme {
    pub(crate) fn syntect_only(theme: SyntectTheme) -> Self {
        Self {
            syntect: theme,
            palette_map: HashMap::new(),
        }
    }

    pub(crate) fn uses_palette(&self) -> bool {
        !self.palette_map.is_empty()
    }
}

pub(crate) fn build_syntect_theme(theme: &Theme) -> CodeHighlightTheme {
    let mut palette_map = HashMap::new();
    let mut syntect_theme = SyntectTheme {
        name: Some(format!("mdv:{}", theme.name)),
        ..SyntectTheme::default()
    };
    syntect_theme.settings.foreground = Some(register_color(&mut palette_map, &theme.text));
    if let Some(background) = theme.background.as_ref() {
        syntect_theme.settings.background = Some(register_color(&mut palette_map, background));
    }
    syntect_theme.settings.caret = Some(register_color(&mut palette_map, &theme.text));
    syntect_theme.settings.selection = Some(register_color(&mut palette_map, &theme.text_light));
    syntect_theme.settings.inactive_selection = syntect_theme.settings.selection;

    let syntax = &theme.syntax;
    let mut scopes: Vec<ThemeItem> = Vec::new();

    // Comments
    push_scope(
        &mut scopes,
        &mut palette_map,
        "comment",
        &syntax.comment,
        Some(FontStyle::ITALIC),
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "punctuation.definition.comment",
        &syntax.comment,
        Some(FontStyle::ITALIC),
    );

    // Keywords and directives
    push_scope(
        &mut scopes,
        &mut palette_map,
        "keyword",
        &syntax.keyword,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "storage",
        &syntax.keyword,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "meta.directive",
        &syntax.keyword,
        Some(FontStyle::BOLD),
    );

    // Operators and punctuation
    push_scope(
        &mut scopes,
        &mut palette_map,
        "keyword.operator",
        &syntax.operator,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "punctuation",
        &syntax.operator,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "meta.brace",
        &syntax.operator,
        None,
    );

    // Strings
    push_scope(
        &mut scopes,
        &mut palette_map,
        "string",
        &syntax.string,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "constant.character.escape",
        &syntax.string,
        None,
    );

    // Numbers and constants
    push_scope(
        &mut scopes,
        &mut palette_map,
        "constant.numeric",
        &syntax.number,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "constant.language",
        &syntax.number,
        None,
    );

    // Functions and methods
    push_scope(
        &mut scopes,
        &mut palette_map,
        "entity.name.function",
        &syntax.function,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "support.function",
        &syntax.function,
        None,
    );

    // Types and classes
    push_scope(
        &mut scopes,
        &mut palette_map,
        "entity.name.type",
        &syntax.type_name,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "support.type",
        &syntax.type_name,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "storage.type",
        &syntax.type_name,
        Some(FontStyle::BOLD),
    );

    // Variables and parameters
    push_scope(
        &mut scopes,
        &mut palette_map,
        "variable",
        &syntax.variable,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "variable.parameter",
        &syntax.variable,
        Some(FontStyle::ITALIC),
    );
    push_scope(
        &mut scopes,
        &mut palette_map,
        "entity.other.attribute-name",
        &syntax.variable,
        None,
    );

    syntect_theme.scopes = scopes;
    CodeHighlightTheme {
        syntect: syntect_theme,
        palette_map,
    }
}

/// Escape highlighted fragments using terminal palette codes when possible.
pub(crate) fn as_palette_terminal_escaped(
    ranges: &[(Style, &str)],
    palette_map: &HashMap<(u8, u8, u8), Color>,
    no_colors: bool,
) -> String {
    if no_colors {
        return ranges.iter().map(|(_, text)| *text).collect();
    }

    let mut result = String::new();
    for (style, text) in ranges {
        let fg = blend_fg_color(style.foreground, style.background);
        let key = (fg.r, fg.g, fg.b);
        let color = palette_map.get(&key).cloned().unwrap_or(Color::Rgb {
            r: fg.r,
            g: fg.g,
            b: fg.b,
        });

        let mut ansi = AnsiStyle::new().fg(color.into());
        if style.font_style.contains(FontStyle::BOLD) {
            ansi = ansi.bold();
        }
        if style.font_style.contains(FontStyle::ITALIC) {
            ansi = ansi.italic();
        }
        if style.font_style.contains(FontStyle::UNDERLINE) {
            ansi = ansi.underline();
        }
        result.push_str(&ansi.apply(text, false));
    }

    result
}

fn push_scope(
    scopes: &mut Vec<ThemeItem>,
    palette_map: &mut HashMap<(u8, u8, u8), Color>,
    selector: &str,
    color: &Color,
    font_style: Option<FontStyle>,
) {
    let syntect_color = register_color(palette_map, color);
    if let Ok(scope) = ScopeSelectors::from_str(selector) {
        scopes.push(ThemeItem {
            scope,
            style: StyleModifier {
                foreground: Some(syntect_color),
                background: None,
                font_style,
            },
        });
    }
}

fn register_color(palette_map: &mut HashMap<(u8, u8, u8), Color>, color: &Color) -> SyntectColor {
    let rgb = color_to_rgb(color);
    match palette_map.get(&rgb) {
        Some(existing) if palette_color_priority(color) < palette_color_priority(existing) => {
            palette_map.insert(rgb, color.clone());
        }
        None => {
            palette_map.insert(rgb, color.clone());
        }
        _ => {}
    }
    let (r, g, b) = rgb;
    SyntectColor { r, g, b, a: 0xFF }
}

fn palette_color_priority(color: &Color) -> u8 {
    match color {
        Color::Reset => 0,
        Color::Black => 1,
        Color::Rgb { r, g, b } if *r < 128 && *g < 128 && *b < 128 => 2,
        Color::DarkRed
        | Color::DarkGreen
        | Color::DarkYellow
        | Color::DarkBlue
        | Color::DarkMagenta
        | Color::DarkCyan
        | Color::DarkGrey => 3,
        Color::AnsiValue(n) if *n < 16 => 4,
        Color::White | Color::AnsiValue(231) | Color::AnsiValue(15) => 255,
        _ => 128,
    }
}

fn blend_fg_color(fg: SyntectColor, bg: SyntectColor) -> SyntectColor {
    if fg.a == 0xff {
        return fg;
    }
    let ratio = fg.a as u32;
    let r = (fg.r as u32 * ratio + bg.r as u32 * (255 - ratio)) / 255;
    let g = (fg.g as u32 * ratio + bg.g as u32 * (255 - ratio)) / 255;
    let b = (fg.b as u32 * ratio + bg.b as u32 * (255 - ratio)) / 255;
    SyntectColor {
        r: r as u8,
        g: g as u8,
        b: b as u8,
        a: 255,
    }
}

fn color_to_rgb(color: &Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::DarkBlue => (0, 0, 128),
        Color::DarkMagenta => (128, 0, 128),
        Color::DarkCyan => (0, 128, 128),
        Color::Grey => (192, 192, 192),
        Color::DarkGrey => (128, 128, 128),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Yellow => (255, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::White => (255, 255, 255),
        Color::AnsiValue(index) => ansi256_to_rgb(*index),
        Color::Rgb { r, g, b } => (*r, *g, *b),
        Color::Reset => (255, 255, 255),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use syntect::easy::HighlightLines;
    use syntect::parsing::SyntaxSet;

    #[test]
    fn build_syntect_theme_registers_palette_map() {
        let bundle = build_syntect_theme(&Theme::default());
        assert!(bundle.uses_palette());
        assert!(bundle.palette_map.contains_key(&(255, 255, 255)));
    }

    #[test]
    fn palette_escape_avoids_truecolor_white_for_default_terminal_theme() {
        let bundle = build_syntect_theme(&Theme::default());
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax = syntax_set.find_syntax_by_extension("sh").unwrap();
        let code =
            "omarchy-theme-install https://github.com/euandeas/omarchy-flexoki-light-theme.git\n";
        let mut highlighter = HighlightLines::new(syntax, &bundle.syntect);
        let ranges = highlighter
            .highlight_line(code, &syntax_set)
            .expect("highlight bash line");

        let escaped = as_palette_terminal_escaped(&ranges, &bundle.palette_map, false);
        assert!(
            !escaped.contains("\x1b[38;2;255;255;255m"),
            "palette escape must not emit truecolor white: {escaped:?}"
        );
        assert!(
            !escaped.contains("\x1b[97m") && !escaped.contains("\x1b[38;5;231m"),
            "palette escape must not emit bright white: {escaped:?}"
        );
        assert!(
            escaped.contains("\x1b[39m"),
            "default terminal syntax should use terminal default foreground: {escaped:?}"
        );
    }
}
