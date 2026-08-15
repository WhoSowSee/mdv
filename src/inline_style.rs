use crate::terminal::AnsiStyle;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineStyleKind {
    Emphasis,
    Strong,
    StrongEmphasis,
    Code,
    Strikethrough,
    Highlight,
}

impl InlineStyleKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Emphasis => "emphasis",
            Self::Strong => "strong",
            Self::StrongEmphasis => "strong_emphasis",
            Self::Code => "code",
            Self::Strikethrough => "strikethrough",
            Self::Highlight => "highlight",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match normalize_name(value).as_str() {
            "emphasis" => Ok(Self::Emphasis),
            "strong" => Ok(Self::Strong),
            "strong_emphasis" => Ok(Self::StrongEmphasis),
            "code" => Ok(Self::Code),
            "strikethrough" => Ok(Self::Strikethrough),
            "highlight" => Ok(Self::Highlight),
            _ => Err(format!("Unknown inline style element '{}'.", value.trim())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InlineStyle {
    pub backticks: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl InlineStyle {
    pub(crate) const fn plain() -> Self {
        Self {
            backticks: false,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }

    pub(crate) fn merge_attributes(&mut self, other: Self) {
        self.bold |= other.bold;
        self.italic |= other.italic;
        self.underline |= other.underline;
        self.strikethrough |= other.strikethrough;
    }

    pub(crate) fn apply_attributes(self, mut style: AnsiStyle) -> AnsiStyle {
        if self.bold {
            style = style.bold();
        }
        if self.italic {
            style = style.italic();
        }
        if self.underline {
            style = style.underline();
        }
        if self.strikethrough {
            style = style.strikethrough();
        }
        style
    }
}

impl Default for InlineStyle {
    fn default() -> Self {
        Self::plain()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InlineStyleOverride {
    pub backticks: Option<bool>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
}

impl InlineStyleOverride {
    fn apply_to(self, style: &mut InlineStyle) {
        if let Some(value) = self.backticks {
            style.backticks = value;
        }
        if let Some(value) = self.bold {
            style.bold = value;
        }
        if let Some(value) = self.italic {
            style.italic = value;
        }
        if let Some(value) = self.underline {
            style.underline = value;
        }
        if let Some(value) = self.strikethrough {
            style.strikethrough = value;
        }
    }

    fn merge(&mut self, other: &Self) {
        if other.backticks.is_some() {
            self.backticks = other.backticks;
        }
        if other.bold.is_some() {
            self.bold = other.bold;
        }
        if other.italic.is_some() {
            self.italic = other.italic;
        }
        if other.underline.is_some() {
            self.underline = other.underline;
        }
        if other.strikethrough.is_some() {
            self.strikethrough = other.strikethrough;
        }
    }

    fn set(&mut self, property: &str, value: bool) -> Result<bool, String> {
        let normalized = normalize_name(property);
        let target = match normalized.as_str() {
            "backticks" => &mut self.backticks,
            "bold" => &mut self.bold,
            "italic" => &mut self.italic,
            "underline" => &mut self.underline,
            "strikethrough" => &mut self.strikethrough,
            _ => {
                return Err(format!(
                    "Unknown inline style property '{}'.",
                    property.trim()
                ));
            }
        };
        Ok(target.replace(value).is_some())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InlineStyleOverrides {
    pub emphasis: InlineStyleOverride,
    pub strong: InlineStyleOverride,
    pub strong_emphasis: InlineStyleOverride,
    pub code: InlineStyleOverride,
    pub strikethrough: InlineStyleOverride,
    pub highlight: InlineStyleOverride,
}

impl InlineStyleOverrides {
    pub(crate) fn merge(&mut self, other: &Self) {
        self.emphasis.merge(&other.emphasis);
        self.strong.merge(&other.strong);
        self.strong_emphasis.merge(&other.strong_emphasis);
        self.code.merge(&other.code);
        self.strikethrough.merge(&other.strikethrough);
        self.highlight.merge(&other.highlight);
    }

    fn get_mut(&mut self, kind: InlineStyleKind) -> &mut InlineStyleOverride {
        match kind {
            InlineStyleKind::Emphasis => &mut self.emphasis,
            InlineStyleKind::Strong => &mut self.strong,
            InlineStyleKind::StrongEmphasis => &mut self.strong_emphasis,
            InlineStyleKind::Code => &mut self.code,
            InlineStyleKind::Strikethrough => &mut self.strikethrough,
            InlineStyleKind::Highlight => &mut self.highlight,
        }
    }
}

impl FromStr for InlineStyleOverrides {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Inline style cannot be empty.".to_string());
        }

        let mut overrides = Self::default();
        for entry in input.split(';') {
            let entry = entry.trim();
            if entry.is_empty() {
                return Err("Inline style entry cannot be empty.".to_string());
            }
            let (element, properties) = entry
                .split_once(':')
                .ok_or_else(|| format!("Inline style entry '{}' must contain ':'.", entry))?;
            let kind = InlineStyleKind::parse(element)?;
            if properties.trim().is_empty() {
                return Err(format!(
                    "Inline style element '{}' must contain at least one property.",
                    kind.as_str()
                ));
            }

            for assignment in properties.split(',') {
                let assignment = assignment.trim();
                let (property, value) = assignment.split_once('=').ok_or_else(|| {
                    format!("Inline style property '{}' must contain '='.", assignment)
                })?;
                let value = parse_bool(value)?;
                if overrides.get_mut(kind).set(property, value)? {
                    return Err(format!(
                        "Inline style property '{}' is specified more than once for '{}'.",
                        property.trim(),
                        kind.as_str()
                    ));
                }
            }
        }

        Ok(overrides)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InlineStyleSet {
    pub emphasis: InlineStyle,
    pub strong: InlineStyle,
    pub strong_emphasis: InlineStyle,
    pub code: InlineStyle,
    pub strikethrough: InlineStyle,
    pub highlight: InlineStyle,
}

impl InlineStyleSet {
    pub(crate) fn apply_overrides(&mut self, overrides: &InlineStyleOverrides) {
        overrides.emphasis.apply_to(&mut self.emphasis);
        overrides.strong.apply_to(&mut self.strong);
        overrides
            .strong_emphasis
            .apply_to(&mut self.strong_emphasis);
        overrides.code.apply_to(&mut self.code);
        overrides.strikethrough.apply_to(&mut self.strikethrough);
        overrides.highlight.apply_to(&mut self.highlight);
    }

    pub(crate) const fn get(&self, kind: InlineStyleKind) -> InlineStyle {
        match kind {
            InlineStyleKind::Emphasis => self.emphasis,
            InlineStyleKind::Strong => self.strong,
            InlineStyleKind::StrongEmphasis => self.strong_emphasis,
            InlineStyleKind::Code => self.code,
            InlineStyleKind::Strikethrough => self.strikethrough,
            InlineStyleKind::Highlight => self.highlight,
        }
    }
}

impl Default for InlineStyleSet {
    fn default() -> Self {
        Self {
            emphasis: InlineStyle {
                italic: true,
                ..InlineStyle::plain()
            },
            strong: InlineStyle {
                bold: true,
                ..InlineStyle::plain()
            },
            strong_emphasis: InlineStyle {
                bold: true,
                italic: true,
                ..InlineStyle::plain()
            },
            code: InlineStyle {
                backticks: true,
                ..InlineStyle::plain()
            },
            strikethrough: InlineStyle {
                strikethrough: true,
                ..InlineStyle::plain()
            },
            highlight: InlineStyle::plain(),
        }
    }
}

impl<'de> Deserialize<'de> for InlineStyleSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let overrides = InlineStyleOverrides::deserialize(deserializer)?;
        let mut styles = Self::default();
        styles.apply_overrides(&overrides);
        Ok(styles)
    }
}

fn normalize_name(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_ascii_lowercase()
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!(
            "Inline style value '{}' must be 'true' or 'false'.",
            value.trim()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_duplicate_properties_across_entries() {
        let error = "code:bold=true;code:bold=false"
            .parse::<InlineStyleOverrides>()
            .unwrap_err();

        assert!(error.contains("specified more than once"));
    }

    #[test]
    fn partial_yaml_keeps_semantic_defaults() {
        let styles: InlineStyleSet =
            serde_yaml::from_str("emphasis:\n  underline: true\ncode:\n  backticks: false\n")
                .unwrap();

        assert!(styles.emphasis.italic);
        assert!(styles.emphasis.underline);
        assert!(!styles.code.backticks);
        assert!(styles.strong.bold);
    }
}
