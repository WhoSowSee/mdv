use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LineNumberFile {
    pub number: Option<ColorYaml>,
    pub separator: Option<ColorYaml>,
}

impl LineNumberFile {
    pub(super) fn resolve(&self, base: &LineNumberTheme) -> LineNumberTheme {
        LineNumberTheme {
            number: pick(self.number.as_ref(), &base.number),
            separator: pick(self.separator.as_ref(), &base.separator),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct TableFile {
    pub header: Option<ColorYaml>,
    pub border: OptionalColorYaml,
}

impl TableFile {
    pub(super) fn resolve(&self, base: &TableTheme) -> TableTheme {
        TableTheme {
            header: pick(self.header.as_ref(), &base.header),
            border: self.border.resolve(base.border.as_ref()),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct MathFile {
    pub text: Option<ColorYaml>,
    pub border: OptionalColorYaml,
}

impl MathFile {
    pub(super) fn resolve(&self, base: &MathTheme) -> MathTheme {
        MathTheme {
            text: pick(self.text.as_ref(), &base.text),
            border: self.border.resolve(base.border.as_ref()),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct FrontMatterFile {
    pub title: Option<ColorYaml>,
    pub key: Option<ColorYaml>,
    pub value: Option<ColorYaml>,
    pub border: OptionalColorYaml,
}

impl FrontMatterFile {
    pub(super) fn resolve(&self, base: &FrontMatterTheme) -> FrontMatterTheme {
        FrontMatterTheme {
            title: pick(self.title.as_ref(), &base.title),
            key: pick(self.key.as_ref(), &base.key),
            value: pick(self.value.as_ref(), &base.value),
            border: self.border.resolve(base.border.as_ref()),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct CodeBlockFile {
    pub label: Option<ColorYaml>,
    pub border: OptionalColorYaml,
}

impl CodeBlockFile {
    pub(super) fn resolve(&self, base: &CodeBlockTheme) -> CodeBlockTheme {
        CodeBlockTheme {
            label: pick(self.label.as_ref(), &base.label),
            border: self.border.resolve(base.border.as_ref()),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ListFile {
    pub ordered: Option<ColorYaml>,
    pub unordered: Option<ColorYaml>,
}

impl ListFile {
    pub(super) fn resolve(&self, base: &ListTheme) -> ListTheme {
        ListTheme {
            ordered: pick(self.ordered.as_ref(), &base.ordered),
            unordered: pick(self.unordered.as_ref(), &base.unordered),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct TodoFile {
    pub checked: Option<ColorYaml>,
    pub unchecked: Option<ColorYaml>,
}

impl TodoFile {
    pub(super) fn resolve(&self, base: &TodoTheme) -> TodoTheme {
        TodoTheme {
            checked: pick(self.checked.as_ref(), &base.checked),
            unchecked: pick(self.unchecked.as_ref(), &base.unchecked),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct CalloutPaletteFile {
    pub note: Option<ColorYaml>,
    #[serde(rename = "abstract")]
    pub abstract_color: Option<ColorYaml>,
    pub info: Option<ColorYaml>,
    pub todo: Option<ColorYaml>,
    pub tip: Option<ColorYaml>,
    pub success: Option<ColorYaml>,
    pub question: Option<ColorYaml>,
    pub warning: Option<ColorYaml>,
    pub failure: Option<ColorYaml>,
    pub danger: Option<ColorYaml>,
    pub bug: Option<ColorYaml>,
    pub example: Option<ColorYaml>,
    pub quote: Option<ColorYaml>,
}

impl CalloutPaletteFile {
    pub(super) fn resolve(&self, base: &CalloutPalette) -> CalloutPalette {
        CalloutPalette {
            note: pick(self.note.as_ref(), &base.note),
            abstract_color: pick(self.abstract_color.as_ref(), &base.abstract_color),
            info: pick(self.info.as_ref(), &base.info),
            todo: pick(self.todo.as_ref(), &base.todo),
            tip: pick(self.tip.as_ref(), &base.tip),
            success: pick(self.success.as_ref(), &base.success),
            question: pick(self.question.as_ref(), &base.question),
            warning: pick(self.warning.as_ref(), &base.warning),
            failure: pick(self.failure.as_ref(), &base.failure),
            danger: pick(self.danger.as_ref(), &base.danger),
            bug: pick(self.bug.as_ref(), &base.bug),
            example: pick(self.example.as_ref(), &base.example),
            quote: pick(self.quote.as_ref(), &base.quote),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct CalloutFile {
    pub label: OptionalColorYaml,
    pub border: OptionalColorYaml,
    pub palette: Option<CalloutPaletteFile>,
}

impl CalloutFile {
    pub(super) fn resolve(&self, base: &CalloutTheme) -> CalloutTheme {
        CalloutTheme {
            label: self.label.resolve(base.label.as_ref()),
            border: self.border.resolve(base.border.as_ref()),
            palette: self.palette.as_ref().map_or_else(
                || base.palette.clone(),
                |palette| palette.resolve(&base.palette),
            ),
        }
    }
}
