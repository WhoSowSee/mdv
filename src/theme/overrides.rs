use super::*;

/// Applies `key=value` overrides separated by semicolons or newlines.
pub fn apply_custom_theme(theme: &mut Theme, overrides: &str) -> Result<()> {
    for (key, value) in parse_override_pairs(overrides)? {
        apply_theme_override(theme, &key, &value)
            .with_context(|| format!("Failed to apply override '{}={}'", key, value))?;
    }
    Ok(())
}

/// Apply overrides for syntax highlighting colors using the same format as [`apply_custom_theme`]
pub fn apply_custom_code_theme(theme: &mut Theme, overrides: &str) -> Result<()> {
    for (key, value) in parse_override_pairs(overrides)? {
        apply_code_theme_override(&mut theme.syntax, &key, &value)
            .with_context(|| format!("Failed to apply syntax override '{}={}'", key, value))?;
    }
    Ok(())
}

fn parse_override_pairs(input: &str) -> Result<Vec<(String, String)>> {
    let mut pairs = Vec::new();

    for raw in input.split([';', '\n']) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (key, value) = trimmed
            .split_once('=')
            .ok_or_else(|| anyhow!("Override pair '{}' must contain '='", trimmed))?;

        let key = key.trim();
        let value = value.trim();

        if key.is_empty() {
            bail!("Found empty key in override '{}'.", trimmed);
        }

        if value.is_empty() {
            bail!("Key '{}' has an empty value in override.", key);
        }

        pairs.push((key.to_string(), value.to_string()));
    }

    if pairs.is_empty() {
        bail!("Override string is empty.");
    }

    Ok(pairs)
}

fn apply_theme_override(theme: &mut Theme, key: &str, value: &str) -> Result<()> {
    let normalized_key = normalize_key(key);

    match normalized_key.as_str() {
        "text" => theme.text = parse_color_spec(value)?,
        "text_light" | "textlight" => theme.text_light = parse_color_spec(value)?,
        "line_number" | "linenumber" => theme.line_number = parse_color_spec(value)?,
        "line_number_separator" | "linenumberseparator" => {
            theme.line_number_separator = parse_color_spec(value)?
        }
        "pager_status_bar_transparent" | "pagerstatusbartransparent" => {
            theme.pager_status_bar_transparent = parse_bool_spec(value)?
        }
        "h1" => theme.h1 = parse_color_spec(value)?,
        "h2" => theme.h2 = parse_color_spec(value)?,
        "h3" => theme.h3 = parse_color_spec(value)?,
        "h4" => theme.h4 = parse_color_spec(value)?,
        "h5" => theme.h5 = parse_color_spec(value)?,
        "h6" => theme.h6 = parse_color_spec(value)?,
        "code" => theme.code = parse_color_spec(value)?,
        "quote" => theme.quote = parse_color_spec(value)?,
        "link" => theme.link = parse_color_spec(value)?,
        "emphasis" => theme.emphasis = parse_color_spec(value)?,
        "strong" => theme.strong = parse_color_spec(value)?,
        "strong_emphasis" | "strongemphasis" => {
            theme.strong_emphasis = parse_optional_color_spec(value)?
        }
        "strikethrough" | "strike" | "del" => theme.strikethrough = parse_color_spec(value)?,
        "highlight" => theme.highlight = parse_optional_color_spec(value)?,
        "highlight_background" | "highlight_bg" => {
            theme.highlight_background = parse_color_spec(value)?
        }
        "emphasis_background" | "emphasis_bg" => {
            theme.emphasis_background = parse_optional_color_spec(value)?
        }
        "strong_background" | "strong_bg" => {
            theme.strong_background = parse_optional_color_spec(value)?
        }
        "strong_emphasis_background" | "strong_emphasis_bg" => {
            theme.strong_emphasis_background = parse_optional_color_spec(value)?
        }
        "code_background" | "code_bg" => theme.code_background = parse_optional_color_spec(value)?,
        "strikethrough_background" | "strikethrough_bg" | "strike_background" | "strike_bg" => {
            theme.strikethrough_background = parse_optional_color_spec(value)?
        }
        "background" | "bg" => theme.background = parse_optional_color_spec(value)?,
        "border" => theme.border = parse_color_spec(value)?,
        "front_matter_title" | "frontmattertitle" => {
            theme.front_matter_title = parse_optional_color_spec(value)?
        }
        "front_matter_key" | "frontmatterkey" => {
            theme.front_matter_key = parse_optional_color_spec(value)?
        }
        "front_matter_value" | "frontmattervalue" => {
            theme.front_matter_value = parse_optional_color_spec(value)?
        }
        "front_matter_border" | "frontmatterborder" => {
            theme.front_matter_border = parse_optional_color_spec(value)?
        }
        "list_marker" | "listmarker" => theme.list_marker = parse_color_spec(value)?,
        "table_header" | "tableheader" => theme.table_header = parse_color_spec(value)?,
        "table_border" | "tableborder" => theme.table_border = parse_color_spec(value)?,
        "error" => theme.error = parse_color_spec(value)?,
        "warning" => theme.warning = parse_color_spec(value)?,
        other => bail!("Unknown key for custom theme: '{}'.", other),
    }

    Ok(())
}

fn parse_optional_color_spec(value: &str) -> Result<Option<Color>> {
    if is_none_value(value) {
        Ok(None)
    } else {
        parse_color_spec(value).map(Some)
    }
}

fn apply_code_theme_override(syntax: &mut SyntaxTheme, key: &str, value: &str) -> Result<()> {
    let normalized_key = normalize_key(key);

    match normalized_key.as_str() {
        "keyword" => syntax.keyword = parse_color_spec(value)?,
        "string" => syntax.string = parse_color_spec(value)?,
        "comment" => syntax.comment = parse_color_spec(value)?,
        "number" => syntax.number = parse_color_spec(value)?,
        "operator" => syntax.operator = parse_color_spec(value)?,
        "function" => syntax.function = parse_color_spec(value)?,
        "variable" => syntax.variable = parse_color_spec(value)?,
        "type_name" | "typename" | "type" => syntax.type_name = parse_color_spec(value)?,
        other => bail!("Unknown key for custom syntax theme: '{}'.", other),
    }

    Ok(())
}

fn normalize_key(key: &str) -> String {
    key.trim()
        .replace(['-', ' '], "_")
        .replace("__", "_")
        .to_ascii_lowercase()
}

fn parse_bool_spec(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Boolean value '{}' must be 'true' or 'false'.", value),
    }
}
