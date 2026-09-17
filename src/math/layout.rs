use super::ast::{AccentKind, AnnotationKind, BraceKind, FractionStyle, MathFont, MathNode};
use super::rendering::needs_operator_spacing;
use crate::utils::display_width;

#[derive(Debug)]
pub(crate) struct MathBox {
    pub(crate) lines: Vec<String>,
    pub(crate) preserved_trailing: Vec<Option<String>>,
    pub(crate) width: usize,
    pub(crate) baseline: usize,
}

impl MathBox {
    fn text(text: impl Into<String>) -> Self {
        let text = text.into();
        let lines = if text.contains('\n') {
            text.split('\n').map(str::to_string).collect::<Vec<_>>()
        } else {
            vec![text]
        };
        Self {
            preserved_trailing: vec![None; lines.len()],
            width: lines
                .iter()
                .map(|line| display_width(line))
                .max()
                .unwrap_or(0),
            lines,
            baseline: 0,
        }
    }

    fn unsupported(text: &str) -> Self {
        let lines = text.split('\n').map(str::to_string).collect::<Vec<_>>();
        let preserved_trailing = lines
            .iter()
            .map(|line| Some(line[line.trim_end().len()..].to_string()))
            .collect();
        Self {
            width: lines
                .iter()
                .map(|line| display_width(line))
                .max()
                .unwrap_or(0),
            lines,
            preserved_trailing,
            baseline: 0,
        }
    }

    pub(crate) fn render(&self) -> String {
        let mut first = 0usize;
        let mut last = self.lines.len();
        while first < last
            && self.lines[first].trim_end().is_empty()
            && self.preserved_trailing[first].is_none()
        {
            first += 1;
        }
        while last > first
            && self.lines[last - 1].trim_end().is_empty()
            && self.preserved_trailing[last - 1].is_none()
        {
            last -= 1;
        }

        let mut output = String::new();
        for (index, line) in self.lines[first..last].iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            output.push_str(line.trim_end());
            if let Some(trailing) = &self.preserved_trailing[first + index] {
                output.push_str(trailing);
            }
        }
        output
    }
}

pub(crate) fn layout(node: &MathNode) -> MathBox {
    layout_with_fonts(node, &[])
}

fn layout_with_fonts(node: &MathNode, fonts: &[MathFont]) -> MathBox {
    match node {
        MathNode::Sequence(nodes) => layout_sequence(nodes, fonts),
        MathNode::Text(text) => MathBox::text(super::fonts::apply(text, fonts)),
        MathNode::Operator { body, .. } => layout_with_fonts(body, fonts),
        MathNode::Compact(body) => {
            let rendered = super::rendering::render_inline_with_fonts(body, fonts);
            if body.contains_unsupported() {
                MathBox::unsupported(&rendered)
            } else {
                MathBox::text(rendered)
            }
        }
        MathNode::Unsupported(text) => MathBox::unsupported(text),
        MathNode::Fraction {
            numerator,
            denominator,
            style,
        } => layout_fraction(numerator, denominator, *style, fonts),
        MathNode::Root { index, radicand } => layout_root(index.as_deref(), radicand, fonts),
        MathNode::Binomial { upper, lower } => layout_binomial(upper, lower, fonts),
        MathNode::Scripts {
            base,
            subscript,
            superscript,
        } => layout_scripts(base, subscript.as_deref(), superscript.as_deref(), fonts),
        MathNode::Accent { kind, body } => layout_accent(*kind, body, fonts),
        MathNode::Annotation {
            kind,
            body,
            annotation,
        } => layout_annotation(*kind, body, annotation, fonts),
        MathNode::Brace { kind, body } => layout_brace(*kind, body, None, fonts),
        MathNode::Delimited { left, body, right } => {
            delimit(layout_with_fonts(body, fonts), left, right)
        }
        MathNode::Environment(environment) => layout_environment(environment, fonts),
        MathNode::Styled { font, body } => {
            let mut fonts = fonts.to_vec();
            fonts.push(*font);
            layout_with_fonts(body, &fonts)
        }
        MathNode::DisplayStyle => MathBox::text(String::new()),
        MathNode::LineBreak => MathBox::text(String::new()),
    }
}

fn layout_sequence(nodes: &[MathNode], fonts: &[MathFont]) -> MathBox {
    if !nodes.iter().any(|node| matches!(node, MathNode::LineBreak)) {
        return layout_horizontal_sequence(nodes, fonts);
    }

    let mut rows = Vec::new();
    let mut start = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        if matches!(node, MathNode::LineBreak) {
            rows.push(layout_horizontal_sequence(&nodes[start..index], fonts));
            start = index + 1;
        }
    }
    rows.push(layout_horizontal_sequence(&nodes[start..], fonts));
    vertical_rows(rows)
}

fn layout_horizontal_sequence(nodes: &[MathNode], fonts: &[MathFont]) -> MathBox {
    let mut boxes = Vec::with_capacity(nodes.len().saturating_mul(2));
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 && needs_operator_spacing(&nodes[index - 1], node) {
            boxes.push(MathBox::text(" "));
        }
        boxes.push(layout_with_fonts(node, fonts));
    }
    horizontal(&boxes)
}

