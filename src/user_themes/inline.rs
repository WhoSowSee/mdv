use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct InlineFile {
    pub text: Option<ColorYaml>,
    pub background: OptionalColorYaml,
}

impl InlineFile {
    pub(super) fn resolve(&self, base: &InlineTheme) -> InlineTheme {
        InlineTheme {
            text: pick(&self.text, &base.text),
            background: self.background.resolve(&base.background),
        }
    }

    pub(super) fn resolve_optional(
        &self,
        base: &InlineTheme<Option<Color>>,
    ) -> InlineTheme<Option<Color>> {
        InlineTheme {
            text: self
                .text
                .as_ref()
                .map(|color| color.0.clone())
                .or_else(|| base.text.clone()),
            background: self.background.resolve(&base.background),
        }
    }

    pub(super) fn into_complete(self, name: &str, section: &str) -> Result<InlineTheme> {
        Ok(InlineTheme {
            text: required(self.text, name, &format!("{section}:text"))?,
            background: self.background.resolve(&None),
        })
    }

    pub(super) fn into_optional(self) -> InlineTheme<Option<Color>> {
        InlineTheme {
            text: self.text.map(|color| color.0),
            background: self.background.resolve(&None),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct HighlightFile {
    pub text: Option<ColorYaml>,
    pub background: Option<ColorYaml>,
}

impl HighlightFile {
    pub(super) fn resolve(
        &self,
        base: &InlineTheme<Option<Color>, Color>,
    ) -> InlineTheme<Option<Color>, Color> {
        InlineTheme {
            text: self
                .text
                .as_ref()
                .map(|color| color.0.clone())
                .or_else(|| base.text.clone()),
            background: pick(&self.background, &base.background),
        }
    }

    pub(super) fn into_complete(self, name: &str) -> Result<InlineTheme<Option<Color>, Color>> {
        Ok(InlineTheme {
            text: self.text.map(|color| color.0),
            background: required(self.background, name, "highlight:background")?,
        })
    }
}
