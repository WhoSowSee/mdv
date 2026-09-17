use super::*;

pub(super) fn dollar_math_ranges(content: &str, mut options: Options) -> Vec<Range<usize>> {
    options.remove(Options::ENABLE_TABLES);
    Parser::new_ext(content, options)
        .into_offset_iter()
        .filter_map(|(event, range)| {
            matches!(event, Event::InlineMath(_) | Event::DisplayMath(_)).then_some(range)
        })
        .collect()
}

pub(super) fn protect_math_pipes(
    content: &str,
    ranges: &[Range<usize>],
    placeholders: &MathPlaceholders,
) -> String {
    let mut output = String::with_capacity(content.len());
    let mut cursor = 0usize;

    for range in ranges {
        output.push_str(&content[cursor..range.start]);
        output.push_str(&content[range.clone()].replace('|', &placeholders.pipe));
        cursor = range.end;
    }
    output.push_str(&content[cursor..]);
    output
}
