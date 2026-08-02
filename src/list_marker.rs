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
mod tests {
    use super::*;
    use crate::theme::Color;

    #[test]
    fn pretty_list_uses_large_nerd_font_icons_by_default() {
        let cfg = ListMarkerConfig {
            style: Some(PrettyListStyle::default()),
            ..Default::default()
        };

        let (icon1, _) = cfg.resolve(1).unwrap();
        let (icon2, _) = cfg.resolve(2).unwrap();
        let (icon3, _) = cfg.resolve(3).unwrap();
        let (icon4, _) = cfg.resolve(4).unwrap();
        let (icon10, _) = cfg.resolve(10).unwrap();

        assert_eq!(icon1, "\u{f444}");
        assert_eq!(icon2, "\u{f445}");
        assert_eq!(icon3, "\u{f4c3}");
        assert_eq!(icon4, "\u{f51d}");
        assert_eq!(icon10, "\u{f51d}");
    }

    #[test]
    fn pretty_list_uses_small_nerd_font_icons() {
        let cfg = ListMarkerConfig {
            style: Some(PrettyListStyle::parse("type:nerd-font;size:small").unwrap()),
            ..Default::default()
        };

        let icons = (1..=5)
            .map(|level| cfg.resolve(level).unwrap().0)
            .collect::<Vec<_>>();

        assert_eq!(
            icons,
            [
                "\u{f09de}",
                "\u{f0a13}",
                "\u{f14dc}",
                "\u{f0a14}",
                "\u{f0a14}"
            ]
        );
    }

    #[test]
    fn pretty_list_style_defaults_omitted_fields() {
        assert_eq!(
            PrettyListStyle::parse("size:small").unwrap(),
            PrettyListStyle {
                marker_type: PrettyListType::NerdFont,
                size: PrettyListSize::Small,
            }
        );
        assert_eq!(
            PrettyListStyle::parse("type:unicode").unwrap(),
            PrettyListStyle {
                marker_type: PrettyListType::Unicode,
                size: PrettyListSize::Large,
            }
        );
        assert!(PrettyListStyle::parse("type:ascii").is_err());
    }

    #[test]
    fn unicode_icons_ignore_size() {
        let large = ListMarkerConfig {
            style: Some(PrettyListStyle::parse("type:unicode;size:large").unwrap()),
            ..Default::default()
        };
        let small = ListMarkerConfig {
            style: Some(PrettyListStyle::parse("type:unicode;size:small").unwrap()),
            ..Default::default()
        };

        for (level, expected) in [(1, "⦁"), (2, "▪"), (3, "⚬"), (4, "▫"), (8, "▫")] {
            assert_eq!(large.resolve(level).unwrap().0, expected);
            assert_eq!(small.resolve(level).unwrap().0, expected);
        }
    }

    #[test]
    fn uniform_custom_icon_is_used_for_every_level() {
        let cfg = ListMarkerConfig {
            style: Some(PrettyListStyle::default()),
            uniform: Some(UniformListMarker::Icon("*".to_string())),
            ..Default::default()
        };

        for level in 1..=6 {
            assert_eq!(cfg.resolve(level).unwrap().0, "*");
        }
    }

    #[test]
    fn custom_list_overrides_per_level() {
        let cfg = ListMarkerConfig {
            style: Some(PrettyListStyle::default()),
            overrides: ListMarkerConfig::parse_custom_list("5:&").unwrap(),
            ..Default::default()
        };

        let (icon, color) = cfg.resolve(5).unwrap();
        assert_eq!(icon, "&");
        assert_eq!(color, None);
    }

    #[test]
    fn custom_list_parses_color() {
        let overrides = ListMarkerConfig::parse_custom_list("5:&:#ff0000").unwrap();
        let entry = overrides.get(&5).unwrap();
        assert_eq!(entry.icon, Some("&".to_string()));
        assert_eq!(
            entry.color,
            Some(Color::Rgb {
                r: 0xff,
                g: 0,
                b: 0
            })
        );
    }

    #[test]
    fn custom_list_rejects_duplicate_levels() {
        assert!(ListMarkerConfig::parse_custom_list("1:a;1:b").is_err());
    }

    #[test]
    fn custom_list_rejects_zero_level() {
        assert!(ListMarkerConfig::parse_custom_list("0:a").is_err());
    }

    #[test]
    fn custom_list_rejects_empty_value() {
        assert!(ListMarkerConfig::parse_custom_list("1:").is_err());
    }

    #[test]
    fn inactive_config_returns_none() {
        let cfg = ListMarkerConfig::default();
        assert!(cfg.resolve(1).is_none());
    }

    #[test]
    fn custom_list_color_only_parses() {
        let overrides = ListMarkerConfig::parse_custom_list("1:red;2:#00ff00").unwrap();
        assert_eq!(overrides.get(&1).unwrap().icon, None);
        assert_eq!(overrides.get(&1).unwrap().color, Some(Color::Red));
        assert_eq!(overrides.get(&2).unwrap().icon, None);
        assert_eq!(
            overrides.get(&2).unwrap().color,
            Some(Color::Rgb {
                r: 0,
                g: 0xff,
                b: 0
            })
        );
    }

    #[test]
    fn color_only_falls_back_to_pretty_icon() {
        let cfg = ListMarkerConfig {
            style: Some(PrettyListStyle::default()),
            overrides: ListMarkerConfig::parse_custom_list("1:red").unwrap(),
            ..Default::default()
        };
        let (icon, color) = cfg.resolve(1).unwrap();
        assert_eq!(icon, "\u{f444}");
        assert_eq!(color, Some(Color::Red));
    }

    #[test]
    fn color_only_rejects_extra_tokens() {
        assert!(ListMarkerConfig::parse_custom_list("1:red:extra").is_err());
    }

    #[test]
    fn custom_icon_overrides_uniform_marker() {
        let cfg = ListMarkerConfig {
            style: Some(PrettyListStyle::default()),
            uniform: Some(UniformListMarker::Level(2)),
            overrides: ListMarkerConfig::parse_custom_list("3:>").unwrap(),
        };

        assert_eq!(cfg.resolve(2).unwrap().0, "\u{f445}");
        assert_eq!(cfg.resolve(3).unwrap().0, ">");
    }

    #[test]
    fn uniform_marker_parser_enforces_exclusive_valid_value() {
        assert_eq!(
            UniformListMarker::parse("level:4").unwrap(),
            UniformListMarker::Level(4)
        );
        assert_eq!(
            UniformListMarker::parse("icon:◆").unwrap(),
            UniformListMarker::Icon("◆".to_string())
        );
        assert!(UniformListMarker::parse("level:0").is_err());
        assert!(UniformListMarker::parse("level:5").is_err());
        assert!(UniformListMarker::parse("level:1;icon:◆").is_err());
    }
}
