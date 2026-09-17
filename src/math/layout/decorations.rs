use super::*;

pub(super) fn layout_accent(kind: AccentKind, body: &MathNode, fonts: &[MathFont]) -> MathBox {
    let body = layout_with_fonts(body, fonts);
    if body.lines.len() == 1
        && let Some(compact) =
            crate::math::rendering::compact_accent_from_rendered(kind, &body.lines[0])
    {
        return MathBox::text(compact);
    }
    let accent = match kind {
        AccentKind::Hat => center("^", body.width),
        AccentKind::Vector => center("→", body.width),
        AccentKind::Bar | AccentKind::Overline => "‾".repeat(body.width),
    };
    let mut lines = vec![accent];
    lines.extend(body.lines);
    let mut preserved_trailing = vec![None];
    preserved_trailing.extend(body.preserved_trailing);
    MathBox {
        lines,
        preserved_trailing,
        width: body.width,
        baseline: body.baseline + 1,
    }
}

pub(super) fn layout_annotation(
    kind: AnnotationKind,
    body: &MathNode,
    annotation: &MathNode,
    fonts: &[MathFont],
) -> MathBox {
    let body = layout_with_fonts(body, fonts);
    let annotation = layout_with_fonts(annotation, fonts);
    let width = body.width.max(annotation.width);
    match kind {
        AnnotationKind::Over => {
            let mut lines = annotation
                .lines
                .iter()
                .map(|line| center(line, width))
                .collect::<Vec<_>>();
            lines.extend(body.lines.iter().map(|line| center(line, width)));
            let mut preserved_trailing = annotation.preserved_trailing.clone();
            preserved_trailing.extend(body.preserved_trailing.clone());
            MathBox {
                lines,
                preserved_trailing,
                width,
                baseline: annotation.lines.len() + body.baseline,
            }
        }
        AnnotationKind::Under => {
            let mut lines = body
                .lines
                .iter()
                .map(|line| center(line, width))
                .collect::<Vec<_>>();
            lines.extend(annotation.lines.iter().map(|line| center(line, width)));
            let mut preserved_trailing = body.preserved_trailing.clone();
            preserved_trailing.extend(annotation.preserved_trailing.clone());
            MathBox {
                lines,
                preserved_trailing,
                width,
                baseline: body.baseline,
            }
        }
    }
}

pub(super) fn layout_brace(
    kind: BraceKind,
    body: &MathNode,
    annotation: Option<&MathNode>,
    fonts: &[MathFont],
) -> MathBox {
    let body = layout_with_fonts(body, fonts);
    let annotation = annotation.map(|node| layout_with_fonts(node, fonts));
    let width = body
        .width
        .max(annotation.as_ref().map_or(0, |item| item.width))
        .max(2);
    let brace = match kind {
        BraceKind::Under => format!("└{}┘", "─".repeat(width.saturating_sub(2))),
        BraceKind::Over => format!("┌{}┐", "─".repeat(width.saturating_sub(2))),
    };
    let body_lines = body.lines.iter().map(|line| center(line, width));
    let annotation_lines = annotation
        .as_ref()
        .into_iter()
        .flat_map(|item| item.lines.iter().map(|line| center(line, width)));
    let lines = match kind {
        BraceKind::Under => body_lines
            .chain(std::iter::once(brace))
            .chain(annotation_lines)
            .collect(),
        BraceKind::Over => annotation_lines
            .chain(std::iter::once(brace))
            .chain(body_lines)
            .collect(),
    };
    let preserved_trailing = match kind {
        BraceKind::Under => body
            .preserved_trailing
            .iter()
            .cloned()
            .chain(std::iter::once(None))
            .chain(
                annotation
                    .iter()
                    .flat_map(|item| item.preserved_trailing.iter().cloned()),
            )
            .collect(),
        BraceKind::Over => annotation
            .iter()
            .flat_map(|item| item.preserved_trailing.iter().cloned())
            .chain(std::iter::once(None))
            .chain(body.preserved_trailing.iter().cloned())
            .collect(),
    };
    let baseline = match kind {
        BraceKind::Under => body.baseline,
        BraceKind::Over => {
            annotation.as_ref().map_or(0, |item| item.lines.len()) + 1 + body.baseline
        }
    };
    MathBox {
        lines,
        preserved_trailing,
        width,
        baseline,
    }
}
