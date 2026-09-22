use super::*;

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkStyle {
    /// Make link text clickable without showing the URL (alias: c)
    #[value(alias = "c")]
    #[serde(alias = "c")]
    Clickable,
    /// Underline clickable link text (alias: fc)
    #[value(name = "fclickable", alias = "fc")]
    #[serde(alias = "fclickable", alias = "fc")]
    ClickableForced,
    /// Show the URL after the link text (alias: i)
    #[value(alias = "i")]
    #[serde(alias = "i")]
    Inline,
    /// Number links and show a URL table after the text (alias: it)
    #[value(name = "inlinetable", alias = "it")]
    #[serde(alias = "inlinetable", alias = "it")]
    InlineTable,
    /// Number links and show a URL table at the document end (alias: et)
    #[value(name = "endtable", alias = "et")]
    #[serde(alias = "endtable", alias = "et")]
    EndTable,
    /// Hide link URLs (alias: h)
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
    /// Allow links to overflow horizontally
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
