use super::*;

mod merging;
mod sections;

pub(crate) use merging::merge_custom_theme_overrides;

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
        let normalized = normalize_key(&key);
        let field = normalized.strip_prefix("syntax:").unwrap_or(&normalized);
        apply_code_theme_override(&mut theme.syntax, field, &value)
            .with_context(|| format!("Failed to apply syntax override '{}={}'", key, value))?;
    }
    Ok(())
}

pub(super) fn parse_override_pairs(input: &str) -> Result<Vec<(String, String)>> {
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

    if let Some((section, field)) = normalized_key.split_once(':') {
        return sections::apply_section_override(theme, section, field, value);
    }

    match normalized_key.as_str() {
        "text" => theme.text = parse_color_spec(value)?,
        "text_light" | "textlight" => theme.text_light = parse_color_spec(value)?,
        "h1" => theme.h1 = parse_color_spec(value)?,
        "h2" => theme.h2 = parse_color_spec(value)?,
        "h3" => theme.h3 = parse_color_spec(value)?,
        "h4" => theme.h4 = parse_color_spec(value)?,
        "h5" => theme.h5 = parse_color_spec(value)?,
        "h6" => theme.h6 = parse_color_spec(value)?,
        "quote" => theme.quote = parse_color_spec(value)?,
        "link" => theme.link = parse_color_spec(value)?,
        "background" | "bg" => theme.background = parse_optional_color_spec(value)?,
        "horizontal_rule" | "horizontalrule" => {
            theme.horizontal_rule = parse_optional_color_spec(value)?
        }
        "details_border" => theme.details_border = parse_optional_color_spec(value)?,
        "footnote_separator" | "footnoteseparator" => {
            theme.footnote_separator = parse_optional_color_spec(value)?
        }
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

    SyntaxField::parse(&normalized_key)?.set(syntax, parse_color_spec(value)?);

    Ok(())
}

pub(super) fn normalize_key(key: &str) -> String {
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
