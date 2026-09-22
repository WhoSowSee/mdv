use super::*;

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextWrapMode {
    #[value(help = "Wrap at character boundaries")]
    Char,
    #[value(help = "Wrap at word boundaries")]
    Word,
    #[value(help = "Avoid soft wrapping; constrained table cells may hard-wrap")]
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableWrapMode {
    #[value(help = "Fit table cells to terminal width")]
    Fit,
    #[value(help = "Split wide tables into blocks of columns")]
    Wrap,
    #[value(help = "Allow tables to overflow horizontally without cell wrapping")]
    None,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
/// Visual container used for display and fenced math blocks.
pub enum MathBlockStyle {
    #[default]
    #[value(help = "Indent math without a border")]
    Basic,
    #[value(help = "Show a single left border")]
    Simple,
    #[value(help = "Show a frame around the block")]
    Pretty,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeadingLayout {
    #[value(help = "Indent headings by level and content one column further")]
    Level,
    #[value(help = "Center all headings, no content indentation")]
    Center,
    #[value(help = "Align headings to the left and indent content by one column")]
    Flat,
    #[value(help = "Align headings and content to the left")]
    None,
}
