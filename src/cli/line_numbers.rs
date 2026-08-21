use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineNumberTarget {
    #[default]
    Rendered,
    Source,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LineNumberOptions {
    pub target: LineNumberTarget,
    pub separator: bool,
}

impl LineNumberOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        Self::from_str(value, false)
    }
}

impl ValueEnum for LineNumberOptions {
    fn value_variants<'a>() -> &'a [Self] {
        const VARIANTS: &[LineNumberOptions] = &[
            LineNumberOptions {
                target: LineNumberTarget::Rendered,
                separator: false,
            },
            LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: false,
            },
            LineNumberOptions {
                target: LineNumberTarget::Rendered,
                separator: true,
            },
            LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: true,
            },
        ];
        VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match (self.target, self.separator) {
            (LineNumberTarget::Rendered, false) => Some(PossibleValue::new("rendered").hide(true)),
            (LineNumberTarget::Source, false) => Some(
                PossibleValue::new("source")
                    .help("Number physical Markdown source lines instead of rendered rows"),
            ),
            (LineNumberTarget::Rendered, true) => Some(
                PossibleValue::new("separator")
                    .help("Display a separator after each rendered row number"),
            ),
            (LineNumberTarget::Source, true) => Some(
                PossibleValue::new("source;separator")
                    .alias("separator;source")
                    .help("Number physical Markdown source lines and add the ` │ ` separator"),
            ),
        }
    }
}

impl fmt::Display for LineNumberOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.target, self.separator) {
            (LineNumberTarget::Rendered, false) => Ok(()),
            (LineNumberTarget::Rendered, true) => f.write_str("separator"),
            (LineNumberTarget::Source, false) => f.write_str("source"),
            (LineNumberTarget::Source, true) => f.write_str("source;separator"),
        }
    }
}
