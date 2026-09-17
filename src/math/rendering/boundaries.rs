use super::MathNode;

pub(super) fn contains_fraction(node: &MathNode) -> bool {
    match node {
        MathNode::Fraction { .. } => true,
        MathNode::Root { radicand, .. } => contains_fraction(radicand),
        _ => node.any_child(contains_fraction),
    }
}

pub(super) fn node_boundary(node: &MathNode, first: bool) -> Option<char> {
    match node {
        MathNode::Text(text) | MathNode::Unsupported(text) => {
            if first {
                text.chars().next()
            } else {
                text.chars().next_back()
            }
        }
        MathNode::Sequence(nodes) => {
            if first {
                nodes.iter().find_map(|node| node_boundary(node, true))
            } else {
                nodes
                    .iter()
                    .rev()
                    .find_map(|node| node_boundary(node, false))
            }
        }
        MathNode::Fraction {
            numerator,
            denominator,
            ..
        } => {
            let child = if first { numerator } else { denominator };
            if needs_fraction_parentheses(child) {
                Some(if first { '(' } else { ')' })
            } else {
                node_boundary(child, first)
            }
        }
        MathNode::Root { .. } => Some(if first { '√' } else { ')' }),
        MathNode::Binomial { .. } => Some(if first { '(' } else { ')' }),
        MathNode::Scripts { base, .. }
        | MathNode::Compact(base)
        | MathNode::Operator { body: base, .. }
        | MathNode::Accent { body: base, .. }
        | MathNode::Styled { body: base, .. } => node_boundary(base, first),
        MathNode::Annotation { .. } | MathNode::Brace { .. } => Some(if first { 'o' } else { ')' }),
        MathNode::Delimited { left, body, right } => {
            if first {
                left.chars().next().or_else(|| node_boundary(body, first))
            } else {
                right
                    .chars()
                    .next_back()
                    .or_else(|| node_boundary(body, first))
            }
        }
        MathNode::Environment(environment) => {
            let (opening, closing) = match environment.name.as_str() {
                "pmatrix" => ('(', ')'),
                "bmatrix" | "bmatrix*" | "matrix" | "array" => ('[', ']'),
                "Bmatrix" | "Bmatrix*" | "cases" => ('{', '}'),
                "vmatrix" | "vmatrix*" => ('|', '|'),
                "Vmatrix" | "Vmatrix*" => ('‖', '‖'),
                _ => (' ', ' '),
            };
            Some(if first { opening } else { closing })
        }
        MathNode::DisplayStyle => None,
        MathNode::LineBreak => Some(' '),
    }
}

fn needs_fraction_parentheses(node: &MathNode) -> bool {
    contains_fraction(node) || contains_math_operator(node)
}

fn contains_math_operator(node: &MathNode) -> bool {
    match node {
        MathNode::Text(text) | MathNode::Unsupported(text) => text
            .chars()
            .any(|ch| matches!(ch, '+' | '-' | '*' | '/' | '=' | '−')),
        _ => node.any_child(contains_math_operator),
    }
}
