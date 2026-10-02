use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerTheme {
    pub transparent: bool,
    pub title: Option<Color>,
    pub help: Option<Color>,
    pub file_name: Option<Color>,
    pub progress: Option<Color>,
    pub matches: Option<Color>,
    pub selection: InlineTheme<Option<Color>>,
    pub search: InlineTheme<Option<Color>>,
    pub search_current: InlineTheme<Option<Color>>,
}

impl Default for PagerTheme {
    fn default() -> Self {
        Theme::default().pager
    }
}
