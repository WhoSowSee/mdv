use super::*;
use crate::math::ast::{AccentKind, AnnotationKind, BraceKind, FractionStyle, MathFont};
use crate::math::symbols::{
    command_symbol, is_opening_delimiter_command, literal_command, mathbb_symbol, operator_command,
    spacing_command,
};

impl MathParser<'_> {
    pub(super) fn parse_command(&mut self) -> MathNode {
        let start = self.pos;
        self.bump();
        let name = self.read_command_name();
        if name.is_empty() {
            return MathNode::Text("\\".to_string());
        }
        if name == "\\" {
            return MathNode::LineBreak;
        }
        if let Some(literal) = literal_command(&name) {
            return MathNode::Text(literal.to_string());
        }
        if let Some(space) = spacing_command(&name) {
            return MathNode::Text(space.to_string());
        }
        if let Some((text, limits)) = operator_command(&name) {
            return MathNode::Operator {
                body: Box::new(MathNode::Text(text.to_string())),
                limits,
            };
        }
        if is_opening_delimiter_command(&name) {
            self.skip_whitespace();
        }

        match name.as_str() {
            "displaystyle" => {
                self.skip_whitespace();
                MathNode::DisplayStyle
            }
            "textstyle" | "scriptstyle" | "scriptscriptstyle" | "limits" | "nolimits" => {
                MathNode::Text(String::new())
            }
            "frac" | "dfrac" | "tfrac" => MathNode::Fraction {
                numerator: Box::new(self.parse_required_argument(&name)),
                denominator: Box::new(self.parse_required_argument(&name)),
                style: match name.as_str() {
                    "tfrac" => FractionStyle::Inline,
                    _ => FractionStyle::Stacked,
                },
            },
            "sqrt" => MathNode::Root {
                index: self.parse_optional_argument().map(Box::new),
                radicand: Box::new(self.parse_required_argument("sqrt")),
            },
            "binom" => MathNode::Binomial {
                upper: Box::new(self.parse_required_argument("binom")),
                lower: Box::new(self.parse_required_argument("binom")),
            },
            "left" => self.parse_delimited(),
            "right" => self.unknown_command(start),
            "begin" => self.parse_environment(start),
            "end" => self.unknown_command(start),
            "text" => self.parse_text_argument(),
            "mathrm" | "mathsf" | "mathit" => self.parse_required_argument(&name),
            "mathcal" | "mathscr" | "mathfrak" | "mathbf" | "boldsymbol" => MathNode::Styled {
                font: match name.as_str() {
                    "mathcal" | "mathscr" => MathFont::Script,
                    "mathfrak" => MathFont::Fraktur,
                    _ => MathFont::Bold,
                },
                body: Box::new(self.parse_required_argument(&name)),
            },
            "mathbb" => {
                let argument = self.parse_required_argument("mathbb");
                if argument.contains_unsupported() {
                    return MathNode::Compact(Box::new(argument));
                }
                let rendered = crate::math::rendering::render_inline(&argument);
                MathNode::Text(
                    mathbb_symbol(rendered.trim())
                        .unwrap_or(rendered.trim())
                        .to_string(),
                )
            }
            "operatorname" => {
                let limits = self.peek() == Some('*');
                if limits {
                    self.bump();
                }
                MathNode::Operator {
                    body: Box::new(MathNode::Compact(Box::new(
                        self.parse_required_argument("operatorname"),
                    ))),
                    limits,
                }
            }
            "hat" | "bar" | "overline" | "vec" => MathNode::Accent {
                kind: match name.as_str() {
                    "hat" => AccentKind::Hat,
                    "bar" => AccentKind::Bar,
                    "overline" => AccentKind::Overline,
                    _ => AccentKind::Vector,
                },
                body: Box::new(self.parse_required_argument(&name)),
            },
            "overset" | "underset" => {
                let annotation = self.parse_required_argument(&name);
                let body = self.parse_required_argument(&name);
                MathNode::Annotation {
                    kind: if name == "overset" {
                        AnnotationKind::Over
                    } else {
                        AnnotationKind::Under
                    },
                    body: Box::new(body),
                    annotation: Box::new(annotation),
                }
            }
            "underbrace" | "overbrace" => MathNode::Brace {
                kind: if name == "underbrace" {
                    BraceKind::Under
                } else {
                    BraceKind::Over
                },
                body: Box::new(self.parse_required_argument(&name)),
            },
            "substack" => {
                let (content, content_offset) =
                    self.parse_required_raw_group_with_offset("substack");
                self.environment_from_content("substack", None, &content, content_offset)
            }
            _ => command_symbol(&name)
                .map(|symbol| MathNode::Text(symbol.to_string()))
                .unwrap_or_else(|| self.unknown_command(start)),
        }
    }

    fn parse_delimited(&mut self) -> MathNode {
        if self.depth >= MAX_MATH_DEPTH {
            self.diagnostic("maximum delimiter depth exceeded", true);
            return MathNode::Unsupported(self.input[self.pos..].to_string());
        }
        self.skip_tex_ignored();
        let Some(left) = self.parse_delimiter() else {
            self.diagnostic("missing delimiter after \\left", true);
            return MathNode::Unsupported(self.input[self.pos..].to_string());
        };
        self.skip_tex_ignored();
        self.depth += 1;
        let body = self.parse_sequence(None, true);
        self.depth = self.depth.saturating_sub(1);
        if !self.starts_command("right") {
            self.diagnostic("missing \\right delimiter", true);
            return MathNode::Delimited {
                left,
                body: Box::new(body),
                right: String::new(),
            };
        }
        self.pos += "\\right".len();
        self.skip_tex_ignored();
        let Some(right) = self.parse_delimiter() else {
            self.diagnostic("missing delimiter after \\right", true);
            return MathNode::Delimited {
                left,
                body: Box::new(body),
                right: String::new(),
            };
        };
        MathNode::Delimited {
            left,
            body: Box::new(body),
            right,
        }
    }

    fn parse_delimiter(&mut self) -> Option<String> {
        match self.peek() {
            Some('.') => {
                self.bump();
                Some(String::new())
            }
            Some('\\') => {
                self.bump();
                let name = self.read_command_name();
                if let Some(symbol) = command_symbol(&name) {
                    Some(symbol.to_string())
                } else {
                    Some(format!("\\{name}"))
                }
            }
            Some(ch) => {
                self.bump();
                Some(ch.to_string())
            }
            None => None,
        }
    }
}
