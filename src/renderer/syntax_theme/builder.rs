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
    for field in crate::theme::SyntaxField::ALL {
        for &(selector, font_style) in super::scopes::syntax_scopes(field) {
            push_scope(
                &mut syntect_theme.scopes,
                &mut palette,
                selector,
                field.color(&theme.syntax),
                font_style,
            );
        }
    }
    CodeHighlightTheme {
        syntect: syntect_theme,
        palette,
    }
}

pub(super) fn register_color(
    palette: &mut HashMap<(u8, u8, u8), Color>,
    color: &Color,
) -> SyntectColor {
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

pub(super) fn push_scope(
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
