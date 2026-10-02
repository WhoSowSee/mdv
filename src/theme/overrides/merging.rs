use super::*;

pub(crate) fn merge_custom_theme_overrides(
    custom_theme: &mut Option<String>,
    custom_code_theme: &mut Option<String>,
    next_theme: Option<&str>,
    next_code_theme: Option<&str>,
) -> Result<()> {
    let mut theme_pairs = Vec::new();
    let mut syntax_pairs = Vec::new();
    for (theme, code_theme) in [
        (custom_theme.as_deref(), custom_code_theme.as_deref()),
        (next_theme, next_code_theme),
    ] {
        if let Some(overrides) = theme {
            for (key, value) in parse_override_pairs(overrides)? {
                let normalized = normalize_key(&key);
                if let Some(field) = normalized.strip_prefix("syntax:") {
                    syntax_pairs.push((field.to_string(), value));
                } else {
                    theme_pairs.push((key, value));
                }
            }
        }
        if let Some(overrides) = code_theme {
            for (key, value) in parse_override_pairs(overrides)? {
                let normalized = normalize_key(&key);
                let field = normalized.strip_prefix("syntax:").unwrap_or(&normalized);
                syntax_pairs.push((field.to_string(), value));
            }
        }
    }
    *custom_theme = format_pairs(theme_pairs);
    *custom_code_theme = format_pairs(syntax_pairs);
    Ok(())
}

fn format_pairs(pairs: Vec<(String, String)>) -> Option<String> {
    if pairs.is_empty() {
        return None;
    }
    Some(
        pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(";"),
    )
}
