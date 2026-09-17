use super::*;
use crate::math::ScriptKind;
use crate::math::rendering::render_script;

pub(super) fn layout_scripts(
    base: &MathNode,
    subscript: Option<&MathNode>,
    superscript: Option<&MathNode>,
    fonts: &[MathFont],
) -> MathBox {
    if let MathNode::Brace { kind, body } = base {
        let (annotation, remaining_subscript, remaining_superscript) = match kind {
            BraceKind::Under => (subscript, None, superscript),
            BraceKind::Over => (superscript, subscript, None),
        };
        if annotation.is_some() {
            return layout_scripts_around_box(
                layout_brace(*kind, body, annotation, fonts),
                remaining_subscript,
                remaining_superscript,
                fonts,
            );
        }
    }

    if let MathNode::Operator { limits, .. } = base {
        let base = layout_with_fonts(base, fonts);
        return if *limits {
            layout_limits(base, subscript, superscript, fonts)
        } else {
            layout_side_scripts(base, subscript, superscript, fonts)
        };
    }

    let sub = subscript.map(|node| render_script(node, ScriptKind::Sub, fonts));
    let sup = superscript.map(|node| render_script(node, ScriptKind::Sup, fonts));
    if matches!(base, MathNode::Styled { .. })
        && (sub.as_ref().is_some_and(|script| !script.fully_converted)
            || sup.as_ref().is_some_and(|script| !script.fully_converted))
    {
        return layout_side_scripts(
            layout_with_fonts(base, fonts),
            subscript,
            superscript,
            fonts,
        );
    }

    horizontal(&[
        layout_with_fonts(base, fonts),
        MathBox::text(sub.map(|script| script.output).unwrap_or_default()),
        MathBox::text(sup.map(|script| script.output).unwrap_or_default()),
    ])
}

fn layout_scripts_around_box(
    base: MathBox,
    subscript: Option<&MathNode>,
    superscript: Option<&MathNode>,
    fonts: &[MathFont],
) -> MathBox {
    let converted_sub = subscript.map(|node| render_script(node, ScriptKind::Sub, fonts).output);
    let converted_sup = superscript.map(|node| render_script(node, ScriptKind::Sup, fonts).output);
    horizontal(&[
        base,
        MathBox::text(converted_sub.unwrap_or_default()),
        MathBox::text(converted_sup.unwrap_or_default()),
    ])
}

fn layout_side_scripts(
    base: MathBox,
    subscript: Option<&MathNode>,
    superscript: Option<&MathNode>,
    fonts: &[MathFont],
) -> MathBox {
    let sup = superscript.map(|node| layout_with_fonts(node, fonts));
    let sub = subscript.map(|node| layout_with_fonts(node, fonts));
    let script_width = sup
        .as_ref()
        .map_or(0, |item| item.width)
        .max(sub.as_ref().map_or(0, |item| item.width));
    let sup_height = sup.as_ref().map_or(0, |item| item.lines.len());
    let mut script_lines = Vec::new();
    let mut script_preserved = Vec::new();
    if let Some(sup) = sup {
        script_preserved.extend(sup.preserved_trailing);
        script_lines.extend(
            sup.lines
                .into_iter()
                .map(|line| center(&line, script_width)),
        );
    }
    script_lines.push(" ".repeat(script_width));
    script_preserved.push(None);
    if let Some(sub) = sub {
        script_preserved.extend(sub.preserved_trailing);
        script_lines.extend(
            sub.lines
                .into_iter()
                .map(|line| center(&line, script_width)),
        );
    }
    horizontal(&[
        base,
        MathBox {
            lines: script_lines,
            preserved_trailing: script_preserved,
            width: script_width,
            baseline: sup_height,
        },
    ])
}

fn layout_limits(
    base: MathBox,
    subscript: Option<&MathNode>,
    superscript: Option<&MathNode>,
    fonts: &[MathFont],
) -> MathBox {
    let subscript = subscript.map(|node| layout_with_fonts(node, fonts));
    let superscript = superscript.map(|node| layout_with_fonts(node, fonts));
    let width = base
        .width
        .max(subscript.as_ref().map_or(0, |item| item.width))
        .max(superscript.as_ref().map_or(0, |item| item.width));
    let superscript_height = superscript.as_ref().map_or(0, |item| item.lines.len());
    let mut lines = Vec::new();
    let mut preserved_trailing = Vec::new();
    if let Some(superscript) = superscript {
        preserved_trailing.extend(superscript.preserved_trailing);
        lines.extend(superscript.lines.iter().map(|line| center(line, width)));
    }
    preserved_trailing.extend(base.preserved_trailing.clone());
    lines.extend(base.lines.iter().map(|line| center(line, width)));
    if let Some(subscript) = subscript {
        preserved_trailing.extend(subscript.preserved_trailing);
        lines.extend(subscript.lines.iter().map(|line| center(line, width)));
    }
    MathBox {
        lines,
        preserved_trailing,
        width,
        baseline: superscript_height + base.baseline,
    }
}
