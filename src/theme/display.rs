use super::*;

/// Lists all available themes from the given manager.
pub fn list_themes(manager: &ThemeManager) {
    let themes = manager.get_themes_by_luminosity();

    println!("Available themes:");
    println!();

    for (name, theme, luminosity) in themes {
        println!(
            "  {:<20} - {} (luminosity: {:.3})",
            name, theme.description, luminosity
        );
    }
}

/// Create a style from theme colors
pub fn create_style(theme: &Theme, element: ThemeElement) -> AnsiStyle {
    let color = match element {
        ThemeElement::Text => &theme.text,
        ThemeElement::TextLight => &theme.text_light,
        ThemeElement::LineNumber => &theme.line_number,
        ThemeElement::LineNumberSeparator => &theme.line_number_separator,
        ThemeElement::H1 => &theme.h1,
        ThemeElement::H2 => &theme.h2,
        ThemeElement::H3 => &theme.h3,
        ThemeElement::H4 => &theme.h4,
        ThemeElement::H5 => &theme.h5,
        ThemeElement::H6 => &theme.h6,
        ThemeElement::Code => &theme.code,
        ThemeElement::Quote => &theme.quote,
        ThemeElement::Link => &theme.link,
        ThemeElement::Emphasis => &theme.emphasis,
        ThemeElement::Strong => &theme.strong,
        ThemeElement::Strikethrough => &theme.strikethrough,
        ThemeElement::Underline => &theme.text,
        ThemeElement::Border => &theme.border,
        ThemeElement::ListMarker => &theme.list_marker,
        ThemeElement::TableHeader => &theme.table_header,
        ThemeElement::TableBorder => &theme.table_border,
        ThemeElement::Error => &theme.error,
        ThemeElement::Warning => &theme.warning,
    };

    let mut style = AnsiStyle::new().fg(color.clone().into());

    match element {
        ThemeElement::Strong | ThemeElement::H1 => style = style.bold(),
        ThemeElement::Emphasis => style = style.italic(),
        ThemeElement::Strikethrough => style = style.strikethrough(),
        ThemeElement::Underline => style = style.underline(),
        _ => {}
    }

    style
}
