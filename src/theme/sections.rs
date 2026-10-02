use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineTheme<Text = Color, Background = Option<Color>> {
    pub text: Text,
    pub background: Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineNumberTheme {
    pub number: Color,
    pub separator: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableTheme {
    pub header: Color,
    pub border: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathTheme {
    pub text: Color,
    pub border: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatterTheme {
    pub title: Color,
    pub key: Color,
    pub value: Color,
    pub border: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockTheme {
    pub label: Color,
    pub border: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutTheme {
    pub label: Option<Color>,
    pub border: Option<Color>,
    pub palette: CalloutPalette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutPalette {
    pub note: Color,
    #[serde(rename = "abstract")]
    pub abstract_color: Color,
    pub info: Color,
    pub todo: Color,
    pub tip: Color,
    pub success: Color,
    pub question: Color,
    pub warning: Color,
    pub failure: Color,
    pub danger: Color,
    pub bug: Color,
    pub example: Color,
    pub quote: Color,
}

impl CalloutPalette {
    pub(crate) fn color_mut(&mut self, kind: &str) -> Result<&mut Color> {
        match kind {
            "note" => Ok(&mut self.note),
            "abstract" => Ok(&mut self.abstract_color),
            "info" => Ok(&mut self.info),
            "todo" => Ok(&mut self.todo),
            "tip" => Ok(&mut self.tip),
            "success" => Ok(&mut self.success),
            "question" => Ok(&mut self.question),
            "warning" => Ok(&mut self.warning),
            "failure" => Ok(&mut self.failure),
            "danger" => Ok(&mut self.danger),
            "bug" => Ok(&mut self.bug),
            "example" => Ok(&mut self.example),
            "quote" => Ok(&mut self.quote),
            _ => bail!("Unknown callout palette entry: '{kind}'."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTheme {
    pub ordered: Color,
    pub unordered: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoTheme {
    pub checked: Color,
    pub unchecked: Color,
}
