use crate::theme::parse_color_value;
use serde::{Deserialize, Deserializer, de};
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrMap<K, V>
where
    K: Ord,
{
    String(String),
    Mapping(BTreeMap<K, V>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CustomCodeBlockEntry {
    icon: Option<String>,
    label: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IconColorEntry {
    icon: Option<String>,
    color: Option<Value>,
}

pub(super) fn deserialize_theme_overrides<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<StringOrMap<String, Value>>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrMap::String(value)) => Ok(Some(value)),
        Some(StringOrMap::Mapping(entries)) => {
            mapping_to_assignments(entries).map_err(de::Error::custom)
        }
    }
}

fn mapping_to_assignments(entries: BTreeMap<String, Value>) -> Result<Option<String>, String> {
    if entries.is_empty() {
        return Ok(None);
    }

    entries
        .into_iter()
        .map(|(key, value)| {
            if key.trim().is_empty() {
                return Err("Theme override key cannot be empty.".to_string());
            }
            let value = scalar_to_string(value, &format!("Theme override '{key}'"))?;
            reject_delimiters(&value, [';', '\n'], &format!("Theme override '{key}'"))?;
            Ok(format!("{key}={value}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|entries| Some(entries.join(";")))
}

pub(super) fn deserialize_custom_callout<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = match Option::<StringOrMap<String, IconColorEntry>>::deserialize(deserializer)? {
        None => return Ok(None),
        Some(StringOrMap::String(value)) => return Ok(Some(value)),
        Some(StringOrMap::Mapping(entries)) if entries.is_empty() => return Ok(None),
        Some(StringOrMap::Mapping(entries)) => entries
            .into_iter()
            .map(|(name, entry)| {
                let mut options = Vec::new();
                if let Some(icon) = entry.icon {
                    reject_delimiters(&icon, [',', ';', '\n'], "Custom callout icon")?;
                    options.push(format!("icon={icon}"));
                }
                if let Some(color) = entry.color {
                    options.push(format!(
                        "color={}",
                        scalar_to_string(color, "Custom callout color")?
                    ));
                }
                if options.is_empty() {
                    return Err(format!(
                        "Custom callout '{name}' must define at least one of icon or color."
                    ));
                }
                Ok(format!("{name}:{}", options.join(",")))
            })
            .collect::<Result<Vec<_>, String>>()
            .map_err(de::Error::custom)?
            .join(";"),
    };

    Ok(Some(raw))
}

pub(super) fn deserialize_custom_code_block<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = match Option::<StringOrMap<String, CustomCodeBlockEntry>>::deserialize(deserializer)?
    {
        None => return Ok(None),
        Some(StringOrMap::String(value)) => return Ok(Some(value)),
        Some(StringOrMap::Mapping(entries)) if entries.is_empty() => return Ok(None),
        Some(StringOrMap::Mapping(entries)) => entries
            .into_iter()
            .map(|(name, entry)| {
                let mut options = Vec::new();
                if let Some(icon) = entry.icon {
                    reject_delimiters(&icon, [',', ';', '=', '\n'], "Custom code block icon")?;
                    options.push(format!("icon={icon}"));
                }
                if let Some(label) = entry.label {
                    reject_delimiters(&label, [',', ';', '=', '\n'], "Custom code block label")?;
                    options.push(format!("label={label}"));
                }
                if !entry.aliases.is_empty() {
                    for alias in &entry.aliases {
                        reject_delimiters(
                            alias,
                            ['|', ',', ';', '=', '\n'],
                            "Custom code block alias",
                        )?;
                    }
                    options.push(format!("aliases={}", entry.aliases.join("|")));
                }
                if options.is_empty() {
                    return Err(format!(
                        "Custom code block '{name}' must define at least one option."
                    ));
                }
                Ok(format!("{name}:{}", options.join(",")))
            })
            .collect::<Result<Vec<_>, String>>()
            .map_err(de::Error::custom)?
            .join(";"),
    };

    Ok(Some(raw))
}

pub(super) fn deserialize_custom_checkbox<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = match Option::<StringOrMap<String, IconColorEntry>>::deserialize(deserializer)? {
        None => return Ok(None),
        Some(StringOrMap::String(value)) => return Ok(Some(value)),
        Some(StringOrMap::Mapping(entries)) if entries.is_empty() => return Ok(None),
        Some(StringOrMap::Mapping(entries)) => entries
            .into_iter()
            .map(|(state, entry)| checkbox_entry_to_string(&state, entry))
            .collect::<Result<Vec<_>, String>>()
            .map_err(de::Error::custom)?
            .join(";"),
    };

    Ok(Some(raw))
}

fn checkbox_entry_to_string(state: &str, entry: IconColorEntry) -> Result<String, String> {
    if state.chars().count() != 1 || state.contains([':', ';', '\n']) {
        return Err(format!(
            "Custom checkbox state '{state}' must be exactly one non-delimiter character."
        ));
    }

    let color = entry
        .color
        .map(|value| scalar_to_string(value, "Custom checkbox color"))
        .transpose()?;
    match (entry.icon, color) {
        (Some(icon), Some(color)) => {
            reject_delimiters(&icon, [':', ';', '\n'], "Custom checkbox icon")?;
            Ok(format!("{state}:{icon}:{color}"))
        }
        (Some(icon), None) => {
            reject_delimiters(&icon, [':', ';', '\n'], "Custom checkbox icon")?;
            Ok(format!("{state}:{icon}:"))
        }
        (None, Some(color)) => Ok(format!("{state}:{color}")),
        (None, None) => Err(format!(
            "Custom checkbox state '{state}' must define icon or color."
        )),
    }
}

pub(super) fn deserialize_custom_list<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = match Option::<StringOrMap<usize, IconColorEntry>>::deserialize(deserializer)? {
        None => return Ok(None),
        Some(StringOrMap::String(value)) => return Ok(Some(value)),
        Some(StringOrMap::Mapping(entries)) if entries.is_empty() => return Ok(None),
        Some(StringOrMap::Mapping(entries)) => entries
            .into_iter()
            .map(|(level, entry)| list_entry_to_string(level, entry))
            .collect::<Result<Vec<_>, String>>()
            .map_err(de::Error::custom)?
            .join(";"),
    };

    Ok(Some(raw))
}

