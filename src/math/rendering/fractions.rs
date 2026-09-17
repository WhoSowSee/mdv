use super::boundaries::contains_fraction;
use super::{MathFont, MathNode, render_inline_with_fonts};

pub(in crate::math) fn render_fraction_inline(
    numerator: &MathNode,
    denominator: &MathNode,
    fonts: &[MathFont],
) -> String {
    format!(
        "{}⁄{}",
        wrap_if_needed(numerator, fonts),
        wrap_if_needed(denominator, fonts)
    )
}

fn wrap_if_needed(node: &MathNode, fonts: &[MathFont]) -> String {
    let rendered = render_inline_with_fonts(node, fonts);
    let trimmed = rendered.trim();
    let needs_parens = contains_fraction(node)
        || trimmed
            .chars()
            .any(|ch| matches!(ch, '+' | '-' | '*' | '/' | '=' | '−'));
    if needs_parens && !has_outer_parentheses(trimmed) {
        format!("({trimmed})")
    } else {
        trimmed.to_string()
    }
}

fn has_outer_parentheses(text: &str) -> bool {
    if !text.starts_with('(') {
        return false;
    }
    let mut depth = 0usize;
    for (offset, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(remaining) = depth.checked_sub(1) else {
                    return false;
                };
                depth = remaining;
                if depth == 0 {
                    return offset + ch.len_utf8() == text.len();
                }
            }
            _ => {}
        }
    }
    false
}
