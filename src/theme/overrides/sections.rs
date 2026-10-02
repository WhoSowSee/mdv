use super::*;

pub(super) fn apply_section_override(
    theme: &mut Theme,
    section: &str,
    field: &str,
    value: &str,
) -> Result<()> {
    if section == "pager" {
        return apply_pager_override(&mut theme.pager, field, value);
    }
    if section == "syntax" {
        apply_code_theme_override(&mut theme.syntax, field, value)?;
        return Ok(());
    }
    if section == "callout"
        && let Some(kind) = field.strip_prefix("palette:")
    {
        *theme.callout.palette.color_mut(kind)? = parse_color_spec(value)?;
        return Ok(());
    }
    match (section, field) {
        ("emphasis", "text") => theme.emphasis.text = parse_color_spec(value)?,
        ("emphasis", "background") => theme.emphasis.background = parse_optional_color_spec(value)?,
        ("strong", "text") => theme.strong.text = parse_color_spec(value)?,
        ("strong", "background") => theme.strong.background = parse_optional_color_spec(value)?,
        ("strong_emphasis", "text") => {
            theme.strong_emphasis.text = parse_optional_color_spec(value)?
        }
        ("strong_emphasis", "background") => {
            theme.strong_emphasis.background = parse_optional_color_spec(value)?
        }
        ("code", "text") => theme.code.text = parse_color_spec(value)?,
        ("code", "background") => theme.code.background = parse_optional_color_spec(value)?,
        ("strikethrough", "text") => theme.strikethrough.text = parse_color_spec(value)?,
        ("strikethrough", "background") => {
            theme.strikethrough.background = parse_optional_color_spec(value)?
        }
        ("highlight", "text") => theme.highlight.text = parse_optional_color_spec(value)?,
        ("highlight", "background") => theme.highlight.background = parse_color_spec(value)?,

        ("line_number", "number") => theme.line_number.number = parse_color_spec(value)?,
        ("line_number", "separator") => theme.line_number.separator = parse_color_spec(value)?,
        ("table", "header") => theme.table.header = parse_color_spec(value)?,
        ("table", "border") => theme.table.border = parse_optional_color_spec(value)?,
        ("math", "text") => theme.math.text = parse_color_spec(value)?,
        ("math", "border") => theme.math.border = parse_optional_color_spec(value)?,
        ("front_matter", "title") => theme.front_matter.title = parse_color_spec(value)?,
        ("front_matter", "key") => theme.front_matter.key = parse_color_spec(value)?,
        ("front_matter", "value") => theme.front_matter.value = parse_color_spec(value)?,
        ("front_matter", "border") => theme.front_matter.border = parse_optional_color_spec(value)?,
        ("code_block", "label") => theme.code_block.label = parse_color_spec(value)?,
        ("code_block", "border") => theme.code_block.border = parse_optional_color_spec(value)?,
        ("callout", "label") => theme.callout.label = parse_optional_color_spec(value)?,
        ("callout", "border") => theme.callout.border = parse_optional_color_spec(value)?,
        ("list", "ordered") => theme.list.ordered = parse_color_spec(value)?,
        ("list", "unordered") => theme.list.unordered = parse_color_spec(value)?,
        ("todo", "checked") => theme.todo.checked = parse_color_spec(value)?,
        ("todo", "unchecked") => theme.todo.unchecked = parse_color_spec(value)?,
        _ => bail!("Unknown key for custom theme: '{section}:{field}'."),
    }
    Ok(())
}

fn apply_pager_override(pager: &mut PagerTheme, field: &str, value: &str) -> Result<()> {
    if field == "transparent" {
        pager.transparent = parse_bool_spec(value)?;
        return Ok(());
    }
    let target = match field {
        "title" => &mut pager.title,
        "help" => &mut pager.help,
        "file_name" => &mut pager.file_name,
        "progress" => &mut pager.progress,
        "matches" => &mut pager.matches,
        _ => {
            let Some((kind, property)) = field.split_once(':') else {
                bail!("Unknown key for custom theme: 'pager:{field}'.");
            };
            let colors = match kind {
                "selection" => &mut pager.selection,
                "search" => &mut pager.search,
                "search_current" => &mut pager.search_current,
                _ => bail!("Unknown key for custom theme: 'pager:{field}'."),
            };
            match property {
                "text" => &mut colors.text,
                "background" => &mut colors.background,
                _ => bail!("Unknown key for custom theme: 'pager:{field}'."),
            }
        }
    };
    *target = parse_optional_color_spec(value)?;
    Ok(())
}
