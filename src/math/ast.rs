#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FractionStyle {
    Stacked,
    Inline,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccentKind {
    Hat,
    Bar,
    Overline,
    Vector,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AnnotationKind {
    Over,
    Under,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BraceKind {
    Over,
    Under,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MathFont {
    Script,
    Fraktur,
    Bold,
}

#[derive(Debug)]
pub(crate) struct MathEnvironment {
    pub(crate) name: String,
    pub(crate) alignment: Option<String>,
    pub(crate) rows: Vec<Vec<MathNode>>,
}

#[derive(Debug)]
pub(crate) enum MathNode {
    Sequence(Vec<MathNode>),
    Compact(Box<MathNode>),
    Text(String),
    Fraction {
        numerator: Box<MathNode>,
        denominator: Box<MathNode>,
        style: FractionStyle,
    },
    Root {
        index: Option<Box<MathNode>>,
        radicand: Box<MathNode>,
    },
    Binomial {
        upper: Box<MathNode>,
        lower: Box<MathNode>,
    },
    Operator {
        body: Box<MathNode>,
        limits: bool,
    },
    Scripts {
        base: Box<MathNode>,
        subscript: Option<Box<MathNode>>,
        superscript: Option<Box<MathNode>>,
    },
    Accent {
        kind: AccentKind,
        body: Box<MathNode>,
    },
    Annotation {
        kind: AnnotationKind,
        body: Box<MathNode>,
        annotation: Box<MathNode>,
    },
    Brace {
        kind: BraceKind,
        body: Box<MathNode>,
    },
    Delimited {
        left: String,
        body: Box<MathNode>,
        right: String,
    },
    Environment(MathEnvironment),
    Styled {
        font: MathFont,
        body: Box<MathNode>,
    },
    DisplayStyle,
    LineBreak,
    Unsupported(String),
}

impl MathNode {
    pub(crate) fn sequence(mut nodes: Vec<Self>) -> Self {
        match nodes.len() {
            0 => Self::Text(String::new()),
            1 => nodes.pop().expect("sequence length checked"),
            _ => Self::Sequence(nodes),
        }
    }

    pub(crate) fn is_structured(&self) -> bool {
        match self {
            Self::Sequence(nodes) => nodes.iter().any(Self::is_structured),
            Self::Text(_)
            | Self::Operator { .. }
            | Self::DisplayStyle
            | Self::LineBreak
            | Self::Compact(_)
            | Self::Unsupported(_) => false,
            Self::Styled { body, .. } => body.is_structured(),
            _ => true,
        }
    }

    pub(crate) fn contains_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported(_)) || self.any_child(Self::contains_unsupported)
    }

    pub(crate) fn requests_display_style(&self) -> bool {
        match self {
            Self::DisplayStyle => true,
            Self::Compact(_) => false,
            _ => self.any_child(Self::requests_display_style),
        }
    }

    pub(super) fn any_child(&self, predicate: impl Fn(&Self) -> bool) -> bool {
        match self {
            Self::Sequence(nodes) => nodes.iter().any(predicate),
            Self::Scripts {
                base,
                subscript,
                superscript,
            } => {
                predicate(base)
                    || subscript.as_deref().is_some_and(&predicate)
                    || superscript.as_deref().is_some_and(predicate)
            }
            Self::Fraction {
                numerator,
                denominator,
                ..
            } => predicate(numerator) || predicate(denominator),
            Self::Root { index, radicand } => {
                index.as_deref().is_some_and(&predicate) || predicate(radicand)
            }
            Self::Binomial { upper, lower } => predicate(upper) || predicate(lower),
            Self::Accent { body, .. }
            | Self::Operator { body, .. }
            | Self::Brace { body, .. }
            | Self::Delimited { body, .. }
            | Self::Styled { body, .. }
            | Self::Compact(body) => predicate(body),
            Self::Annotation {
                body, annotation, ..
            } => predicate(body) || predicate(annotation),
            Self::Environment(environment) => environment.rows.iter().flatten().any(predicate),
            Self::Text(_) | Self::DisplayStyle | Self::LineBreak | Self::Unsupported(_) => false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MathDiagnostic {
    pub(crate) offset: usize,
    pub(crate) message: String,
    pub(crate) fatal: bool,
}

#[derive(Debug)]
pub(crate) struct ParsedMath {
    pub(crate) root: MathNode,
    pub(crate) diagnostics: Vec<MathDiagnostic>,
}

impl ParsedMath {
    pub(crate) fn has_fatal_diagnostic(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.fatal)
    }
}
