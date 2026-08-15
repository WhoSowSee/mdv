use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeBlockStyle {
    #[value(help = "Indented code block without a border")]
    Basic,
    #[value(help = "Classic terminal gutter with single left border")]
    Simple,
    #[value(help = "Box-drawn frame around code blocks")]
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CodeBlockStyleConfig {
    pub style: CodeBlockStyle,
    pub show_name: bool,
    pub show_icon: bool,
}

impl Default for CodeBlockStyleConfig {
    fn default() -> Self {
        Self {
            style: CodeBlockStyle::Basic,
            show_name: false,
            show_icon: false,
        }
    }
}

impl CodeBlockStyleConfig {
    fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Code block style cannot be empty.".to_string());
        }

        let (style_raw, options_raw) = match input.split_once(':') {
            Some((style, options)) => (style.trim(), Some(options.trim())),
            None => (input, None),
        };

        let style = match style_raw.to_ascii_lowercase().as_str() {
            "basic" => CodeBlockStyle::Basic,
            "simple" => CodeBlockStyle::Simple,
            "pretty" => CodeBlockStyle::Pretty,
            _ => {
                return Err(format!(
                    "Unknown code block style '{}'. Expected 'basic', 'simple', or 'pretty'.",
                    style_raw
                ));
            }
        };

        let mut config = Self {
            style,
            ..Self::default()
        };

        if let Some(options_raw) = options_raw {
            if options_raw.is_empty() {
                return Err("Code block style options cannot be empty.".to_string());
            }

            for option in options_raw.split(';') {
                let option = option.trim();
                if option.is_empty() {
                    return Err("Code block style option cannot be empty.".to_string());
                }

                match option.to_ascii_lowercase().as_str() {
                    "show-name" => config.show_name = true,
                    "show-icon" => config.show_icon = true,
                    _ => return Err(format!("Unknown code block style option '{}'.", option)),
                }
            }
        }

        Ok(config)
    }
}

impl fmt::Display for CodeBlockStyleConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = match self.style {
            CodeBlockStyle::Basic => "basic",
            CodeBlockStyle::Simple => "simple",
            CodeBlockStyle::Pretty => "pretty",
        };

        let mut options = Vec::new();
        if self.show_name {
            options.push("show-name");
        }
        if self.show_icon {
            options.push("show-icon");
        }

        if options.is_empty() {
            write!(f, "{}", style)
        } else {
            write!(f, "{}:{}", style, options.join(";"))
        }
    }
}

impl TryFrom<String> for CodeBlockStyleConfig {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CodeBlockStyleConfig::parse(&value)
    }
}

impl From<CodeBlockStyleConfig> for String {
    fn from(value: CodeBlockStyleConfig) -> Self {
        value.to_string()
    }
}

pub(super) fn parse_code_block_style_config(value: &str) -> Result<CodeBlockStyleConfig, String> {
    CodeBlockStyleConfig::parse(value)
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeWrapIndent {
    #[value(help = "Do not add extra indentation to wrapped lines")]
    None,
    #[value(help = "Align wrapped lines to the original indentation")]
    Base,
    #[value(help = "Add two extra spaces on top of the original indentation")]
    Double,
}
