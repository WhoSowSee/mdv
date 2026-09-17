#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MathMode {
    Inline,
    Display,
}

pub fn render_math(input: &str, mode: MathMode) -> String {
    render_math_detailed(input, mode).output
}

pub(crate) struct RenderedMath {
    pub(crate) output: String,
    pub(crate) diagnostics: Vec<ast::MathDiagnostic>,
    pub(crate) structured: bool,
}

pub(crate) fn render_math_detailed(input: &str, mode: MathMode) -> RenderedMath {
    let parsed = parser::parse_math(input);
    render_parsed_detailed(input, parsed, mode, mode == MathMode::Display)
}

pub(crate) fn render_table_math_detailed(input: &str, force_display: bool) -> RenderedMath {
    let parsed = parser::parse_math(input);
    let mode = if force_display || parsed.root.requests_display_style() {
        MathMode::Display
    } else {
        MathMode::Inline
    };
    render_parsed_detailed(input, parsed, mode, false)
}

fn render_parsed_detailed(
    input: &str,
    parsed: ast::ParsedMath,
    mode: MathMode,
    track_structure: bool,
) -> RenderedMath {
    if parsed.has_fatal_diagnostic() {
        return RenderedMath {
            output: input.to_string(),
            diagnostics: parsed.diagnostics,
            structured: track_structure,
        };
    }
    RenderedMath {
        output: rendering::render_parsed_math(&parsed.root, mode),
        diagnostics: parsed.diagnostics,
        structured: track_structure && parsed.root.is_structured(),
    }
}

pub fn is_math_language_hint(language_hint: &str) -> bool {
    let lower = language_hint.to_ascii_lowercase();
    lower
        .split([' ', '\t', ',', ';', '|'])
        .any(|token| matches!(token.trim(), "math" | "latex" | "tex" | "katex" | "mathjax"))
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ScriptKind {
    Sup,
    Sub,
}

mod ast;
mod diagnostics;
mod fonts;
mod layout;
mod parser;
mod rendering;
mod scripts;
mod symbols;

pub(crate) use diagnostics::MathDiagnostics;
pub(crate) use rendering::wrap_flat_math;
pub(crate) use scripts::convert_html_script;
use scripts::convert_script;

#[cfg(test)]
mod tests;
