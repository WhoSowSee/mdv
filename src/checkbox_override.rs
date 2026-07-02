//! Checkbox icon + optional color override for `--custom-checkbox`.

use crate::theme::{Color, parse_color_value};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CheckboxOverride {
    pub icon: Option<String>,
    pub color: Option<Color>,
}

impl CheckboxOverride {
    pub fn parse_entry(entry: &str) -> Result<Option<(char, Self)>> {
        let entry = entry.trim_end();
        if entry.is_empty() {
            return Ok(None);
        }

        let Some((key, rest)) = entry.split_once(':') else {
            return Ok(None);
        };
        let Some(ch) = key.chars().next() else {
            return Ok(None);
        };

        if let Some((icon_raw, color_raw)) = rest.split_once(':') {
            let icon = icon_raw.trim().to_string();
            let icon = if icon.is_empty() { None } else { Some(icon) };
            let color = parse_optional_color(color_raw)?;
            if icon.is_none() && color.is_none() {
                return Ok(None);
            }
            return Ok(Some((ch, Self { icon, color })));
        }

        let trimmed = rest.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        if let Ok(color) = parse_color_value(trimmed) {
            return Ok(Some((
                ch,
                Self {
                    icon: None,
                    color: Some(color),
                },
            )));
        }

        Ok(Some((
            ch,
            Self {
                icon: Some(trimmed.to_string()),
                color: None,
            },
        )))
    }
}

fn parse_optional_color(raw: &str) -> Result<Option<Color>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(parse_color_value(trimmed)?))
}
