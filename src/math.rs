use crate::utils::display_width;

#[derive(Debug, Clone, Copy)]
pub enum MathMode {
    Inline,
    Display,
}

pub fn render_math(input: &str, mode: MathMode) -> String {
    let mut parser = MathParser::new(input, mode);
    let rendered = parser.parse_until(None);
    normalize_output(rendered, mode)
}

pub fn is_math_language_hint(language_hint: &str) -> bool {
    let lower = language_hint.to_ascii_lowercase();
    for token in lower.split([' ', '\t', ',', ';', '|']) {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "math" | "latex" | "tex" | "katex" | "mathjax") {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ScriptKind {
    Sup,
    Sub,
}

struct MathParser {
    chars: Vec<char>,
    pos: usize,
    mode: MathMode,
}

mod parser;
mod rendering;
mod scripts;
mod symbols;

pub(crate) use scripts::convert_script;

use rendering::*;
use scripts::{delimiter_symbol, literal_command, mathbb_symbol, spacing_command};
use symbols::command_symbol;
