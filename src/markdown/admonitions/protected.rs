use super::{Dialect, Line, Opening};
use crate::markdown::source_lines::{index_for_offset, starts};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use std::ops::Range;

pub(super) struct ProtectedLines {
    ends: Vec<Option<usize>>,
    boundary_ends: Vec<Option<usize>>,
}

impl ProtectedLines {
    pub fn new(lines: &[Line]) -> Self {
        let source = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let line_starts = starts(&source);
        let mut ranges: Vec<Range<usize>> = Vec::new();
        let mut boundary_ranges = Vec::new();
        let mut container_depth = 0usize;
        for (event, range) in Parser::new(&source).into_offset_iter() {
            match &event {
                Event::Start(Tag::BlockQuote(_) | Tag::List(_)) => container_depth += 1,
                Event::End(TagEnd::BlockQuote(_) | TagEnd::List(_)) => container_depth -= 1,
                _ => {}
            }
            let protect = match &event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    let line = &lines[index_for_offset(&line_starts, range.start)].text;
                    !Opening::parse(line.trim_start())
                        .is_some_and(|opening| opening.dialect == Dialect::Myst)
                }
                Event::Start(Tag::CodeBlock(CodeBlockKind::Indented) | Tag::HtmlBlock)
                | Event::Code(_) => true,
                _ => false,
            };
            if !protect {
                continue;
            }
            let mut start = index_for_offset(&line_starts, range.start);
            let end =
                (index_for_offset(&line_starts, range.end.saturating_sub(1)) + 1).min(lines.len());
            if matches!(event, Event::Code(_)) {
                let line = &lines[start].text;
                let marker_offset = line_starts[start] + line.len() - line.trim_start().len();
                // Inline code in a title must not protect the preceding callout opener.
                if marker_offset < range.start {
                    start += 1;
                }
            }
            if start < end {
                boundary_ranges.push(start..end);
                if container_depth == 0 || matches!(event, Event::Code(_)) {
                    ranges.push(start..end);
                }
            }
        }
        Self {
            ends: range_ends(ranges, lines.len()),
            boundary_ends: range_ends(boundary_ranges, lines.len()),
        }
    }

    pub fn end_at(&self, line: usize) -> Option<usize> {
        self.ends[line]
    }

    pub fn boundary_end_at(&self, line: usize) -> Option<usize> {
        self.boundary_ends[line]
    }
}

fn range_ends(mut ranges: Vec<Range<usize>>, line_count: usize) -> Vec<Option<usize>> {
    ranges.sort_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && range.start <= previous.end
        {
            previous.end = previous.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    let mut ends = vec![None; line_count];
    for range in merged {
        let end = range.end;
        ends[range].fill(Some(end));
    }
    ends
}
