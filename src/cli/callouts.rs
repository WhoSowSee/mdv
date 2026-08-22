use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalloutStyle {
    #[value(help = "Callout label with the quote gutter")]
    Simple,
    #[value(help = "Box-drawn frame with callout label on top")]
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckboxShape {
    #[value(help = "Square Nerd Font icons for task-list checkboxes")]
    Square,
    #[value(help = "Circular Nerd Font icons for task-list checkboxes")]
    Circle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrettyDefinitionStyle {
    #[value(help = "Unicode heavy arrow")]
    Unicode,
    #[value(help = "Nerd Font icon U+F0315")]
    NerdFont,
}

impl PrettyDefinitionStyle {
    pub(crate) fn marker(self) -> &'static str {
        match self {
            Self::Unicode => "🠶 ",
            Self::NerdFont => "\u{f0315} ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(into = "String")]
pub struct CalloutStyleConfig {
    pub style: CalloutStyle,
    pub show_icons: bool,
    pub show_simple_icons: bool,
    pub show_fold_icons: bool,
    pub label_inside: bool,
    pub uppercase: bool,
}

impl Default for CalloutStyleConfig {
    fn default() -> Self {
        Self {
            style: CalloutStyle::Pretty,
            show_icons: false,
            show_simple_icons: false,
            show_fold_icons: false,
            label_inside: false,
            uppercase: false,
        }
    }
}

impl CalloutStyleConfig {
    fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Callout style cannot be empty.".to_string());
        }

        let (style_raw, options_raw) = match input.split_once(':') {
            Some((style, options)) => (style.trim(), Some(options.trim())),
            None => (input, None),
        };

        let style = match style_raw.to_ascii_lowercase().as_str() {
            "simple" => CalloutStyle::Simple,
            "pretty" => CalloutStyle::Pretty,
            _ => {
                return Err(format!(
                    "Unknown callout style '{}'. Expected 'simple' or 'pretty'.",
                    style_raw
                ));
            }
        };

        let mut config = CalloutStyleConfig {
            style,
            ..CalloutStyleConfig::default()
        };

        if let Some(options_raw) = options_raw {
            if options_raw.is_empty() {
                return Err("Callout style options cannot be empty.".to_string());
            }

            for option in options_raw.split(';') {
                let option = option.trim();
                if option.is_empty() {
                    return Err("Callout style option cannot be empty.".to_string());
                }

                match option.to_ascii_lowercase().as_str() {
                    "show-icons" => config.show_icons = true,
                    "show-simple-icons" => config.show_simple_icons = true,
                    "fold-icons" => config.show_fold_icons = true,
                    "label-inside" => config.label_inside = true,
                    "uppercase" => config.uppercase = true,
                    _ => return Err(format!("Unknown callout style option '{}'.", option)),
                }
            }
        }

        config.validate()
    }

    fn validate(self) -> Result<Self, String> {
        if matches!(self.style, CalloutStyle::Simple) && self.label_inside {
            return Err(
                "Option 'label-inside' is only supported with 'pretty' callout style.".to_string(),
            );
        }

        if self.show_icons && self.show_simple_icons {
            return Err(
                "Options 'show-icons' and 'show-simple-icons' cannot be combined.".to_string(),
            );
        }

        Ok(self)
    }

    pub(crate) fn icons_enabled(&self) -> bool {
        self.show_icons || self.show_simple_icons
    }
}

impl fmt::Display for CalloutStyleConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = match self.style {
            CalloutStyle::Simple => "simple",
            CalloutStyle::Pretty => "pretty",
        };

        let mut options = Vec::new();
        if self.show_icons {
            options.push("show-icons");
        }
        if self.show_simple_icons {
            options.push("show-simple-icons");
        }
        if self.show_fold_icons {
            options.push("fold-icons");
        }
        if self.label_inside {
            options.push("label-inside");
        }
        if self.uppercase {
            options.push("uppercase");
        }

        if options.is_empty() {
            write!(f, "{}", style)
        } else {
            write!(f, "{}:{}", style, options.join(";"))
        }
    }
}

impl TryFrom<String> for CalloutStyleConfig {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CalloutStyleConfig::parse(&value)
    }
}

impl From<CalloutStyleConfig> for String {
    fn from(value: CalloutStyleConfig) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for CalloutStyleConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Mapping {
            style: CalloutStyle,
            #[serde(default)]
            show_icons: bool,
            #[serde(default)]
            show_simple_icons: bool,
            #[serde(default)]
            fold_icons: bool,
            #[serde(default)]
            label_inside: bool,
            #[serde(default)]
            uppercase: bool,
        }

        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Input {
            String(String),
            Mapping(Mapping),
        }

        let config = match Input::deserialize(deserializer)? {
            Input::String(value) => return Self::parse(&value).map_err(serde::de::Error::custom),
            Input::Mapping(value) => Self {
                style: value.style,
                show_icons: value.show_icons,
                show_simple_icons: value.show_simple_icons,
                show_fold_icons: value.fold_icons,
                label_inside: value.label_inside,
                uppercase: value.uppercase,
            },
        };
        config.validate().map_err(serde::de::Error::custom)
    }
}

pub(super) fn parse_callout_style_config(value: &str) -> Result<CalloutStyleConfig, String> {
    CalloutStyleConfig::parse(value)
}
