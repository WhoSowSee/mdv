use super::overrides::{normalize_key, parse_override_pairs};
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ColorSource {
    Theme,
    Config,
    Preset,
    Cli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyntaxOverrideKind {
    Theme,
    Code,
}

#[derive(Debug, Clone, Copy)]
struct SyntaxPriority {
    source: ColorSource,
    kind: SyntaxOverrideKind,
}

#[derive(Debug, Clone)]
pub(crate) struct ColorPriorities {
    elements: HashMap<String, ColorSource>,
    syntax: HashMap<SyntaxField, SyntaxPriority>,
    pub(crate) code_theme: ColorSource,
    custom_colors: HashMap<String, HashMap<String, ColorSource>>,
}

impl Default for ColorPriorities {
    fn default() -> Self {
        Self {
            elements: HashMap::new(),
            syntax: HashMap::new(),
            code_theme: ColorSource::Config,
            custom_colors: HashMap::new(),
        }
    }
}

impl ColorPriorities {
    pub(crate) fn record(
        &mut self,
        custom_theme: Option<&str>,
        custom_code_theme: Option<&str>,
        source: ColorSource,
    ) -> Result<()> {
        if let Some(overrides) = custom_theme {
            for (key, _) in parse_override_pairs(overrides)? {
                let key = normalize_key(&key);
                if let Some(field) = key.strip_prefix("syntax:") {
                    self.syntax.insert(
                        SyntaxField::parse(field)?,
                        SyntaxPriority {
                            source,
                            kind: SyntaxOverrideKind::Theme,
                        },
                    );
                } else if matches!(
                    key.as_str(),
                    "callout:label"
                        | "list:ordered"
                        | "list:unordered"
                        | "todo:checked"
                        | "todo:unchecked"
                ) || key.starts_with("callout:palette:")
                {
                    self.elements.insert(key, source);
                }
            }
        }
        if let Some(overrides) = custom_code_theme {
            for (key, _) in parse_override_pairs(overrides)? {
                let key = normalize_key(&key);
                let field = key.strip_prefix("syntax:").unwrap_or(&key);
                self.syntax.insert(
                    SyntaxField::parse(field)?,
                    SyntaxPriority {
                        source,
                        kind: SyntaxOverrideKind::Code,
                    },
                );
            }
        }
        Ok(())
    }

    pub(crate) fn overlay(&mut self, other: &Self) {
        self.elements.extend(
            other
                .elements
                .iter()
                .map(|(key, &source)| (key.clone(), source)),
        );
        self.syntax.extend(
            other
                .syntax
                .iter()
                .map(|(&field, &priority)| (field, priority)),
        );
        self.code_theme = other.code_theme;
    }

    pub(crate) fn element(&self, path: &str) -> ColorSource {
        self.elements
            .get(path)
            .copied()
            .unwrap_or(ColorSource::Theme)
    }

    pub(crate) fn custom_color(&self, setting: &str, name: &str) -> ColorSource {
        self.custom_colors
            .get(setting)
            .and_then(|entries| entries.get(name))
            .copied()
            .unwrap_or(ColorSource::Config)
    }

    pub(crate) fn set_custom_color(
        &mut self,
        setting: &str,
        name: &str,
        source: Option<ColorSource>,
    ) {
        if let Some(source) = source {
            self.custom_colors
                .entry(setting.to_string())
                .or_default()
                .insert(name.to_string(), source);
        } else if let Some(entries) = self.custom_colors.get_mut(setting) {
            entries.remove(name);
        }
    }

    pub(crate) fn clear_custom_colors(&mut self, setting: &str) {
        self.custom_colors.remove(setting);
    }

    pub(crate) fn clear_theme(&mut self) {
        self.elements.clear();
        self.syntax
            .retain(|_, priority| priority.kind != SyntaxOverrideKind::Theme);
    }

    pub(crate) fn clear_code(&mut self) {
        self.syntax
            .retain(|_, priority| priority.kind != SyntaxOverrideKind::Code);
    }

    pub(crate) fn syntax_above_code_theme(&self) -> Vec<SyntaxField> {
        SyntaxField::ALL
            .into_iter()
            .filter(|field| {
                self.syntax.get(field).is_some_and(|priority| {
                    priority.source > self.code_theme
                        || (priority.source == self.code_theme
                            && priority.kind == SyntaxOverrideKind::Code)
                })
            })
            .collect()
    }
}
