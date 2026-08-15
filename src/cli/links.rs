use super::*;

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkStyle {
    /// [alias:  c] Link text becomes clickable without showing URL
    #[value(alias = "c")]
    #[serde(alias = "c")]
    Clickable,
    /// [alias: fc] Clickable links with forced underline
    #[value(name = "fclickable", alias = "fc")]
    #[serde(alias = "fclickable", alias = "fc")]
    ClickableForced,
    /// [alias:  i] Link URL after link name
    #[value(alias = "i")]
    #[serde(alias = "i")]
    Inline,
    /// [alias: it] Index after link name and link URL table after text
    #[value(name = "inlinetable", alias = "it")]
    #[serde(alias = "inlinetable", alias = "it")]
    InlineTable,
    /// [alias: et] Index after link name and link URL table at document end
    #[value(name = "endtable", alias = "et")]
    #[serde(alias = "endtable", alias = "et")]
    EndTable,
    /// [alias:  h] Hide link URLs
    #[value(alias = "h")]
    #[serde(alias = "h")]
    Hide,
}

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkTruncationStyle {
    /// Wrap links when they don't fit
    Wrap,
    /// Cut links and replace with "..." when they don't fit
    Cut,
    /// Cut links in normal flow and inside table cells
    #[value(name = "tablecut")]
    #[serde(rename = "tablecut")]
    TableCut,
    /// No truncation - links overflow horizontally
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FootnoteStyle {
    #[value(help = "Collect all footnotes at the end of the document")]
    Endnotes,
    #[value(help = "Render footnotes immediately after the block that references them")]
    Attached,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingFootnoteStyle {
    #[value(help = "Render missing footnotes with a placeholder entry")]
    Show,
    #[value(help = "Omit missing footnotes from the footnote block")]
    Hide,
}
