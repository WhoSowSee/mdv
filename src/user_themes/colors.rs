use super::*;

#[derive(Debug)]
pub(crate) struct ColorYaml(pub(crate) Color);

impl<'de> Deserialize<'de> for ColorYaml {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_color_value(&raw)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Default)]
pub(crate) enum OptionalColorYaml {
    #[default]
    Inherit,
    Uncolored,
    Color(ColorYaml),
}

impl<'de> Deserialize<'de> for OptionalColorYaml {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match Option::<ColorYaml>::deserialize(deserializer)? {
            Some(color) => Self::Color(color),
            None => Self::Uncolored,
        })
    }
}

impl OptionalColorYaml {
    pub(super) fn resolve(&self, base: &Option<Color>) -> Option<Color> {
        match self {
            Self::Inherit => base.clone(),
            Self::Uncolored => None,
            Self::Color(color) => Some(color.0.clone()),
        }
    }
}

pub(super) fn pick(value: &Option<ColorYaml>, base: &Color) -> Color {
    value
        .as_ref()
        .map_or_else(|| base.clone(), |color| color.0.clone())
}

pub(super) fn required(value: Option<ColorYaml>, name: &str, field: &str) -> Result<Color> {
    value
        .map(|color| color.0)
        .with_context(|| format!("Embedded theme '{name}' is missing '{field}'"))
}
