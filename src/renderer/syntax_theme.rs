use crate::terminal::ansi256_to_rgb;
use crate::theme::{Color, Theme};
use once_cell::sync::Lazy;
use std::str::FromStr;
use syntect::highlighting::ScopeSelectors;
use syntect::highlighting::{
    Color as SyntectColor, FontStyle, StyleModifier, Theme as SyntectTheme, ThemeItem, ThemeSet,
};

/// Global cache of themes
static DEFAULT_THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub(crate) fn default_theme_set() -> &'static ThemeSet {
    &DEFAULT_THEME_SET
}

pub(crate) fn build_syntect_theme(theme: &Theme) -> SyntectTheme {
    let mut syntect_theme = SyntectTheme::default();
    syntect_theme.name = Some(format!("mdv:{}", theme.name));
    syntect_theme.settings.foreground = Some(to_syntect_color(&theme.text));
    if let Some(background) = theme
        .background
        .as_ref()
        .map(|color| to_syntect_color(color))
    {
        syntect_theme.settings.background = Some(background);
    }
    syntect_theme.settings.caret = Some(to_syntect_color(&theme.text));
    syntect_theme.settings.selection = Some(to_syntect_color(&theme.text_light));
    syntect_theme.settings.inactive_selection = syntect_theme.settings.selection;

    let syntax = &theme.syntax;
    let mut scopes: Vec<ThemeItem> = Vec::new();

    // Comments
    push_scope(
        &mut scopes,
        "comment",
        to_syntect_color(&syntax.comment),
        Some(FontStyle::ITALIC),
    );
    push_scope(
        &mut scopes,
        "punctuation.definition.comment",
        to_syntect_color(&syntax.comment),
        Some(FontStyle::ITALIC),
    );

    // Keywords and directives
    push_scope(
        &mut scopes,
        "keyword",
        to_syntect_color(&syntax.keyword),
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        "storage",
        to_syntect_color(&syntax.keyword),
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        "meta.directive",
        to_syntect_color(&syntax.keyword),
        Some(FontStyle::BOLD),
    );

    // Operators and punctuation
    push_scope(
        &mut scopes,
        "keyword.operator",
        to_syntect_color(&syntax.operator),
        None,
    );
    push_scope(
        &mut scopes,
        "punctuation",
        to_syntect_color(&syntax.operator),
        None,
    );
    push_scope(
        &mut scopes,
        "meta.brace",
        to_syntect_color(&syntax.operator),
        None,
    );

    // Strings
    push_scope(
        &mut scopes,
        "string",
        to_syntect_color(&syntax.string),
        None,
    );
    push_scope(
        &mut scopes,
        "constant.character.escape",
        to_syntect_color(&syntax.string),
        None,
    );

    // Numbers and constants
    push_scope(
        &mut scopes,
        "constant.numeric",
        to_syntect_color(&syntax.number),
        None,
    );
    push_scope(
        &mut scopes,
        "constant.language",
        to_syntect_color(&syntax.number),
        None,
    );

    // Functions and methods
    push_scope(
        &mut scopes,
        "entity.name.function",
        to_syntect_color(&syntax.function),
        None,
    );
    push_scope(
        &mut scopes,
        "support.function",
        to_syntect_color(&syntax.function),
        None,
    );

    // Types and classes
    push_scope(
        &mut scopes,
        "entity.name.type",
        to_syntect_color(&syntax.type_name),
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        "support.type",
        to_syntect_color(&syntax.type_name),
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        "storage.type",
        to_syntect_color(&syntax.type_name),
        Some(FontStyle::BOLD),
    );

    // Variables and parameters
    push_scope(
        &mut scopes,
        "variable",
        to_syntect_color(&syntax.variable),
        None,
    );
    push_scope(
        &mut scopes,
        "variable.parameter",
        to_syntect_color(&syntax.variable),
        Some(FontStyle::ITALIC),
    );
    push_scope(
        &mut scopes,
        "entity.other.attribute-name",
        to_syntect_color(&syntax.variable),
        None,
    );

    syntect_theme.scopes = scopes;
    syntect_theme
}

fn push_scope(
    scopes: &mut Vec<ThemeItem>,
    selector: &str,
    color: SyntectColor,
    font_style: Option<FontStyle>,
) {
    if let Ok(scope) = ScopeSelectors::from_str(selector) {
        scopes.push(ThemeItem {
            scope,
            style: StyleModifier {
                foreground: Some(color),
                background: None,
                font_style,
            },
        });
    }
}

fn to_syntect_color(color: &Color) -> SyntectColor {
    let (r, g, b) = color_to_rgb(color);
    SyntectColor { r, g, b, a: 0xFF }
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
