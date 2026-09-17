mod wrapping;
pub(crate) use wrapping::wrap_flat_math;
mod boundaries;
use super::ast::{AccentKind, AnnotationKind, BraceKind, MathEnvironment, MathFont, MathNode};
use super::layout;
use super::{MathMode, ScriptKind, convert_script};
use crate::utils::display_width;
use boundaries::node_boundary;
mod fractions;
pub(super) use fractions::render_fraction_inline;
use unicode_segmentation::UnicodeSegmentation;

#[cfg(test)]
thread_local! {
    static RENDER_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn render_parsed_math(node: &MathNode, mode: MathMode) -> String {
    match mode {
        MathMode::Inline => render_inline(node).trim().to_string(),
        MathMode::Display => layout::layout(node).render(),
    }
}

pub(super) fn render_inline(node: &MathNode) -> String {
    render_inline_with_fonts(node, &[])
}

pub(super) fn render_inline_with_fonts(node: &MathNode, fonts: &[MathFont]) -> String {
    #[cfg(test)]
    RENDER_VISITS.with(|visits| visits.set(visits.get() + 1));
    let render_inline = |node| render_inline_with_fonts(node, fonts);
    match node {
        MathNode::Sequence(nodes) => render_inline_sequence(nodes, fonts),
        MathNode::Text(text) => super::fonts::apply(text, fonts),
        MathNode::Operator { body, .. } | MathNode::Compact(body) => render_inline(body),
        MathNode::Unsupported(text) => text.clone(),
        MathNode::Fraction {
            numerator,
            denominator,
            ..
        } => render_fraction_inline(numerator, denominator, fonts),
        MathNode::Root { index, radicand } => render_root_inline(index.as_deref(), radicand, fonts),
        MathNode::Binomial { upper, lower } => format!(
            "({} choose {})",
            render_inline(upper).trim(),
            render_inline(lower).trim()
        ),
        MathNode::Scripts {
            base,
            subscript,
            superscript,
        } => render_scripts_inline(base, subscript.as_deref(), superscript.as_deref(), fonts),
        MathNode::Accent { kind, body } => render_accent_inline(*kind, body, fonts),
        MathNode::Annotation {
            kind,
            body,
            annotation,
        } => {
            let name = match kind {
                AnnotationKind::Over => "overset",
                AnnotationKind::Under => "underset",
            };
            format!(
                "{name}({}, {})",
                render_inline(annotation).trim(),
                render_inline(body).trim()
            )
        }
        MathNode::Brace { kind, body } => {
            let name = match kind {
                BraceKind::Over => "overbrace",
                BraceKind::Under => "underbrace",
            };
            format!("{name}({})", render_inline(body).trim())
        }
        MathNode::Delimited { left, body, right } => {
            format!("{left}{}{right}", render_inline(body))
        }
        MathNode::Environment(environment) => render_environment_inline(environment, fonts),
        MathNode::Styled { font, body } => {
            let mut fonts = fonts.to_vec();
            fonts.push(*font);
            render_inline_with_fonts(body, &fonts)
        }
        MathNode::DisplayStyle => String::new(),
        MathNode::LineBreak => " ".to_string(),
    }
}

fn render_inline_sequence(nodes: &[MathNode], fonts: &[MathFont]) -> String {
    let mut output = String::new();
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 && needs_operator_spacing(&nodes[index - 1], node) {
            output.push(' ');
        }
        let rendered = render_inline_with_fonts(node, fonts);
        if index > 0 && is_operator(&nodes[index - 1]) && matches!(node, MathNode::Fraction { .. })
        {
            output.push('(');
            output.push_str(rendered.trim());
            output.push(')');
        } else {
            output.push_str(&rendered);
        }
    }
    output
}

fn render_root_inline(index: Option<&MathNode>, radicand: &MathNode, fonts: &[MathFont]) -> String {
    let mut output = index
        .map(|node| render_script(node, ScriptKind::Sup, fonts).output)
        .unwrap_or_default();
    output.push('√');
    let body = render_inline_with_fonts(radicand, fonts);
    if root_body_is_atomic(radicand) {
        output.push_str(body.trim());
    } else {
        output.push('(');
        output.push_str(body.trim());
        output.push(')');
    }
    output
}

pub(super) fn root_body_is_atomic(node: &MathNode) -> bool {
    match node {
        MathNode::Text(text) => text.graphemes(true).count() == 1,
        MathNode::Scripts { base, .. } | MathNode::Accent { body: base, .. } => {
            root_body_is_atomic(base)
        }
        MathNode::Styled { body, .. } => root_body_is_atomic(body),
        _ => false,
    }
}

pub(super) fn needs_operator_spacing(left: &MathNode, right: &MathNode) -> bool {
    let left_operator = is_operator(left);
    let right_operator = is_operator(right);
    if !left_operator && !right_operator {
        return false;
    }
    let (Some(left_last), Some(right_first)) =
        (node_boundary(left, false), node_boundary(right, true))
    else {
        return false;
    };
    if left_last.is_whitespace() || right_first.is_whitespace() {
        return false;
    }
    let right_is_open = matches!(right_first, '(' | '[' | '{');
    let right_is_punctuation = matches!(right_first, ')' | ']' | '}' | ',' | '.' | ';' | ':');
    let left_is_open = matches!(left_last, '(' | '[' | '{');
    !(left_operator && (right_is_open || right_is_punctuation)) && !(right_operator && left_is_open)
}

fn is_operator(node: &MathNode) -> bool {
    match node {
        MathNode::Operator { .. } => true,
        MathNode::Scripts { base, .. } => is_operator(base),
        _ => false,
    }
}

