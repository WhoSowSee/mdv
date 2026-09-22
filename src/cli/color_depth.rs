use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize, de};

/// Requested limit for terminal colors.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, ValueEnum)]
pub enum ColorDepth {
    #[default]
    #[serde(rename = "auto")]
    #[value(name = "auto", help = "Detect the terminal color depth")]
    Auto,
    #[serde(rename = "16")]
    #[value(name = "16", help = "Use the 16 standard and bright ANSI colors")]
    Ansi16,
    #[serde(rename = "256")]
    #[value(name = "256", help = "Use the 256-color ANSI palette")]
    Ansi256,
    #[serde(rename = "truecolor")]
    #[value(name = "truecolor", help = "Use 24-bit RGB colors")]
    TrueColor,
}

impl<'de> Deserialize<'de> for ColorDepth {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            Text(String),
            Number(u16),
        }
        let value = match Value::deserialize(deserializer)? {
            Value::Text(text) => text,
            Value::Number(number) => number.to_string(),
        };
        Self::from_str(&value, false)
            .map_err(|_| de::Error::custom("color_depth must be auto, 16, 256, or truecolor"))
    }
}
