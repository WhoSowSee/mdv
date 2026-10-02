use super::*;

/// Lists all available themes from the given manager.
/// Returns stdout write or flush failures.
pub fn list_themes(manager: &ThemeManager) -> std::io::Result<()> {
    use std::io::Write;
    let themes = manager.get_themes_by_luminosity();
    let mut output = std::io::stdout().lock();
    writeln!(output, "Available themes:\n")?;

    for (name, theme, luminosity) in themes {
        writeln!(
            output,
            "  {:<20} - {} (luminosity: {:.3})",
            name, theme.description, luminosity
        )?;
    }
    output.flush()
}

/// Create a style from theme colors
pub fn create_style(theme: &Theme, element: ThemeElement) -> AnsiStyle {
    let color = match element {
        ThemeElement::Text => Some(&theme.text),
        ThemeElement::TextLight => Some(&theme.text_light),
        ThemeElement::LineNumber => Some(&theme.line_number.number),
        ThemeElement::LineNumberSeparator => Some(&theme.line_number.separator),
        ThemeElement::H1 => Some(&theme.h1),
        ThemeElement::H2 => Some(&theme.h2),
        ThemeElement::H3 => Some(&theme.h3),
        ThemeElement::H4 => Some(&theme.h4),
        ThemeElement::H5 => Some(&theme.h5),
        ThemeElement::H6 => Some(&theme.h6),
        ThemeElement::Code => Some(&theme.code.text),
        ThemeElement::Math => Some(&theme.math.text),
        ThemeElement::MathBorder => theme.math.border.as_ref(),
        ThemeElement::Quote => Some(&theme.quote),
        ThemeElement::Link => Some(&theme.link),
        ThemeElement::Emphasis => Some(&theme.emphasis.text),
        ThemeElement::Strong => Some(&theme.strong.text),
        ThemeElement::Strikethrough => Some(&theme.strikethrough.text),
        ThemeElement::Underline => Some(&theme.text),
        ThemeElement::DetailsBorder => theme.details_border.as_ref(),
        ThemeElement::CodeBlockLabel => Some(&theme.code_block.label),
        ThemeElement::CalloutLabel => theme.callout.label.as_ref(),
        ThemeElement::FrontMatterTitle => Some(&theme.front_matter.title),
        ThemeElement::FrontMatterKey => Some(&theme.front_matter.key),
        ThemeElement::FrontMatterValue => Some(&theme.front_matter.value),
        ThemeElement::FrontMatterBorder => theme.front_matter.border.as_ref(),
        ThemeElement::OrderedListMarker => Some(&theme.list.ordered),
        ThemeElement::UnorderedListMarker => Some(&theme.list.unordered),
        ThemeElement::TodoChecked => Some(&theme.todo.checked),
        ThemeElement::TodoUnchecked => Some(&theme.todo.unchecked),
        ThemeElement::TableHeader => Some(&theme.table.header),
        ThemeElement::TableBorder => theme.table.border.as_ref(),
        ThemeElement::Error => Some(&theme.error),
        ThemeElement::Warning => Some(&theme.warning),
    };

    let mut style = AnsiStyle::new();
    if let Some(color) = color {
        style = style.fg(color.clone().into());
    }

    match element {
        ThemeElement::Strong
        | ThemeElement::H1
        | ThemeElement::FrontMatterTitle
        | ThemeElement::CalloutLabel => style = style.bold(),
        ThemeElement::Emphasis => style = style.italic(),
        ThemeElement::Strikethrough => style = style.strikethrough(),
        ThemeElement::Underline => style = style.underline(),
        _ => {}
    }

    style
}
