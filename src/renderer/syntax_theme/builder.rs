use super::*;

/// Builds a syntax theme whose reset color inherits the terminal foreground.
pub(crate) fn build_syntect_theme(theme: &Theme) -> CodeHighlightTheme {
    let mut palette = HashMap::new();
    let mut syntect_theme = SyntectTheme {
        name: Some(format!("mdv:{}", theme.name)),
        ..SyntectTheme::default()
    };
    syntect_theme.settings.foreground = Some(register_color(&mut palette, &theme.text));
    if let Some(background) = theme.background.as_ref() {
        syntect_theme.settings.background = Some(register_color(&mut palette, background));
    }
    syntect_theme.settings.caret = Some(register_color(&mut palette, &theme.text));
    syntect_theme.settings.selection = Some(register_color(&mut palette, &theme.text_light));
    syntect_theme.settings.inactive_selection = syntect_theme.settings.selection;

    let syntax = &theme.syntax;
    let mut scopes: Vec<ThemeItem> = Vec::new();

    // Comments
    push_scope(
        &mut scopes,
        &mut palette,
        "comment",
        &syntax.comment,
        Some(FontStyle::ITALIC),
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "punctuation.definition.comment",
        &syntax.comment,
        Some(FontStyle::ITALIC),
    );

    // Keywords and directives
    push_scope(
        &mut scopes,
        &mut palette,
        "keyword",
        &syntax.keyword,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "storage",
        &syntax.keyword,
        Some(FontStyle::BOLD),
    );

    // Outrank the bare storage.type selector (primitive types in type_name) for declaration keywords.
    for selector in [
        "storage.type.function",
        "storage.type.struct",
        "storage.type.impl",
        "storage.type.trait",
        "storage.type.enum",
        "storage.type.union",
        "storage.type.mod",
    ] {
        push_scope(
            &mut scopes,
            &mut palette,
            selector,
            &syntax.keyword,
            Some(FontStyle::BOLD),
        );
    }
    push_scope(
        &mut scopes,
        &mut palette,
        "meta.directive",
        &syntax.keyword,
        Some(FontStyle::BOLD),
    );

    // Operators and punctuation
    push_scope(
        &mut scopes,
        &mut palette,
        "keyword.operator",
        &syntax.operator,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "punctuation",
        &syntax.operator,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "meta.brace",
        &syntax.operator,
        None,
    );

    // Strings
    push_scope(&mut scopes, &mut palette, "string", &syntax.string, None);
    push_scope(
        &mut scopes,
        &mut palette,
        "constant.character.escape",
        &syntax.string,
        None,
    );

    // Numbers and constants
    push_scope(
        &mut scopes,
        &mut palette,
        "constant.numeric",
        &syntax.number,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "constant.language",
        &syntax.number,
        None,
    );

    // Functions and methods
    push_scope(
        &mut scopes,
        &mut palette,
        "entity.name.function",
        &syntax.function,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "support.function",
        &syntax.function,
        None,
    );

    // Types and classes
    push_scope(
        &mut scopes,
        &mut palette,
        "entity.name.type",
        &syntax.type_name,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "support.type",
        &syntax.type_name,
        Some(FontStyle::BOLD),
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "storage.type",
        &syntax.type_name,
        Some(FontStyle::BOLD),
    );

    // Variables and parameters
    push_scope(
        &mut scopes,
        &mut palette,
        "variable",
        &syntax.variable,
        None,
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "variable.parameter",
        &syntax.variable,
        Some(FontStyle::ITALIC),
    );
    push_scope(
        &mut scopes,
        &mut palette,
        "entity.other.attribute-name",
        &syntax.variable,
        None,
    );

    syntect_theme.scopes = scopes;
    CodeHighlightTheme {
        syntect: syntect_theme,
        palette,
    }
}

fn register_color(palette: &mut HashMap<(u8, u8, u8), Color>, color: &Color) -> SyntectColor {
    if let Some(syntect) = transparent_for_reset(color) {
        return syntect;
    }
    let rgb = color_to_rgb(color);
    // Prefer palette/named over Rgb on RGB collision — it follows the terminal better.
    palette
        .entry(rgb)
        .and_modify(|existing| {
            if matches!(existing, Color::Rgb { .. }) && !matches!(color, Color::Rgb { .. }) {
                *existing = color.clone();
            }
        })
        .or_insert_with(|| color.clone());
    let (r, g, b) = rgb;
    SyntectColor { r, g, b, a: 0xFF }
}

fn push_scope(
    scopes: &mut Vec<ThemeItem>,
    palette: &mut HashMap<(u8, u8, u8), Color>,
    selector: &str,
    color: &Color,
    font_style: Option<FontStyle>,
) {
    let syntect_color = register_color(palette, color);
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

/// `Color::Reset` → transparent sentinel (`a == 0`); the escaper emits `\x1b[39m`.
pub(super) fn transparent_for_reset(color: &Color) -> Option<SyntectColor> {
    match color {
        Color::Reset => Some(SyntectColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }),
        _ => None,
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