fn horizontal(boxes: &[MathBox]) -> MathBox {
    if boxes.is_empty() {
        return MathBox::text(String::new());
    }
    let baseline = boxes.iter().map(|item| item.baseline).max().unwrap_or(0);
    let below = boxes
        .iter()
        .map(|item| item.lines.len().saturating_sub(item.baseline + 1))
        .max()
        .unwrap_or(0);
    let height = baseline + below + 1;
    let width = boxes.iter().map(|item| item.width).sum();
    let mut lines = vec![String::new(); height];
    let mut preserved_trailing = vec![None; height];

    for item in boxes {
        let top = baseline.saturating_sub(item.baseline);
        for (row, output) in lines.iter_mut().enumerate() {
            if let Some(content) = row.checked_sub(top).and_then(|index| item.lines.get(index)) {
                let item_index = row - top;
                output.push_str(content);
                let padding = item.width.saturating_sub(display_width(content));
                output.push_str(&" ".repeat(padding));
                if let Some(trailing) = &item.preserved_trailing[item_index] {
                    preserved_trailing[row] = Some(trailing.clone());
                } else if content.chars().any(|character| !character.is_whitespace()) {
                    preserved_trailing[row] = None;
                }
            } else {
                output.push_str(&" ".repeat(item.width));
            }
        }
    }

    MathBox {
        lines,
        preserved_trailing,
        width,
        baseline,
    }
}

fn vertical_rows(rows: Vec<MathBox>) -> MathBox {
    let width = rows.iter().map(|row| row.width).max().unwrap_or(0);
    let mut lines = Vec::new();
    let mut preserved_trailing = Vec::new();
    for row in rows {
        preserved_trailing.extend(row.preserved_trailing);
        for line in row.lines {
            lines.push(center(&line, width));
        }
    }
    MathBox {
        baseline: lines.len() / 2,
        lines,
        preserved_trailing,
        width,
    }
}

fn layout_fraction(
    numerator: &MathNode,
    denominator: &MathNode,
    style: FractionStyle,
    fonts: &[MathFont],
) -> MathBox {
    if style == FractionStyle::Inline {
        return MathBox::text(super::rendering::render_fraction_inline(
            numerator,
            denominator,
            fonts,
        ));
    }
    let numerator = layout_with_fonts(numerator, fonts);
    let denominator = layout_with_fonts(denominator, fonts);
    let width = numerator.width.max(denominator.width).max(1);
    let numerator_height = numerator.lines.len();
    let mut preserved_trailing = numerator.preserved_trailing;
    let mut lines = numerator
        .lines
        .iter()
        .map(|line| center(line, width))
        .collect::<Vec<_>>();
    lines.push("─".repeat(width));
    preserved_trailing.push(None);
    lines.extend(denominator.lines.iter().map(|line| center(line, width)));
    preserved_trailing.extend(denominator.preserved_trailing);
    MathBox {
        lines,
        preserved_trailing,
        width,
        baseline: numerator_height,
    }
}

fn delimit(mut body: MathBox, left: &str, right: &str) -> MathBox {
    let height = body.lines.len();
    for (index, line) in body.lines.iter_mut().enumerate() {
        *line = format!(
            "{}{}{}",
            delimiter_row(left, index, height, true),
            line,
            delimiter_row(right, index, height, false)
        );
    }
    if !right.is_empty() {
        body.preserved_trailing.fill(None);
    }
    body.width += display_width(left) + display_width(right);
    body
}

fn delimiter_row(delimiter: &str, row: usize, height: usize, left: bool) -> &str {
    if delimiter.is_empty() {
        return "";
    }
    if height <= 1 {
        return delimiter;
    }
    let position = if row == 0 {
        0
    } else if row + 1 == height {
        2
    } else {
        1
    };
    match (delimiter, left, position) {
        ("(", true, 0) => "⎛",
        ("(", true, 1) => "⎜",
        ("(", true, 2) => "⎝",
        (")", false, 0) => "⎞",
        (")", false, 1) => "⎟",
        (")", false, 2) => "⎠",
        ("[", true, 0) => "⎡",
        ("[", true, 1) => "⎢",
        ("[", true, 2) => "⎣",
        ("]", false, 0) => "⎤",
        ("]", false, 1) => "⎥",
        ("]", false, 2) => "⎦",
        ("{", true, 0) => "⎧",
        ("{", true, 1) => "⎨",
        ("{", true, 2) => "⎩",
        ("}", false, 0) => "⎫",
        ("}", false, 1) => "⎬",
        ("}", false, 2) => "⎭",
        ("|", _, _) => "│",
        ("‖", _, _) => "‖",
        _ => delimiter,
    }
}

fn center(text: &str, width: usize) -> String {
    let content_width = display_width(text);
    let remaining = width.saturating_sub(content_width);
    format!(
        "{}{}{}",
        " ".repeat(remaining / 2),
        text,
        " ".repeat(remaining - remaining / 2)
    )
}

mod decorations;
mod environment;
mod scripts;
mod structures;
use decorations::{layout_accent, layout_annotation, layout_brace};
use environment::layout_environment;
use scripts::layout_scripts;
use structures::{layout_binomial, layout_root};
