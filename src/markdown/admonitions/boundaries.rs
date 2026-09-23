use super::{Dialect, Line, MarkdownProcessor, Opening, Terminator, protected::ProtectedLines};
use std::collections::HashSet;

pub(super) type BlockEnd = (usize, usize);

pub(super) fn index(
    lines: &[Line],
    protected: &ProtectedLines,
) -> Vec<Option<(Opening, BlockEnd)>> {
    let openings: Vec<_> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            protected
                .boundary_end_at(index)
                .is_none()
                .then(|| Opening::parse(line.text.trim_start()))
                .flatten()
        })
        .collect();
    let mut ends = vec![None; lines.len()];
    let mut closing_kinds = HashSet::new();
    for start in (0..lines.len()).rev() {
        if let Some(kind) = closing_kind(lines[start].text.trim()) {
            closing_kinds.insert(kind);
        }
        if let Some(opening) = &openings[start]
            && match opening.terminator {
                Terminator::Fence(marker, _) => closing_kinds.contains(&marker),
                Terminator::Gitbook => closing_kinds.contains(&'%'),
                Terminator::Indent => true,
            }
        {
            ends[start] = block_end(lines, start, opening, protected, &openings, &ends);
        }
    }
    openings
        .into_iter()
        .zip(ends)
        .map(|(opening, end)| opening.zip(end))
        .collect()
}

fn block_end(
    lines: &[Line],
    start: usize,
    opening: &Opening,
    protected: &ProtectedLines,
    openings: &[Option<Opening>],
    ends: &[Option<BlockEnd>],
) -> Option<BlockEnd> {
    let base_indent = MarkdownProcessor::leading_indent_columns(&lines[start].text);
    if matches!(opening.terminator, Terminator::Indent) {
        let first = (start + 1..lines.len()).find(|index| !lines[*index].text.trim().is_empty());
        let indented = first.is_some_and(|index| {
            MarkdownProcessor::leading_indent_columns(&lines[index].text) >= base_indent + 4
        });
        let legacy = opening.dialect == Dialect::Mkdocs
            && lines[start].text.trim_start().starts_with('!')
            && first == Some(start + 1)
            && !indented;
        let mut end = start + 1;
        while end < lines.len() {
            let text = &lines[end].text;
            if legacy {
                if text.trim().is_empty() || openings[end].is_some() {
                    break;
                }
            } else if !text.trim().is_empty()
                && MarkdownProcessor::leading_indent_columns(text) < base_indent + 4
            {
                break;
            }
            end += 1;
        }
        return Some((end, end));
    }
    let mut index = start + 1;
    while index < lines.len() {
        if let Some(end) = protected.boundary_end_at(index) {
            index = end;
            continue;
        }
        let text = lines[index].text.trim();
        if MarkdownProcessor::leading_indent_columns(&lines[index].text) <= base_indent + 3 {
            if closes(text, opening.terminator) {
                return Some((index, index + 1));
            }
            if let Some((_, next)) = ends[index] {
                if openings[index]
                    .as_ref()
                    .is_some_and(|nested| matches!(nested.terminator, Terminator::Indent))
                    && let Some(end) = closing_before(
                        lines,
                        index + 1..next,
                        opening.terminator,
                        base_indent + 3,
                        protected,
                    )
                {
                    return Some((end, end + 1));
                }
                index = next;
                continue;
            }
            if let Some((marker, count)) = super::MarkdownProcessor::detect_fence_marker(text) {
                index = (index + 1..lines.len())
                    .find(|next| {
                        MarkdownProcessor::leading_indent_columns(&lines[*next].text)
                            <= base_indent + 3
                            && closes(lines[*next].text.trim(), Terminator::Fence(marker, count))
                    })
                    .map_or(lines.len(), |end| end + 1);
                continue;
            }
        }
        index += 1;
    }
    None
}

fn closes(text: &str, terminator: Terminator) -> bool {
    match terminator {
        Terminator::Fence(marker, count) => {
            text.chars().count() >= count && text.chars().all(|ch| ch == marker)
        }
        Terminator::Gitbook => text
            .strip_prefix("{%")
            .and_then(|rest| rest.strip_suffix("%}"))
            .is_some_and(|rest| rest.trim() == "endhint"),
        Terminator::Indent => false,
    }
}

fn closing_before(
    lines: &[Line],
    range: std::ops::Range<usize>,
    terminator: Terminator,
    max_indent: usize,
    protected: &ProtectedLines,
) -> Option<usize> {
    let mut index = range.start;
    while index < range.end {
        if let Some(end) = protected.boundary_end_at(index) {
            index = end;
            continue;
        }
        let text = &lines[index].text;
        if MarkdownProcessor::leading_indent_columns(text) <= max_indent
            && closes(text.trim(), terminator)
        {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn closing_kind(text: &str) -> Option<char> {
    if closes(text, Terminator::Gitbook) {
        return Some('%');
    }
    let marker = text.chars().next()?;
    (matches!(marker, ':' | '/' | '`' | '~') && closes(text, Terminator::Fence(marker, 3)))
        .then_some(marker)
}
