use super::*;

/// Theme configuration for markdown rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub pager_status_bar_transparent: bool,

    // Text colors
    pub text: Color,
    pub text_light: Color,
    #[serde(default = "default_line_number_color")]
    pub line_number: Color,
    #[serde(default = "default_line_number_color")]
    pub line_number_separator: Color,

    // Header colors (H1-H6)
    pub h1: Color,
    pub h2: Color,
    pub h3: Color,
    pub h4: Color,
    pub h5: Color,
    pub h6: Color,

    // Special elements
    pub code: Color,
    pub quote: Color,
    pub link: Color,
    pub emphasis: Color,
    pub strong: Color,
    #[serde(default)]
    pub strong_emphasis: Option<Color>,
    pub strikethrough: Color,
    #[serde(default)]
    pub highlight: Option<Color>,

    // Background and borders
    pub highlight_background: Color,
    #[serde(default)]
    pub emphasis_background: Option<Color>,
    #[serde(default)]
    pub strong_background: Option<Color>,
    #[serde(default)]
    pub strong_emphasis_background: Option<Color>,
    #[serde(default)]
    pub code_background: Option<Color>,
    #[serde(default)]
    pub strikethrough_background: Option<Color>,
    pub background: Option<Color>,
    pub border: Color,

    #[serde(default)]
    pub inline_style: InlineStyleSet,

    // List and table elements
    pub list_marker: Color,
    pub table_header: Color,
    pub table_border: Color,

    // Error and warning
    pub error: Color,
    pub warning: Color,

    // Code syntax highlighting colors
    pub syntax: SyntaxTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub number: Color,
    pub operator: Color,
    pub function: Color,
    pub variable: Color,
    pub type_name: Color,
}

impl Default for Theme {
    fn default() -> Self {
        BUILTIN_THEMES
            .get("terminal")
            .expect("embedded terminal theme must exist")
            .clone()
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Theme::default().syntax
    }
}

impl Theme {
    pub(crate) fn inline_foreground(&self, kind: InlineStyleKind) -> Option<&Color> {
        match kind {
            InlineStyleKind::Emphasis => Some(&self.emphasis),
            InlineStyleKind::Strong => Some(&self.strong),
            InlineStyleKind::StrongEmphasis => {
                Some(self.strong_emphasis.as_ref().unwrap_or(&self.strong))
            }
            InlineStyleKind::Code => Some(&self.code),
            InlineStyleKind::Strikethrough => Some(&self.strikethrough),
            InlineStyleKind::Highlight => self.highlight.as_ref(),
        }
    }

    pub(crate) fn inline_background(&self, kind: InlineStyleKind) -> Option<&Color> {
        match kind {
            InlineStyleKind::Emphasis => self.emphasis_background.as_ref(),
            InlineStyleKind::Strong => self.strong_background.as_ref(),
            InlineStyleKind::StrongEmphasis => self.strong_emphasis_background.as_ref(),
            InlineStyleKind::Code => self.code_background.as_ref(),
            InlineStyleKind::Strikethrough => self.strikethrough_background.as_ref(),
            InlineStyleKind::Highlight => Some(&self.highlight_background),
        }
    }
}

fn default_line_number_color() -> Color {
    Color::Grey
}
