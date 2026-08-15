use super::*;

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextWrapMode {
    #[value(help = "Character-level wrapping")]
    Char,
    #[value(help = "Wrap at word boundaries")]
    Word,
    #[value(help = "Disable wrapping")]
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableWrapMode {
    #[value(help = "Wrap text within table cells, fit to terminal width")]
    Fit,
    #[value(help = "Column wrapping: split table into blocks when too wide")]
    Wrap,
    #[value(help = "No wrapping: tables overflow horizontally")]
    None,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeadingLayout {
    #[value(help = "Level header indent, content indent = 1")]
    Level,
    #[value(help = "Center all headings, no content indentation")]
    Center,
    #[value(help = "No header indentation, content indent = 1")]
    Flat,
    #[value(help = "No indentation for headers and content")]
    None,
}