fn render_scripts_inline(
    base: &MathNode,
    subscript: Option<&MathNode>,
    superscript: Option<&MathNode>,
    fonts: &[MathFont],
) -> String {
    let render_inline = |node| render_inline_with_fonts(node, fonts);
    if let MathNode::Brace { kind, body } = base {
        let name = match kind {
            BraceKind::Over => "overbrace",
            BraceKind::Under => "underbrace",
        };
        let (annotation, remaining) = match kind {
            BraceKind::Over => (superscript, subscript.map(|node| (node, ScriptKind::Sub))),
            BraceKind::Under => (subscript, superscript.map(|node| (node, ScriptKind::Sup))),
        };
        if let Some(annotation) = annotation {
            let mut output = format!(
                "{name}({}, {})",
                render_inline(body).trim(),
                render_inline(annotation).trim()
            );
            if let Some((script, kind)) = remaining {
                output.push_str(&render_script(script, kind, fonts).output);
            }
            return output;
        }
    }

    let mut output = render_inline(base);
    if let Some(subscript) = subscript {
        output.push_str(&render_script(subscript, ScriptKind::Sub, fonts).output);
    }
    if let Some(superscript) = superscript {
        output.push_str(&render_script(superscript, ScriptKind::Sup, fonts).output);
    }
    output
}

pub(super) fn render_script(
    node: &MathNode,
    kind: ScriptKind,
    fonts: &[MathFont],
) -> super::scripts::ScriptConversion {
    let source = render_inline(node);
    if source.contains('\\') && node.contains_unsupported() {
        let marker = match kind {
            ScriptKind::Sup => '^',
            ScriptKind::Sub => '_',
        };
        let body = if fonts.is_empty() {
            source
        } else {
            render_inline_with_fonts(node, fonts)
        };
        return super::scripts::ScriptConversion {
            output: format!("{marker}({body})"),
            fully_converted: false,
        };
    }
    let mut converted = convert_script(source.trim(), kind);
    if converted.fully_converted {
        converted.output = super::fonts::apply(&converted.output, fonts);
    } else if !fonts.is_empty() {
        converted.output =
            convert_script(render_inline_with_fonts(node, fonts).trim(), kind).output;
    }
    converted
}

fn render_accent_inline(kind: AccentKind, body: &MathNode, fonts: &[MathFont]) -> String {
    let body = render_inline_with_fonts(body, fonts);
    if let Some(compact) = compact_accent_from_rendered(kind, &body) {
        return compact;
    }
    let name = match kind {
        AccentKind::Hat => "hat",
        AccentKind::Bar => "bar",
        AccentKind::Overline => "overline",
        AccentKind::Vector => "vec",
    };
    format!("{name}({})", body.trim())
}

pub(super) fn compact_accent_from_rendered(kind: AccentKind, body: &str) -> Option<String> {
    let mut graphemes = body.graphemes(true);
    let first = graphemes.next()?;
    if graphemes.next().is_some() || display_width(body) > 1 {
        return None;
    }
    let accent = match kind {
        AccentKind::Hat => '\u{0302}',
        AccentKind::Bar | AccentKind::Overline => '\u{0305}',
        AccentKind::Vector => '\u{20d7}',
    };
    Some(format!("{first}{accent}"))
}

fn render_environment_inline(environment: &MathEnvironment, fonts: &[MathFont]) -> String {
    let rows = environment
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| render_inline_with_fonts(cell, fonts).trim().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect::<Vec<_>>()
        .join("; ");
    match environment.name.as_str() {
        "pmatrix" => format!("({rows})"),
        "bmatrix" | "bmatrix*" => format!("[{rows}]"),
        "Bmatrix" | "Bmatrix*" | "cases" => format!("{{{rows}}}"),
        "vmatrix" | "vmatrix*" => format!("|{rows}|"),
        "Vmatrix" | "Vmatrix*" => format!("‖{rows}‖"),
        "matrix" | "array" => format!("[{rows}]"),
        _ => rows,
    }
}

#[cfg(test)]
pub(super) fn reset_render_visits() {
    RENDER_VISITS.with(|visits| visits.set(0));
}

#[cfg(test)]
pub(super) fn render_visits() -> usize {
    RENDER_VISITS.with(std::cell::Cell::get)
}

#[cfg(test)]
mod tests {
    use crate::math::{MathMode, render_math};

    #[test]
    fn display_layout_composes_fraction_matrix_and_annotation() {
        let unsupported = render_math(
            "\\begin{matrix}\\unknown{a \nb}&\\substack{Z\\\\W}\\end{matrix}",
            MathMode::Display,
        );
        let unsupported_lines = unsupported.lines().collect::<Vec<_>>();
        assert_eq!(unsupported_lines.len(), 3, "{unsupported:?}");
        assert!(
            unsupported_lines[1].contains(r"\unknown{a  W"),
            "{unsupported:?}"
        );
        assert!(unsupported_lines[2].contains("b}"), "{unsupported:?}");
        assert_eq!(
            unsupported_lines[0].find('Z'),
            unsupported_lines[1].find('W'),
            "{unsupported:?}"
        );
        assert_eq!(
            render_math("x=\\frac{a+b}{c+d}", MathMode::Display),
            "  a+b\nx=───\n  c+d"
        );
        assert_eq!(
            render_math(
                "A=\\begin{bmatrix}1&2\\\\3&4\\end{bmatrix}",
                MathMode::Display
            ),
            "  ⎡1 2⎤\nA=⎣3 4⎦"
        );
        assert_eq!(
            render_math("a\\overset{def}{=}b", MathMode::Display),
            " def\na = b"
        );
    }
}
