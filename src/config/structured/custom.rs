use super::{StringOrMap, reject_delimiters, scalar_to_string};
use crate::config::custom_overrides::CustomSetting;
use crate::theme::parse_color_value;
use serde::{Deserialize, Deserializer, de};
use serde_yaml::Value;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IconColorEntry {
    #[serde(default, deserialize_with = "present")]
    icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    color: Option<Option<Value>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeBlockEntry {
    #[serde(default, deserialize_with = "present")]
    icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    label: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    aliases: Option<Option<Vec<String>>>,
}

fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

fn option(key: &str, value: Option<String>, output: &mut Vec<String>) {
    output.push(format!("{key}={}", value.as_deref().unwrap_or("null")));
}

fn icon_color_options(
    entry: IconColorEntry,
    setting: CustomSetting,
) -> Result<Vec<String>, String> {
    if let Some(Some(icon)) = &entry.icon {
        if setting == CustomSetting::Callout {
            reject_delimiters(icon, [',', ';', '\n'], "Custom callout icon")?;
        } else {
            reject_delimiters(icon, [':', ';', '\n'], "Custom icon")?;
        }
    }
    let color = entry
        .color
        .map(|color| {
            color
                .map(|color| scalar_to_string(color, "Custom color"))
                .transpose()
        })
        .transpose()?;
    if setting != CustomSetting::Callout
        && !matches!(entry.icon, Some(None))
        && !matches!(color, Some(None))
    {
        let icon = entry.icon.flatten();
        let color = color.flatten();
        let value = match (icon, color) {
            (Some(icon), Some(color)) => format!("{icon}:{color}"),
            (Some(icon), None) => {
                if setting == CustomSetting::List && parse_color_value(&icon).is_ok() {
                    return Err(format!(
                        "Custom list icon '{icon}' is ambiguous with a color value."
                    ));
                }
                format!("{icon}:")
            }
            (None, Some(color)) => color,
            (None, None) => return Ok(Vec::new()),
        };
        return Ok(vec![value]);
    }
    let mut options = Vec::new();
    if let Some(icon) = entry.icon {
        option("icon", icon, &mut options);
    }
    if let Some(color) = color {
        option("color", color, &mut options);
    }
    Ok(options)
}

pub(in crate::config) fn deserialize_custom_callout<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_entries::<D, String, IconColorEntry>(deserializer, |entry| {
        icon_color_options(entry, CustomSetting::Callout)
    })
}

pub(in crate::config) fn deserialize_custom_checkbox<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_entries::<D, String, IconColorEntry>(deserializer, |entry| {
        icon_color_options(entry, CustomSetting::Checkbox)
    })
}

pub(in crate::config) fn deserialize_custom_list<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_entries::<D, usize, IconColorEntry>(deserializer, |entry| {
        icon_color_options(entry, CustomSetting::List)
    })
}

pub(in crate::config) fn deserialize_custom_code_block<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_entries::<D, String, CodeBlockEntry>(deserializer, |entry| {
        let mut options = Vec::new();
        for (key, value) in [("icon", entry.icon), ("label", entry.label)] {
            if let Some(value) = value {
                if let Some(value) = &value {
                    reject_delimiters(value, [',', ';', '=', '\n'], "Custom code block value")?;
                }
                option(key, value, &mut options);
            }
        }
        if let Some(aliases) = entry.aliases {
            let aliases = aliases.filter(|aliases| !aliases.is_empty());
            if let Some(aliases) = &aliases {
                for alias in aliases {
                    reject_delimiters(
                        alias,
                        ['|', ',', ';', '=', '\n'],
                        "Custom code block alias",
                    )?;
                }
            }
            option(
                "aliases",
                aliases.map(|aliases| aliases.join("|")),
                &mut options,
            );
        }
        Ok(options)
    })
}

fn deserialize_entries<'de, D, K, V>(
    deserializer: D,
    options: impl Fn(V) -> Result<Vec<String>, String>,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord + std::fmt::Display,
    V: Deserialize<'de>,
{
    match Option::<StringOrMap<K, Option<V>>>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrMap::String(value)) => Ok(Some(value)),
        Some(StringOrMap::Mapping(entries)) if entries.is_empty() => Ok(None),
        Some(StringOrMap::Mapping(entries)) => {
            let entries = entries
                .into_iter()
                .map(|(name, entry)| {
                    let Some(entry) = entry else {
                        return Ok(format!("{name}:null"));
                    };
                    let options = options(entry)?;
                    if options.is_empty() {
                        return Err(format!(
                            "Custom entry '{name}' must define at least one option."
                        ));
                    }
                    Ok(format!("{name}:{}", options.join(",")))
                })
                .collect::<Result<Vec<_>, String>>()
                .map_err(de::Error::custom)?;
            Ok(Some(entries.join(";")))
        }
    }
}
