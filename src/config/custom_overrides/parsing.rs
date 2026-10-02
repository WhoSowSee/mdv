use super::{CustomSetting, Entries, Fields};
use crate::callout::{is_valid_callout_name, parse_callout_options};
use crate::checkbox_override::CheckboxOverride;
use crate::custom_code_block::{is_valid_code_block_name, parse_code_block_options};
use crate::list_marker::ListMarkerOverride;
use crate::theme::parse_color_value;
use anyhow::{Context, Result, bail};

pub(super) fn parse(setting: CustomSetting, input: Option<&str>) -> Result<Option<Entries>> {
    let Some(input) = input else {
        return Ok(None);
    };
    if input.trim().eq_ignore_ascii_case("null") {
        return Ok(None);
    }
    let mut entries = Entries::new();
    for entry in input.split([';', '\n']) {
        if entry.trim().is_empty() {
            continue;
        }
        let (name, values) = entry
            .split_once(':')
            .with_context(|| format!("{} entry '{entry}' must contain ':'", setting.key()))?;
        let name = normalize_name(setting, name)?;
        let fields = if values.trim().eq_ignore_ascii_case("null") {
            None
        } else if matches!(setting, CustomSetting::Callout | CustomSetting::CodeBlock)
            || values.split_once('=').is_some_and(|(key, _)| {
                key.trim().eq_ignore_ascii_case("icon") || key.trim().eq_ignore_ascii_case("color")
            })
        {
            Some(named_fields(setting, &name, values)?)
        } else {
            Some(positional_fields(setting, entry, values)?)
        };
        if entries.insert(name.clone(), fields).is_some() {
            bail!(
                "{} entry '{name}' is defined more than once.",
                setting.key()
            );
        }
    }
    if entries.is_empty() {
        bail!("{} string is empty.", setting.key());
    }
    Ok(Some(entries))
}

fn normalize_name(setting: CustomSetting, name: &str) -> Result<String> {
    let valid = match setting {
        CustomSetting::Callout => is_valid_callout_name(name.trim()),
        CustomSetting::CodeBlock => is_valid_code_block_name(name.trim()),
        CustomSetting::Checkbox => name.chars().count() == 1,
        CustomSetting::List => name.trim().parse::<usize>().is_ok_and(|level| level > 0),
    };
    if !valid {
        bail!("{} entry name '{name}' is invalid.", setting.key());
    }
    Ok(match setting {
        CustomSetting::Checkbox => name.to_string(),
        CustomSetting::List => name.trim().parse::<usize>()?.to_string(),
        _ => name.trim().to_ascii_lowercase(),
    })
}

fn named_fields(setting: CustomSetting, name: &str, values: &str) -> Result<Fields> {
    let options = if setting == CustomSetting::CodeBlock {
        parse_code_block_options(values)?
    } else {
        parse_callout_options(values)?
    };
    let mut fields = Fields::new();
    for (key, value) in options {
        let key = key.trim().to_ascii_lowercase();
        let valid = if setting == CustomSetting::CodeBlock {
            matches!(key.as_str(), "icon" | "label" | "aliases")
        } else {
            matches!(key.as_str(), "icon" | "color")
        };
        if !valid {
            bail!("{} '{name}' has unknown option '{key}'.", setting.key());
        }
        let value = if value.trim().eq_ignore_ascii_case("null")
            || (key == "aliases" && value.trim() == "[]")
        {
            None
        } else {
            if key == "color" {
                parse_color_value(&value).with_context(|| {
                    format!("{} '{name}' has invalid color '{value}'.", setting.key())
                })?;
            }
            if matches!(setting, CustomSetting::Checkbox | CustomSetting::List)
                && key == "icon"
                && value.contains(':')
            {
                bail!("{} icon cannot contain ':'.", setting.key());
            }
            let value = if key == "aliases" {
                value
                    .split('|')
                    .map(|alias| alias.trim().to_ascii_lowercase())
                    .filter(|alias| !alias.is_empty())
                    .collect::<Vec<_>>()
                    .join("|")
            } else if setting == CustomSetting::CodeBlock {
                value
            } else {
                value.trim().to_string()
            };
            if setting == CustomSetting::List && key == "icon" && parse_color_value(&value).is_ok()
            {
                bail!("Custom list icon '{value}' is ambiguous with a color value.");
            }
            if key == "aliases" && value.is_empty() {
                None
            } else {
                Some(value)
            }
        };
        if fields.insert(key.clone(), value).is_some() {
            bail!("{} '{name}' repeats the {key} option.", setting.key());
        }
    }
    Ok(fields)
}

fn positional_fields(setting: CustomSetting, entry: &str, values: &str) -> Result<Fields> {
    let (icon, color) = if setting == CustomSetting::Checkbox {
        let Some((_, parsed)) = CheckboxOverride::parse_entry(entry)? else {
            return Ok(Fields::new());
        };
        (parsed.icon, parsed.color)
    } else {
        let (_, parsed) = ListMarkerOverride::parse_entry(entry)?;
        (parsed.icon, parsed.color)
    };
    let mut fields = Fields::new();
    if let Some(icon) = icon {
        fields.insert("icon".to_string(), Some(icon));
    }
    if color.is_some() {
        let (first, rest) = values.split_once(':').unwrap_or((values, values));
        let color = if setting == CustomSetting::List && parse_color_value(first.trim()).is_ok() {
            first
        } else {
            rest
        };
        fields.insert("color".to_string(), Some(color.trim().to_string()));
    }
    Ok(fields)
}
