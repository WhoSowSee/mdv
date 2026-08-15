use super::*;

/// [`Color`] that accepts the same value formats as `--custom-theme`
/// (named, hex, rgb, ansi-index) and reuses [`parse_color_value`].
#[derive(Debug, Clone)]
pub(crate) struct ColorYaml(pub(crate) Color);

impl<'de> Deserialize<'de> for ColorYaml {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_color_value(&raw)
            .map(ColorYaml)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ThemeFile {
    pub name: String,
    pub description: Option<String>,
    pub extends: Option<String>,
    pub pager_status_bar_transparent: Option<bool>,

    pub text: Option<ColorYaml>,
    pub text_light: Option<ColorYaml>,
    pub line_number: Option<ColorYaml>,
    pub line_number_separator: Option<ColorYaml>,
    pub h1: Option<ColorYaml>,
    pub h2: Option<ColorYaml>,
    pub h3: Option<ColorYaml>,
    pub h4: Option<ColorYaml>,
    pub h5: Option<ColorYaml>,
    pub h6: Option<ColorYaml>,
    pub code: Option<ColorYaml>,
    pub quote: Option<ColorYaml>,
    pub link: Option<ColorYaml>,
    pub emphasis: Option<ColorYaml>,
    pub strong: Option<ColorYaml>,
    pub strong_emphasis: Option<ColorYaml>,
    pub strikethrough: Option<ColorYaml>,
    pub highlight: Option<ColorYaml>,
    pub highlight_background: Option<ColorYaml>,
    pub emphasis_background: Option<ColorYaml>,
    pub strong_background: Option<ColorYaml>,
    pub strong_emphasis_background: Option<ColorYaml>,
    pub code_background: Option<ColorYaml>,
    pub strikethrough_background: Option<ColorYaml>,
    pub background: Option<ColorYaml>,
    pub border: Option<ColorYaml>,
    pub list_marker: Option<ColorYaml>,
    pub table_header: Option<ColorYaml>,
    pub table_border: Option<ColorYaml>,
    pub error: Option<ColorYaml>,
    pub warning: Option<ColorYaml>,

    pub inline_style: InlineStyleOverrides,

    pub syntax: Option<SyntaxFile>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
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

impl ThemeFile {
    /// Resolve this partial description against `base`, producing a fully
    /// populated [`Theme`]. Omitted fields inherit from `base`; specified
    /// fields override it.
    pub fn resolve(&self, base: &Theme) -> Theme {
        let pick = |override_color: &Option<ColorYaml>, base_color: &Color| -> Color {
            override_color
                .as_ref()
                .map(|c| c.0.clone())
                .unwrap_or_else(|| base_color.clone())
        };
        let pick_optional =
            |override_color: &Option<ColorYaml>, base_color: &Option<Color>| -> Option<Color> {
                override_color
                    .as_ref()
                    .map(|c| c.0.clone())
                    .or_else(|| base_color.clone())
            };

        let syntax = match &self.syntax {
            Some(syntax_file) => SyntaxTheme {
                keyword: pick(&syntax_file.keyword, &base.syntax.keyword),
                string: pick(&syntax_file.string, &base.syntax.string),
                comment: pick(&syntax_file.comment, &base.syntax.comment),
                number: pick(&syntax_file.number, &base.syntax.number),
                operator: pick(&syntax_file.operator, &base.syntax.operator),
                function: pick(&syntax_file.function, &base.syntax.function),
                variable: pick(&syntax_file.variable, &base.syntax.variable),
                type_name: pick(&syntax_file.type_name, &base.syntax.type_name),
            },
            None => base.syntax.clone(),
        };
        let mut inline_style = base.inline_style.clone();
        inline_style.apply_overrides(&self.inline_style);

        Theme {
            name: self.name.clone(),
            description: self
                .description
                .clone()
                .unwrap_or_else(|| base.description.clone()),
            pager_status_bar_transparent: self
                .pager_status_bar_transparent
                .unwrap_or(base.pager_status_bar_transparent),
            text: pick(&self.text, &base.text),
            text_light: pick(&self.text_light, &base.text_light),
            line_number: pick(&self.line_number, &base.line_number),
            line_number_separator: pick(&self.line_number_separator, &base.line_number_separator),
            h1: pick(&self.h1, &base.h1),
            h2: pick(&self.h2, &base.h2),
            h3: pick(&self.h3, &base.h3),
            h4: pick(&self.h4, &base.h4),
            h5: pick(&self.h5, &base.h5),
            h6: pick(&self.h6, &base.h6),
            code: pick(&self.code, &base.code),
            quote: pick(&self.quote, &base.quote),
            link: pick(&self.link, &base.link),
            emphasis: pick(&self.emphasis, &base.emphasis),
            strong: pick(&self.strong, &base.strong),
            strong_emphasis: pick_optional(&self.strong_emphasis, &base.strong_emphasis),
            strikethrough: pick(&self.strikethrough, &base.strikethrough),
            highlight: pick_optional(&self.highlight, &base.highlight),
            highlight_background: pick(&self.highlight_background, &base.highlight_background),
            emphasis_background: pick_optional(
                &self.emphasis_background,
                &base.emphasis_background,
            ),
            strong_background: pick_optional(&self.strong_background, &base.strong_background),
            strong_emphasis_background: pick_optional(
                &self.strong_emphasis_background,
                &base.strong_emphasis_background,
            ),
            code_background: pick_optional(&self.code_background, &base.code_background),
            strikethrough_background: pick_optional(
                &self.strikethrough_background,
                &base.strikethrough_background,
            ),
            background: pick_optional(&self.background, &base.background),
            border: pick(&self.border, &base.border),
            inline_style,
            list_marker: pick(&self.list_marker, &base.list_marker),
            table_header: pick(&self.table_header, &base.table_header),
            table_border: pick(&self.table_border, &base.table_border),
            error: pick(&self.error, &base.error),
            warning: pick(&self.warning, &base.warning),
            syntax,
        }
    }

    fn into_complete(mut self) -> Result<Theme> {
        if self.name.trim().is_empty() {
            bail!("Theme file is missing 'name' field");
        }
        if self.extends.is_some() {
            bail!("Embedded themes cannot use 'extends'");
        }

        let name = self.name.trim().to_string();
        let description = self
            .description
            .take()
            .filter(|value| !value.trim().is_empty())
            .with_context(|| format!("Embedded theme '{name}' is missing 'description'"))?;
        let mut syntax = self
            .syntax
            .take()
            .with_context(|| format!("Embedded theme '{name}' is missing 'syntax'"))?;
        let mut inline_style = InlineStyleSet::default();
        inline_style.apply_overrides(&self.inline_style);

        macro_rules! color {
            ($owner:expr, $field:ident) => {
                $owner.$field.take().map(|value| value.0).with_context(|| {
                    format!(
                        "Embedded theme '{}' is missing '{}'",
                        name,
                        stringify!($field)
                    )
                })?
            };
        }

        Ok(Theme {
            description,
            pager_status_bar_transparent: self.pager_status_bar_transparent.with_context(|| {
                format!(
                    "Embedded theme '{}' is missing 'pager_status_bar_transparent'",
                    name
                )
            })?,
            text: color!(self, text),
            text_light: color!(self, text_light),
            line_number: color!(self, line_number),
            line_number_separator: color!(self, line_number_separator),
            h1: color!(self, h1),
            h2: color!(self, h2),
            h3: color!(self, h3),
            h4: color!(self, h4),
            h5: color!(self, h5),
            h6: color!(self, h6),
            code: color!(self, code),
            quote: color!(self, quote),
            link: color!(self, link),
            emphasis: color!(self, emphasis),
            strong: color!(self, strong),
            strong_emphasis: self.strong_emphasis.take().map(|value| value.0),
            strikethrough: color!(self, strikethrough),
            highlight: self.highlight.take().map(|value| value.0),
            highlight_background: color!(self, highlight_background),
            emphasis_background: self.emphasis_background.take().map(|value| value.0),
            strong_background: self.strong_background.take().map(|value| value.0),
            strong_emphasis_background: self.strong_emphasis_background.take().map(|value| value.0),
            code_background: self.code_background.take().map(|value| value.0),
            strikethrough_background: self.strikethrough_background.take().map(|value| value.0),
            background: self.background.take().map(|value| value.0),
            border: color!(self, border),
            inline_style,
            list_marker: color!(self, list_marker),
            table_header: color!(self, table_header),
            table_border: color!(self, table_border),
            error: color!(self, error),
            warning: color!(self, warning),
            syntax: SyntaxTheme {
                keyword: color!(syntax, keyword),
                string: color!(syntax, string),
                comment: color!(syntax, comment),
                number: color!(syntax, number),
                operator: color!(syntax, operator),
                function: color!(syntax, function),
                variable: color!(syntax, variable),
                type_name: color!(syntax, type_name),
            },
            name,
        })
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
