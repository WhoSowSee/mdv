//! List marker icon + optional color override for `--custom-list` and `--pretty-list`.
//!
//! Maps a 1-based nesting level to an icon and an optional color. Falls back to
//! the built-in pretty-list set or to the default `"- "` marker when no override
//! is configured for a given level.

use crate::theme::{Color, parse_color_value};
use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::fmt;

const NERD_FONT_LARGE_ICONS: [&str; 4] = ["\u{f444}", "\u{f445}", "\u{f4c3}", "\u{f51d}"];
const NERD_FONT_SMALL_ICONS: [&str; 4] = ["\u{f09de}", "\u{f0a13}", "\u{f14dc}", "\u{f0a14}"];
const UNICODE_ICONS: [&str; 4] = ["⦁", "▪", "⚬", "▫"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrettyListType {
    NerdFont,
    Unicode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrettyListSize {
    Large,
    Small,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PrettyListStyle {
    marker_type: PrettyListType,
    size: PrettyListSize,
}

impl Default for PrettyListStyle {
    fn default() -> Self {
        Self {
            marker_type: PrettyListType::NerdFont,
            size: PrettyListSize::Large,
        }
    }
}

impl PrettyListStyle {
    pub fn parse(input: &str) -> std::result::Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("Pretty list style cannot be empty.".to_string());
        }

        let mut style = Self::default();
        let mut type_seen = false;
        let mut size_seen = false;

        for raw_option in input.split(';') {
            let option = raw_option.trim();
            if option.is_empty() {
                return Err("Pretty list style option cannot be empty.".to_string());
            }
            let (key, value) = option
                .split_once(':')
                .ok_or_else(|| format!("Pretty list option '{option}' must contain ':'."))?;
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim().to_ascii_lowercase();

            match key.as_str() {
                "type" => {
                    if type_seen {
                        return Err("Pretty list type is defined more than once.".to_string());
                    }
                    style.marker_type = match value.as_str() {
                        "nerd-font" => PrettyListType::NerdFont,
                        "unicode" => PrettyListType::Unicode,
                        _ => {
                            return Err(format!(
                                "Unknown pretty list type '{value}'. Expected 'nerd-font' or 'unicode'."
                            ));
                        }
                    };
                    type_seen = true;
                }
                "size" => {
                    if size_seen {
                        return Err("Pretty list size is defined more than once.".to_string());
                    }
                    style.size = match value.as_str() {
                        "large" => PrettyListSize::Large,
                        "small" => PrettyListSize::Small,
                        _ => {
                            return Err(format!(
                                "Unknown pretty list size '{value}'. Expected 'large' or 'small'."
                            ));
                        }
                    };
                    size_seen = true;
                }
                _ => return Err(format!("Unknown pretty list option '{key}'.")),
            }
        }

        Ok(style)
    }

    fn icon(self, level: usize) -> &'static str {
        let icons = match (self.marker_type, self.size) {
            (PrettyListType::NerdFont, PrettyListSize::Large) => &NERD_FONT_LARGE_ICONS,
            (PrettyListType::NerdFont, PrettyListSize::Small) => &NERD_FONT_SMALL_ICONS,
            (PrettyListType::Unicode, _) => &UNICODE_ICONS,
        };
        icons[level.saturating_sub(1).min(icons.len() - 1)]
    }
}

impl fmt::Display for PrettyListStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker_type = match self.marker_type {
            PrettyListType::NerdFont => "nerd-font",
            PrettyListType::Unicode => "unicode",
        };
        let size = match self.size {
            PrettyListSize::Large => "large",
            PrettyListSize::Small => "small",
        };
        write!(f, "type:{marker_type};size:{size}")
    }
}

impl TryFrom<String> for PrettyListStyle {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<PrettyListStyle> for String {
    fn from(value: PrettyListStyle) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum UniformListMarker {
    Level(usize),
    Icon(String),
}

impl UniformListMarker {
    pub fn parse(input: &str) -> std::result::Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("Uniform list marker cannot be empty.".to_string());
        }
        if input.contains(';') {
            return Err(
                "Uniform list marker accepts exactly one of 'level:<1-4>' or 'icon:<glyph>'."
                    .to_string(),
            );
        }

        let (key, value) = input
            .split_once(':')
            .ok_or_else(|| "Uniform list marker must contain ':'.".to_string())?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();

