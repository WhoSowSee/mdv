use pulldown_cmark::Event;
use std::collections::HashMap;

const EVENT_PREFIX: &str = "\u{001d}MDV_SOURCE_LINE:";
const BLANK_EVENT_PREFIX: &str = "\u{001d}MDV_SOURCE_BLANK_LINE:";
const EVENT_SUFFIX: char = '\u{001e}';

pub(super) fn event(line: usize) -> Event<'static> {
    Event::InlineHtml(format!("{EVENT_PREFIX}{line}{EVENT_SUFFIX}").into())
}

pub(super) fn blank_event(line: usize) -> Event<'static> {
    Event::InlineHtml(format!("{BLANK_EVENT_PREFIX}{line}{EVENT_SUFFIX}").into())
}

#[derive(Clone, Copy)]
pub(crate) enum Marker {
    Content(usize),
    Blank(usize),
}

pub(crate) fn from_event(event: &Event<'_>) -> Option<Marker> {
    parse_event(event, EVENT_PREFIX)
        .map(Marker::Content)
        .or_else(|| parse_event(event, BLANK_EVENT_PREFIX).map(Marker::Blank))
}

fn parse_event(event: &Event<'_>, prefix: &str) -> Option<usize> {
    let Event::InlineHtml(marker) = event else {
        return None;
    };

    marker
        .strip_prefix(prefix)?
        .strip_suffix(EVENT_SUFFIX)?
        .parse()
        .ok()
}

pub(super) fn apply_transform<F>(
    before: String,
    source_lines: Option<&mut Vec<Option<usize>>>,
    transform: F,
) -> String
where
    F: FnOnce(&str) -> String,
{
    let after = transform(&before);
    if after != before
        && let Some(source_lines) = source_lines
    {
        *source_lines = remap(&before, &after, source_lines);
    }
    after
}

pub(super) fn starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(
        content
            .bytes()
            .enumerate()
            .filter_map(|(idx, byte)| (byte == b'\n').then_some(idx + 1)),
    );
    starts
}

pub(super) fn index_for_offset(line_starts: &[usize], offset: usize) -> usize {
    line_starts
        .partition_point(|line_start| *line_start <= offset)
        .saturating_sub(1)
}

fn remap(before: &str, after: &str, source_lines: &[Option<usize>]) -> Vec<Option<usize>> {
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    let before_positions = line_positions(&before_lines);
    let after_positions = line_positions(&after_lines);
    let mut remapped = Vec::with_capacity(after_lines.len());
    let mut before_idx = 0usize;
    let mut after_idx = 0usize;

    while before_idx < before_lines.len() && after_idx < after_lines.len() {
        if before_lines[before_idx] == after_lines[after_idx] {
            remapped.push(source_lines.get(before_idx).copied().flatten());
            before_idx += 1;
            after_idx += 1;
            continue;
        }

        let insertion_distance = next_index(&after_positions, before_lines[before_idx], after_idx)
            .map(|idx| idx - after_idx);
        let deletion_distance = next_index(&before_positions, after_lines[after_idx], before_idx)
            .map(|idx| idx - before_idx);

        match (insertion_distance, deletion_distance) {
            (Some(insertions), None) => {
                remapped.extend(std::iter::repeat_n(None, insertions));
                after_idx += insertions;
            }
            (None, Some(deletions)) => before_idx += deletions,
            (Some(insertions), Some(deletions)) if insertions < deletions => {
                remapped.extend(std::iter::repeat_n(None, insertions));
                after_idx += insertions;
            }
            (Some(insertions), Some(deletions)) if deletions < insertions => {
                before_idx += deletions;
            }
            _ => {
                remapped.push(source_lines.get(before_idx).copied().flatten());
                before_idx += 1;
                after_idx += 1;
            }
        }
    }

    remapped.extend(std::iter::repeat_n(
        None,
        after_lines.len().saturating_sub(after_idx),
    ));
    remapped
}

fn line_positions<'a>(lines: &[&'a str]) -> HashMap<&'a str, Vec<usize>> {
    let mut positions = HashMap::new();
    for (idx, line) in lines.iter().copied().enumerate() {
        positions.entry(line).or_insert_with(Vec::new).push(idx);
    }
    positions
}

fn next_index(positions: &HashMap<&str, Vec<usize>>, line: &str, current: usize) -> Option<usize> {
    let indices = positions.get(line)?;
    indices
        .get(indices.partition_point(|idx| *idx <= current))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::remap;

    #[test]
    fn inserted_lines_have_no_source_number() {
        let mapped = remap("first\nsecond", "first\n\nsecond", &[Some(1), Some(2)]);

        assert_eq!(mapped, [Some(1), None, Some(2)]);
    }
}
