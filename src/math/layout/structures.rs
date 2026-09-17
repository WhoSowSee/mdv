use super::*;
use crate::math::ScriptKind;

pub(super) fn layout_root(
    index: Option<&MathNode>,
    radicand: &MathNode,
    fonts: &[MathFont],
) -> MathBox {
    let mut prefix = index
        .map(|node| crate::math::rendering::render_script(node, ScriptKind::Sup, fonts).output)
        .unwrap_or_default();
    prefix.push('√');
    let body = if super::super::rendering::root_body_is_atomic(radicand) {
        layout_with_fonts(radicand, fonts)
    } else {
        delimit(layout_with_fonts(radicand, fonts), "(", ")")
    };
    horizontal(&[MathBox::text(prefix), body])
}

pub(super) fn layout_binomial(upper: &MathNode, lower: &MathNode, fonts: &[MathFont]) -> MathBox {
    let upper = layout_with_fonts(upper, fonts);
    let lower = layout_with_fonts(lower, fonts);
    let width = upper.width.max(lower.width).max(1);
    let upper_height = upper.lines.len();
    let mut preserved_trailing = upper.preserved_trailing;
    let mut lines = upper
        .lines
        .iter()
        .map(|line| center(line, width))
        .collect::<Vec<_>>();
    lines.push(" ".repeat(width));
    preserved_trailing.push(None);
    lines.extend(lower.lines.iter().map(|line| center(line, width)));
    preserved_trailing.extend(lower.preserved_trailing);
    delimit(
        MathBox {
            lines,
            preserved_trailing,
            width,
            baseline: upper_height,
        },
        "(",
        ")",
    )
}
