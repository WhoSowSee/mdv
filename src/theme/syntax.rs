use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SyntaxField {
    Keyword,
    String,
    Comment,
    Number,
    Operator,
    Function,
    Variable,
    TypeName,
}

impl SyntaxField {
    pub(crate) const ALL: [Self; 8] = [
        Self::Keyword,
        Self::String,
        Self::Comment,
        Self::Number,
        Self::Operator,
        Self::Function,
        Self::Variable,
        Self::TypeName,
    ];

    pub(crate) fn parse(key: &str) -> Result<Self> {
        match key {
            "keyword" => Ok(Self::Keyword),
            "string" => Ok(Self::String),
            "comment" => Ok(Self::Comment),
            "number" => Ok(Self::Number),
            "operator" => Ok(Self::Operator),
            "function" => Ok(Self::Function),
            "variable" => Ok(Self::Variable),
            "type_name" | "typename" | "type" => Ok(Self::TypeName),
            _ => bail!("Unknown key for custom syntax theme: '{key}'."),
        }
    }

    pub(crate) fn color(self, syntax: &SyntaxTheme) -> &Color {
        match self {
            Self::Keyword => &syntax.keyword,
            Self::String => &syntax.string,
            Self::Comment => &syntax.comment,
            Self::Number => &syntax.number,
            Self::Operator => &syntax.operator,
            Self::Function => &syntax.function,
            Self::Variable => &syntax.variable,
            Self::TypeName => &syntax.type_name,
        }
    }

    pub(super) fn set(self, syntax: &mut SyntaxTheme, color: Color) {
        *match self {
            Self::Keyword => &mut syntax.keyword,
            Self::String => &mut syntax.string,
            Self::Comment => &mut syntax.comment,
            Self::Number => &mut syntax.number,
            Self::Operator => &mut syntax.operator,
            Self::Function => &mut syntax.function,
            Self::Variable => &mut syntax.variable,
            Self::TypeName => &mut syntax.type_name,
        } = color;
    }
}