fn list_entry_to_string(level: usize, entry: IconColorEntry) -> Result<String, String> {
    let color = entry
        .color
        .map(|value| scalar_to_string(value, "Custom list color"))
        .transpose()?;
    match (entry.icon, color) {
        (Some(icon), Some(color)) => {
            reject_delimiters(&icon, [':', ';', '\n'], "Custom list icon")?;
            Ok(format!("{level}:{icon}:{color}"))
        }
        (Some(icon), None) => {
            reject_delimiters(&icon, [':', ';', '\n'], "Custom list icon")?;
            if parse_color_value(&icon).is_ok() {
                return Err(format!(
                    "Custom list icon '{icon}' is ambiguous with a color value."
                ));
            }
            Ok(format!("{level}:{icon}"))
        }
        (None, Some(color)) => Ok(format!("{level}:{color}")),
        (None, None) => Err(format!(
            "Custom list level {level} must define icon or color."
        )),
    }
}

fn scalar_to_string(value: Value, context: &str) -> Result<String, String> {
    match value {
        Value::Null => Ok("none".to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value),
        _ => Err(format!("{context} must be a scalar YAML value.")),
    }
}

fn reject_delimiters<const N: usize>(
    value: &str,
    delimiters: [char; N],
    context: &str,
) -> Result<(), String> {
    if let Some(delimiter) = delimiters
        .into_iter()
        .find(|delimiter| value.contains(*delimiter))
    {
        return Err(format!(
            "{context} cannot contain the '{delimiter}' delimiter."
        ));
    }
    Ok(())
}
