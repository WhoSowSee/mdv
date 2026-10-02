use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PagerFile {
    pub transparent: Option<bool>,
    pub title: OptionalColorYaml,
    pub help: OptionalColorYaml,
    pub file_name: OptionalColorYaml,
    pub progress: OptionalColorYaml,
    pub matches: OptionalColorYaml,
    pub selection: Option<PagerHighlightFile>,
    pub search: Option<PagerHighlightFile>,
    pub search_current: Option<PagerHighlightFile>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PagerHighlightFile {
    pub text: OptionalColorYaml,
    pub background: OptionalColorYaml,
}

impl PagerHighlightFile {
    fn resolve(&self, base: &InlineTheme<Option<Color>>) -> InlineTheme<Option<Color>> {
        InlineTheme {
            text: self.text.resolve(&base.text),
            background: self.background.resolve(&base.background),
        }
    }

    fn into_complete(self) -> InlineTheme<Option<Color>> {
        InlineTheme {
            text: self.text.resolve(&None),
            background: self.background.resolve(&None),
        }
    }
}

impl PagerFile {
    pub(super) fn resolve(&self, base: &PagerTheme) -> PagerTheme {
        PagerTheme {
            transparent: self.transparent.unwrap_or(base.transparent),
            title: self.title.resolve(&base.title),
            help: self.help.resolve(&base.help),
            file_name: self.file_name.resolve(&base.file_name),
            progress: self.progress.resolve(&base.progress),
            matches: self.matches.resolve(&base.matches),
            selection: self.selection.as_ref().map_or_else(
                || base.selection.clone(),
                |value| value.resolve(&base.selection),
            ),
            search: self
                .search
                .as_ref()
                .map_or_else(|| base.search.clone(), |value| value.resolve(&base.search)),
            search_current: self.search_current.as_ref().map_or_else(
                || base.search_current.clone(),
                |value| value.resolve(&base.search_current),
            ),
        }
    }

    pub(super) fn into_complete(self, name: &str) -> Result<PagerTheme> {
        Ok(PagerTheme {
            transparent: self.transparent.with_context(|| {
                format!("Embedded theme '{name}' is missing 'pager:transparent'")
            })?,
            title: self.title.resolve(&None),
            help: self.help.resolve(&None),
            file_name: self.file_name.resolve(&None),
            progress: self.progress.resolve(&None),
            matches: self.matches.resolve(&None),
            selection: self.selection.unwrap_or_default().into_complete(),
            search: self.search.unwrap_or_default().into_complete(),
            search_current: self.search_current.unwrap_or_default().into_complete(),
        })
    }
}