        match key.as_str() {
            "level" => {
                let level = value.parse::<usize>().map_err(|_| {
                    format!("Uniform list marker level '{value}' must be an integer from 1 to 4.")
                })?;
                if !(1..=4).contains(&level) {
                    return Err(format!(
                        "Uniform list marker level must be from 1 to 4 (got {level})."
                    ));
                }
                Ok(Self::Level(level))
            }
            "icon" => {
                if value.is_empty() {
                    return Err("Uniform list marker icon cannot be empty.".to_string());
                }
                Ok(Self::Icon(value.to_string()))
            }
            _ => Err(format!(
                "Unknown uniform list marker option '{key}'. Expected 'level' or 'icon'."
            )),
        }
    }
}

impl fmt::Display for UniformListMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Level(level) => write!(f, "level:{level}"),
            Self::Icon(icon) => write!(f, "icon:{icon}"),
        }
    }
}

impl TryFrom<String> for UniformListMarker {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<UniformListMarker> for String {
    fn from(value: UniformListMarker) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListMarkerOverride {
    pub icon: Option<String>,
    pub color: Option<Color>,
}

#[derive(Debug, Default, Clone)]
pub struct ListMarkerConfig {
    pub style: Option<PrettyListStyle>,
    pub uniform: Option<UniformListMarker>,
    pub overrides: HashMap<usize, ListMarkerOverride>,
}

impl ListMarkerConfig {
    /// Resolve the marker for the given 1-based nesting level.
    ///
    /// Returns `None` when no override is active and the default `"- "` should
    /// be used. The returned tuple is `(icon, color)` where `icon` falls back
    /// to the built-in pretty-list glyph when only a color override is set.
    pub fn resolve(&self, level: usize) -> Option<(String, Option<Color>)> {
        let style = self.style?;
        let override_entry = self.overrides.get(&level);
        let override_icon = override_entry.and_then(|e| e.icon.clone());
        let override_color = override_entry.and_then(|e| e.color.clone());
        let icon = override_icon.unwrap_or_else(|| match &self.uniform {
            Some(UniformListMarker::Level(uniform_level)) => style.icon(*uniform_level).to_string(),
            Some(UniformListMarker::Icon(icon)) => icon.clone(),
            None => style.icon(level).to_string(),
        });
        Some((icon, override_color))
    }

    /// Parse the `--custom-list` string into a list of `(level, override)` pairs.
    /// Returns the parsed map. Multiple entries for the same level are rejected.
    pub fn parse_custom_list(input: &str) -> Result<HashMap<usize, ListMarkerOverride>> {
        let mut out = HashMap::new();
        let mut has_entries = false;

        for raw_entry in input.split(';') {
            let entry = raw_entry.trim();
            if entry.is_empty() {
                continue;
            }
            has_entries = true;

            let (level_raw, rest) = entry
                .split_once(':')
                .with_context(|| format!("Custom list entry '{entry}' must contain ':'"))?;

            let level: usize = level_raw.trim().parse().with_context(|| {
                format!("Custom list level '{level_raw}' must be a positive integer")
            })?;
            if level == 0 {
                bail!("Custom list level must be 1 or greater (got 0).");
            }

            let rest = rest.trim();
            if rest.is_empty() {
                bail!("Custom list level {level} must define an icon or color.");
            }

            // `<icon>[:<color>]` branch, or `<color>` alone when the first
            // token happens to parse as a color. The first split picks up the
            // optional second segment without consuming extra ':' inside.
            let (first, remainder) = match rest.split_once(':') {
                Some(parts) => parts,
                None => (rest, ""),
            };
            let first_trim = first.trim();

            let (icon, color) = if let Ok(parsed) = parse_color_value(first_trim) {
                if !remainder.trim().is_empty() {
                    bail!(
                        "Custom list level {level} color-only entry must not contain extra tokens."
                    );
                }
                (None, Some(parsed))
            } else {
                let color = if remainder.trim().is_empty() {
                    None
                } else {
                    let trimmed = remainder.trim();
                    Some(parse_color_value(trimmed).with_context(|| {
                        format!("Custom list level {level} has invalid color '{trimmed}'")
                    })?)
                };
                (Some(first_trim.to_string()), color)
            };

            if out
                .insert(level, ListMarkerOverride { icon, color })
                .is_some()
            {
                bail!("Custom list level {level} is defined more than once.");
            }
        }

        if !has_entries {
            bail!("Custom list string is empty.");
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests;
