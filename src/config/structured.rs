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

mod custom;
pub(super) use custom::{
    deserialize_custom_callout, deserialize_custom_checkbox, deserialize_custom_code_block,
    deserialize_custom_list,
};

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

    let mut assignments = Vec::new();
    for (key, value) in entries {
        append_theme_assignment(&key, value, &mut assignments)?;
    }
    Ok(Some(assignments.join(";")))
}

fn append_theme_assignment(
    key: &str,
    value: Value,
    output: &mut Vec<String>,
) -> Result<(), String> {
    if key.trim().is_empty() || key.split(':').any(|part| part.trim().is_empty()) {
        return Err("Theme override key cannot be empty.".to_string());
    }
    reject_delimiters(key, ['=', ';', '\n'], "Theme override key")?;
    if let Value::Mapping(entries) = value {
        if entries.is_empty() {
            return Err(format!(
                "Theme override section '{key}' must contain at least one field."
            ));
        }
        if key.split(':').count() >= 3 {
            return Err(format!(
                "Theme override '{key}' exceeds the supported nesting depth."
            ));
        }
        for (child, value) in entries {
            let child = child
                .as_str()
                .ok_or_else(|| "Theme override keys must be strings.".to_string())?;
            append_theme_assignment(&format!("{key}:{child}"), value, output)?;
        }
    } else {
        let value = scalar_to_string(value, &format!("Theme override '{key}'"))?;
        reject_delimiters(&value, [';', '\n'], &format!("Theme override '{key}'"))?;
        output.push(format!("{key}={value}"));
    }
    Ok(())
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
