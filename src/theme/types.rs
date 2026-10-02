use super::*;

/// Theme configuration for markdown rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub pager: PagerTheme,

    // Text colors
    pub text: Color,
    pub text_light: Color,
    pub line_number: LineNumberTheme,

    // Header colors (H1-H6)
    pub h1: Color,
    pub h2: Color,
    pub h3: Color,
    pub h4: Color,
    pub h5: Color,
    pub h6: Color,

    // Special elements
    pub code: InlineTheme,
    pub math: MathTheme,
    pub quote: Color,
    pub link: Color,
    pub emphasis: InlineTheme,
    pub strong: InlineTheme,
    pub strong_emphasis: InlineTheme<Option<Color>>,
    pub strikethrough: InlineTheme,
    pub highlight: InlineTheme<Option<Color>, Color>,

    // Background and borders
    pub background: Option<Color>,
    pub code_block: CodeBlockTheme,
    pub callout: CalloutTheme,
    #[serde(default)]
    pub horizontal_rule: Option<Color>,
    #[serde(default)]
    pub footnote_separator: Option<Color>,
    pub front_matter: FrontMatterTheme,
    #[serde(default)]
    pub details_border: Option<Color>,

    #[serde(default)]
    pub inline_style: InlineStyleSet,

    // List and table elements
    pub list: ListTheme,
    pub table: TableTheme,
    pub todo: TodoTheme,

    // Error and warning
    pub error: Color,
    pub warning: Color,

    // Code syntax highlighting colors
    pub syntax: SyntaxTheme,
    #[serde(skip)]
    pub(crate) color_priorities: ColorPriorities,
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
            InlineStyleKind::Emphasis => Some(&self.emphasis.text),
            InlineStyleKind::Strong => Some(&self.strong.text),
            InlineStyleKind::StrongEmphasis => Some(
                self.strong_emphasis
                    .text
                    .as_ref()
                    .unwrap_or(&self.strong.text),
            ),
            InlineStyleKind::Code => Some(&self.code.text),
            InlineStyleKind::Strikethrough => Some(&self.strikethrough.text),
            InlineStyleKind::Highlight => self.highlight.text.as_ref(),
        }
    }

    pub(crate) fn inline_background(&self, kind: InlineStyleKind) -> Option<&Color> {
        match kind {
            InlineStyleKind::Emphasis => self.emphasis.background.as_ref(),
            InlineStyleKind::Strong => self.strong.background.as_ref(),
            InlineStyleKind::StrongEmphasis => self.strong_emphasis.background.as_ref(),
            InlineStyleKind::Code => self.code.background.as_ref(),
            InlineStyleKind::Strikethrough => self.strikethrough.background.as_ref(),
            InlineStyleKind::Highlight => Some(&self.highlight.background),
        }
    }
}
