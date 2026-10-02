use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ThemeFile {
    pub name: String,
    pub description: Option<String>,
    pub extends: Option<String>,
    pub pager: Option<PagerFile>,

    pub text: Option<ColorYaml>,
    pub text_light: Option<ColorYaml>,
    pub h1: Option<ColorYaml>,
    pub h2: Option<ColorYaml>,
    pub h3: Option<ColorYaml>,
    pub h4: Option<ColorYaml>,
    pub h5: Option<ColorYaml>,
    pub h6: Option<ColorYaml>,
    pub code: Option<InlineFile>,
    pub quote: Option<ColorYaml>,
    pub link: Option<ColorYaml>,
    pub emphasis: Option<InlineFile>,
    pub strong: Option<InlineFile>,
    pub strikethrough: Option<InlineFile>,
    pub error: Option<ColorYaml>,
    pub warning: Option<ColorYaml>,
    pub strong_emphasis: Option<InlineFile>,
    pub highlight: Option<HighlightFile>,
    pub background: OptionalColorYaml,
    pub horizontal_rule: OptionalColorYaml,
    pub footnote_separator: OptionalColorYaml,
    pub line_number: Option<LineNumberFile>,
    pub table: Option<TableFile>,
    pub math: Option<MathFile>,
    pub front_matter: Option<FrontMatterFile>,
    pub code_block: Option<CodeBlockFile>,
    pub list: Option<ListFile>,
    pub todo: Option<TodoFile>,
    pub details_border: OptionalColorYaml,
    pub callout: Option<CalloutFile>,
    pub inline_style: InlineStyleOverrides,
    pub syntax: Option<SyntaxFile>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SyntaxFile {
    pub keyword: Option<ColorYaml>,
    pub string: Option<ColorYaml>,
    pub comment: Option<ColorYaml>,
    pub number: Option<ColorYaml>,
    pub operator: Option<ColorYaml>,
    pub function: Option<ColorYaml>,
    pub variable: Option<ColorYaml>,
    pub type_name: Option<ColorYaml>,
}

impl SyntaxFile {
    pub(super) fn resolve(&self, base: &SyntaxTheme) -> SyntaxTheme {
        SyntaxTheme {
            keyword: pick(&self.keyword, &base.keyword),
            string: pick(&self.string, &base.string),
            comment: pick(&self.comment, &base.comment),
            number: pick(&self.number, &base.number),
            operator: pick(&self.operator, &base.operator),
            function: pick(&self.function, &base.function),
            variable: pick(&self.variable, &base.variable),
            type_name: pick(&self.type_name, &base.type_name),
        }
    }
}

impl ThemeFile {
    pub fn resolve(&self, base: &Theme) -> Theme {
        let mut inline_style = base.inline_style.clone();
        inline_style.apply_overrides(&self.inline_style);
        Theme {
            name: self.name.clone(),
            description: self
                .description
                .clone()
                .unwrap_or_else(|| base.description.clone()),
            pager: self
                .pager
                .as_ref()
                .map_or_else(|| base.pager.clone(), |value| value.resolve(&base.pager)),
            text: pick(&self.text, &base.text),
            text_light: pick(&self.text_light, &base.text_light),
            h1: pick(&self.h1, &base.h1),
            h2: pick(&self.h2, &base.h2),
            h3: pick(&self.h3, &base.h3),
            h4: pick(&self.h4, &base.h4),
            h5: pick(&self.h5, &base.h5),
            h6: pick(&self.h6, &base.h6),
            code: self
                .code
                .as_ref()
                .map_or_else(|| base.code.clone(), |group| group.resolve(&base.code)),
            quote: pick(&self.quote, &base.quote),
            link: pick(&self.link, &base.link),
            emphasis: self.emphasis.as_ref().map_or_else(
                || base.emphasis.clone(),
                |group| group.resolve(&base.emphasis),
            ),
            strong: self
                .strong
                .as_ref()
                .map_or_else(|| base.strong.clone(), |group| group.resolve(&base.strong)),
            strikethrough: self.strikethrough.as_ref().map_or_else(
                || base.strikethrough.clone(),
                |group| group.resolve(&base.strikethrough),
            ),
            error: pick(&self.error, &base.error),
            warning: pick(&self.warning, &base.warning),
            strong_emphasis: self.strong_emphasis.as_ref().map_or_else(
                || base.strong_emphasis.clone(),
                |group| group.resolve_optional(&base.strong_emphasis),
            ),
            highlight: self.highlight.as_ref().map_or_else(
                || base.highlight.clone(),
                |group| group.resolve(&base.highlight),
            ),
            background: self.background.resolve(&base.background),
            horizontal_rule: self.horizontal_rule.resolve(&base.horizontal_rule),
            footnote_separator: self.footnote_separator.resolve(&base.footnote_separator),
            line_number: self.line_number.as_ref().map_or_else(
                || base.line_number.clone(),
                |group| group.resolve(&base.line_number),
            ),
            table: self
                .table
                .as_ref()
                .map_or_else(|| base.table.clone(), |group| group.resolve(&base.table)),
            math: self
                .math
                .as_ref()
                .map_or_else(|| base.math.clone(), |group| group.resolve(&base.math)),
            front_matter: self.front_matter.as_ref().map_or_else(
                || base.front_matter.clone(),
                |group| group.resolve(&base.front_matter),
            ),
            code_block: self.code_block.as_ref().map_or_else(
                || base.code_block.clone(),
                |group| group.resolve(&base.code_block),
            ),
            list: self
                .list
                .as_ref()
                .map_or_else(|| base.list.clone(), |group| group.resolve(&base.list)),
            todo: self
                .todo
                .as_ref()
                .map_or_else(|| base.todo.clone(), |group| group.resolve(&base.todo)),
            details_border: self.details_border.resolve(&base.details_border),
            callout: self.callout.as_ref().map_or_else(
                || base.callout.clone(),
                |group| group.resolve(&base.callout),
            ),
            inline_style,
            syntax: self.syntax.as_ref().map_or_else(
                || base.syntax.clone(),
                |syntax| syntax.resolve(&base.syntax),
            ),
            color_priorities: crate::theme::ColorPriorities::default(),
        }
    }
}

pub(crate) fn parse_embedded_theme(expected_name: &str, source: &str) -> Result<Theme> {
    let file: ThemeFile = serde_yaml::from_str(source)
        .with_context(|| format!("Failed to parse embedded theme '{expected_name}'"))?;
    if file.name != expected_name {
        bail!(
            "Embedded theme name '{}' does not match expected name '{}'",
            file.name,
            expected_name
        );
    }
    file.into_complete()
}
