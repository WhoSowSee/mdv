use anyhow::{Context, Result, bail};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CustomCodeBlock {
    pub icon: Option<String>,
    pub label: Option<String>,
    pub aliases: Vec<String>,
}

pub(crate) fn parse_custom_code_blocks(input: &str) -> Result<HashMap<String, CustomCodeBlock>> {
    let mut blocks = HashMap::new();
    let mut has_entries = false;

    for raw_entry in input.split([';', '\n']) {
        let entry = raw_entry.trim_start();
        if entry.is_empty() {
            continue;
        }

        has_entries = true;

        let (name_raw, values_raw) = entry.split_once(':').ok_or_else(|| {
            anyhow::anyhow!("Custom code block entry '{}' must contain ':'", entry)
        })?;

        let name = name_raw.trim();
        if name.is_empty() {
            bail!(
                "Custom code block entry '{}' is missing a language name.",
                entry
            );
        }

        if !is_valid_code_block_name(name) {
            bail!(
                "Custom code block name '{}' contains invalid characters.",
                name
            );
        }

        let values = values_raw.trim_start();
        if values.is_empty() {
            bail!(
                "Custom code block '{}' must define at least one option.",
                name
            );
        }

        let normalized_name = name.to_ascii_lowercase();
        let mut icon = None;
        let mut label = None;
        let mut aliases = Vec::new();

        for (key_raw, value_raw) in parse_code_block_options(values)
            .with_context(|| format!("Custom code block '{}' has invalid options.", name))?
        {
            let key = key_raw.trim().to_ascii_lowercase();
            let value = value_raw;
            if value.is_empty() {
                bail!(
                    "Custom code block '{}' option '{}' cannot be empty.",
                    name,
                    key_raw
                );
            }

            match key.as_str() {
                "icon" => {
                    if icon.is_some() {
                        bail!("Custom code block '{}' repeats the icon option.", name);
                    }
                    icon = Some(value.to_string());
                }
                "label" => {
                    if label.is_some() {
                        bail!("Custom code block '{}' repeats the label option.", name);
                    }
                    label = Some(value.to_string());
                }
                "aliases" => {
                    if !aliases.is_empty() {
                        bail!("Custom code block '{}' repeats the aliases option.", name);
                    }
                    aliases = value
                        .split('|')
                        .map(|s| s.trim().to_ascii_lowercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {
                    bail!(
                        "Custom code block '{}' has unknown option '{}'. Expected icon, label or aliases.",
                        name,
                        key_raw
                    );
                }
            }
        }

        if icon.is_none() && label.is_none() {
            bail!(
                "Custom code block '{}' must define at least one of icon or label.",
                name
            );
        }

        if blocks
            .insert(
                normalized_name,
                CustomCodeBlock {
                    icon,
                    label,
                    aliases,
                },
            )
            .is_some()
        {
            bail!("Custom code block '{}' is defined more than once.", name);
        }
    }

    if !has_entries {
        bail!("Custom code block string is empty.");
    }

    Ok(blocks)
}

fn is_valid_code_block_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '+' | '#' | '.'))
}

fn parse_code_block_options(values: &str) -> Result<Vec<(String, String)>> {
    let mut options = Vec::new();
    let mut remaining = values.trim_start().to_string();

    while !remaining.is_empty() {
        let (key, rest) = split_value_and_rest(&remaining);
        if key.is_empty() {
            bail!("Empty key in custom code block options '{}'.", values);
        }

        let (value, rest) = split_value_and_rest(&rest);
        if value.is_empty() {
            bail!("Option '{}' in custom code block has no value.", key);
        }

        options.push((key, value));
        remaining = rest.trim_start_matches([',', ';', '=']).trim().to_string();
    }

    Ok(options)
}

fn split_value_and_rest(input: &str) -> (String, String) {
    let mut chars = input.chars().peekable();
    let mut value = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == ',' || ch == ';' || ch == '=' {
            break;
        }
        value.push(ch);
        chars.next();
    }

    let rest: String = chars.collect();
    let rest = rest.trim_start_matches([',', ';', '=']);

    (value, rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_icon_and_label() {
        let blocks =
            parse_custom_code_blocks("rust:icon=*,label=russst;py:icon=?,label=pyt4on").unwrap();
        assert_eq!(blocks.get("rust").unwrap().icon.as_deref(), Some("*"));
        assert_eq!(blocks.get("rust").unwrap().label.as_deref(), Some("russst"));
        assert_eq!(blocks.get("py").unwrap().icon.as_deref(), Some("?"));
        assert_eq!(blocks.get("py").unwrap().label.as_deref(), Some("pyt4on"));
    }

    #[test]
    fn parses_aliases() {
        let blocks = parse_custom_code_blocks("python:icon=*,aliases=py|py3").unwrap();
        let block = blocks.get("python").unwrap();
        assert_eq!(block.icon.as_deref(), Some("*"));
        assert_eq!(block.aliases, vec!["py", "py3"]);
    }

    #[test]
    fn parses_icon_only() {
        let blocks = parse_custom_code_blocks("rust:icon=").unwrap();
        assert_eq!(blocks.get("rust").unwrap().icon.as_deref(), Some(""));
        assert!(blocks.get("rust").unwrap().label.is_none());
    }

    #[test]
    fn parses_label_only() {
        let blocks = parse_custom_code_blocks("rust:label=RustLang").unwrap();
        assert!(blocks.get("rust").unwrap().icon.is_none());
        assert_eq!(
            blocks.get("rust").unwrap().label.as_deref(),
            Some("RustLang")
        );
    }

    #[test]
    fn preserves_trailing_spaces_in_icon_value() {
        let blocks = parse_custom_code_blocks("python:icon=*   ").unwrap();
        let icon = blocks.get("python").cloned().unwrap().icon.unwrap();
        assert_eq!(icon.as_bytes(), b"*   ", "icon={:?}", icon);
    }

    #[test]
    fn normalizes_language_name_to_lowercase() {
        let blocks = parse_custom_code_blocks("Rust:icon=").unwrap();
        assert_eq!(blocks.get("rust").unwrap().icon.as_deref(), Some(""));
    }

    #[test]
    fn rejects_empty_string() {
        assert!(parse_custom_code_blocks("").is_err());
    }

    #[test]
    fn rejects_missing_options() {
        assert!(parse_custom_code_blocks("rust:").is_err());
    }

    #[test]
    fn rejects_empty_option_value() {
        assert!(parse_custom_code_blocks("rust:icon=").is_err());
    }

    #[test]
    fn rejects_unknown_option() {
        assert!(parse_custom_code_blocks("rust:color=red").is_err());
    }

    #[test]
    fn allows_special_language_characters() {
        let blocks = parse_custom_code_blocks("c++:icon=;c#:icon=").unwrap();
        assert_eq!(blocks.get("c++").unwrap().icon.as_deref(), Some(""));
        assert_eq!(blocks.get("c#").unwrap().icon.as_deref(), Some(""));
    }
}
