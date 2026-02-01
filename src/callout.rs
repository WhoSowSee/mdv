use crate::theme::{Color, parse_color_value};
use anyhow::{Context, Result, anyhow, bail};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CustomCalloutStyle {
    pub icon: Option<String>,
    pub color: Option<Color>,
}

pub(crate) fn parse_custom_callouts(input: &str) -> Result<HashMap<String, CustomCalloutStyle>> {
    let mut callouts = HashMap::new();
    let mut has_entries = false;

    for raw_entry in input.split(|ch| ch == ';' || ch == '\n') {
        let entry = raw_entry.trim();
        if entry.is_empty() {
            continue;
        }

        has_entries = true;

        let (name_raw, values_raw) = entry
            .split_once(':')
            .ok_or_else(|| anyhow!("Custom callout entry '{}' must contain ':'", entry))?;

        let name = name_raw.trim();
        if name.is_empty() {
            bail!("Custom callout entry '{}' is missing a name.", entry);
        }

        if !is_valid_callout_name(name) {
            bail!(
                "Custom callout name '{}' contains invalid characters.",
                name
            );
        }

        let values = values_raw.trim();
        if values.is_empty() {
            bail!(
                "Custom callout '{}' must define at least one of icon or color.",
                name
            );
        }

        let normalized_name = name.to_ascii_lowercase();

        let mut icon = None;
        let mut color = None;

        for (key_raw, value_raw) in parse_callout_options(values)
            .with_context(|| format!("Custom callout '{}' has invalid options.", name))?
        {
            let key_raw = key_raw.trim();
            let value_raw = value_raw.trim();

            let key = key_raw.to_ascii_lowercase();
            let value = value_raw;
            if value.is_empty() {
                bail!(
                    "Custom callout '{}' option '{}' cannot be empty.",
                    name,
                    key
                );
            }

            match key.as_str() {
                "icon" => {
                    if icon.is_some() {
                        bail!("Custom callout '{}' repeats the icon option.", name);
                    }
                    icon = Some(value.to_string());
                }
                "color" => {
                    if color.is_some() {
                        bail!("Custom callout '{}' repeats the color option.", name);
                    }
                    let parsed = parse_color_value(value).with_context(|| {
                        format!("Custom callout '{}' has invalid color '{}'.", name, value)
                    })?;
                    color = Some(parsed);
                }
                _ => {
                    bail!(
                        "Custom callout '{}' has unknown option '{}'. Expected icon or color.",
                        name,
                        key
                    );
                }
            }
        }

        if icon.is_none() && color.is_none() {
            bail!(
                "Custom callout '{}' must define at least one of icon or color.",
                name
            );
        }

        if callouts
            .insert(normalized_name.clone(), CustomCalloutStyle { icon, color })
            .is_some()
        {
            bail!("Custom callout '{}' is defined more than once.", name);
        }
    }

    if !has_entries {
        bail!("Custom callout string is empty.");
    }

    Ok(callouts)
}

fn is_valid_callout_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn parse_callout_options(values: &str) -> Result<Vec<(String, String)>> {
    let mut options = Vec::new();
    let mut remaining = values.trim();

    while !remaining.is_empty() {
        let (key, value, rest) = parse_single_option(remaining)?;
        if key.trim().is_empty() {
            bail!("Custom callout option key cannot be empty.");
        }
        if value.trim().is_empty() {
            bail!("Custom callout option '{}' cannot be empty.", key.trim());
        }
        options.push((key.to_string(), value.to_string()));
        remaining = rest.trim_start();
        if remaining.starts_with(',') {
            remaining = remaining[1..].trim_start();
            if remaining.is_empty() {
                bail!("Custom callout contains a trailing comma.");
            }
        }
    }

    if options.is_empty() {
        bail!("Custom callout options are empty.");
    }

    Ok(options)
}

fn parse_single_option(input: &str) -> Result<(&str, &str, &str)> {
    let mut split = input.splitn(2, '=');
    let key = split
        .next()
        .ok_or_else(|| anyhow!("Custom callout option is missing a key."))?;
    let rest = split
        .next()
        .ok_or_else(|| anyhow!("Custom callout option '{}' must contain '='.", key.trim()))?;

    let (value, remaining) = split_value_and_rest(rest);
    Ok((key, value, remaining))
}

fn split_value_and_rest(input: &str) -> (&str, &str) {
    let mut idx = 0usize;
    let bytes = input.as_bytes();
    while idx < bytes.len() {
        if bytes[idx] == b',' {
            let candidate = &input[idx + 1..];
            if starts_with_option_key(candidate) {
                return (&input[..idx], &input[idx + 1..]);
            }
        }
        idx += 1;
    }
    (input, "")
}

fn starts_with_option_key(candidate: &str) -> bool {
    let trimmed = candidate.trim_start();
    trimmed
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("icon="))
        || trimmed
            .get(..6)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("color="))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_custom_callout_icon_only() {
        let parsed = parse_custom_callouts("custom:icon=*").expect("parse custom callout");
        let entry = parsed.get("custom").expect("custom callout present");
        assert_eq!(entry.icon.as_deref(), Some("*"));
        assert!(entry.color.is_none());
    }

    #[test]
    fn parse_custom_callout_color_only() {
        let parsed = parse_custom_callouts("note:color=ansi(42)").expect("parse custom callout");
        let entry = parsed.get("note").expect("note callout present");
        assert!(entry.icon.is_none());
        assert!(entry.color.is_some());
    }

    #[test]
    fn parse_custom_callout_multiple_entries() {
        let parsed =
            parse_custom_callouts("tip:icon=!;hint:icon=?").expect("parse custom callouts");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.get("tip").and_then(|v| v.icon.as_deref()), Some("!"));
        assert_eq!(
            parsed.get("hint").and_then(|v| v.icon.as_deref()),
            Some("?")
        );
    }

    #[test]
    fn parse_custom_callout_rejects_empty_input() {
        let err = parse_custom_callouts("  ").expect_err("empty input should fail");
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn parse_custom_callout_rejects_missing_options() {
        let err = parse_custom_callouts("custom:").expect_err("missing options should fail");
        assert!(err.to_string().contains("at least one"));
    }

    #[test]
    fn parse_custom_callout_rejects_invalid_name() {
        let err = parse_custom_callouts("bad name:icon=*").expect_err("invalid name should fail");
        assert!(err.to_string().contains("invalid characters"));
    }

    #[test]
    fn parse_custom_callout_rejects_unknown_option() {
        let err =
            parse_custom_callouts("custom:shape=box").expect_err("unknown option should fail");
        assert!(err.to_string().contains("unknown option"));
    }

    #[test]
    fn parse_custom_callout_color_rgb_tuple() {
        let parsed =
            parse_custom_callouts("important:color=122,23,44").expect("parse custom callout");
        let entry = parsed.get("important").expect("callout present");
        assert!(entry.color.is_some());
    }

    #[test]
    fn parse_custom_callout_color_and_icon_with_commas() {
        let parsed =
            parse_custom_callouts("tip:color=rgb(1,2,3),icon=*").expect("parse custom callout");
        let entry = parsed.get("tip").expect("callout present");
        assert!(entry.color.is_some());
        assert_eq!(entry.icon.as_deref(), Some("*"));
    }
}
