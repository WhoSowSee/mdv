//! List marker icon + optional color override for `--custom-list` and `--pretty-list`.
//!
//! Maps a 1-based nesting level to an icon and an optional color. Falls back to
//! the built-in pretty-list set or to the default `"- "` marker when no override
//! is configured for a given level.

use crate::theme::{Color, parse_color_value};
use anyhow::{Context, Result, bail};
use std::collections::HashMap;

/// Built-in Nerd Font icons used by `--pretty-list`. Index `level - 1` is
/// used for the marker at that nesting depth; entries past the end fall back
/// to the last glyph.
const PRETTY_LIST_ICONS: &[&str] = &[
    "\u{f444}", // level 1
    "\u{f445}", // level 2
    "\u{f4c3}", // level 3
    "\u{f51d}", // level 4+ (and any deeper nesting)
];

#[derive(Debug, Clone, PartialEq)]
pub struct ListMarkerOverride {
    pub icon: Option<String>,
    pub color: Option<Color>,
}

#[derive(Debug, Default, Clone)]
pub struct ListMarkerConfig {
    /// Built-in pretty list set enabled via `--pretty-list`.
    pub pretty: bool,
    /// User-defined overrides keyed by 1-based nesting level.
    pub overrides: HashMap<usize, ListMarkerOverride>,
}

impl ListMarkerConfig {
    /// Resolve the marker for the given 1-based nesting level.
    ///
    /// Returns `None` when no override is active and the default `"- "` should
    /// be used. The returned tuple is `(icon, color)` where `icon` falls back
    /// to the built-in pretty-list glyph when only a color override is set.
    pub fn resolve(&self, level: usize) -> Option<(String, Option<Color>)> {
        let override_entry = self.overrides.get(&level);
        let override_icon = override_entry.and_then(|e| e.icon.clone());
        let override_color = override_entry.and_then(|e| e.color.clone());
        if self.pretty {
            let icon = override_icon.unwrap_or_else(|| {
                let idx = level.saturating_sub(1).min(PRETTY_LIST_ICONS.len() - 1);
                PRETTY_LIST_ICONS[idx].to_string()
            });
            return Some((icon, override_color));
        }
        override_icon.map(|icon| (icon, override_color))
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
    fn pretty_list_uses_builtin_icons() {
        let cfg = ListMarkerConfig {
            pretty: true,
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
    fn custom_list_overrides_per_level() {
        let cfg = ListMarkerConfig {
            pretty: true,
            overrides: ListMarkerConfig::parse_custom_list("5:&").unwrap(),
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
            Some(Color::Rgb { r: 0, g: 0xff, b: 0 })
        );
    }

    #[test]
    fn color_only_falls_back_to_pretty_icon() {
        let cfg = ListMarkerConfig {
            pretty: true,
            overrides: ListMarkerConfig::parse_custom_list("1:red").unwrap(),
        };
        let (icon, color) = cfg.resolve(1).unwrap();
        assert_eq!(icon, "\u{f444}");
        assert_eq!(color, Some(Color::Red));
    }

    #[test]
    fn color_only_rejects_extra_tokens() {
        assert!(ListMarkerConfig::parse_custom_list("1:red:extra").is_err());
    }
}
