pub fn apply_clickable_link_replacements(
    mut table_output: String,
    replacements: &[(String, String)],
) -> String {
    let mut search_start = 0usize;

    for (plain, styled) in replacements {
        if plain.is_empty() {
            continue;
        }

        if let Some(rel_idx) = table_output[search_start..].find(plain) {
            let idx = search_start + rel_idx;
            let end = idx + plain.len();
            table_output.replace_range(idx..end, styled);
            search_start = idx + styled.len();
        } else {
            search_start =
                apply_fragmented_inline_style(&mut table_output, search_start, plain, styled)
                    .unwrap_or(search_start);
        }
    }

    table_output
}

fn apply_fragmented_inline_style(
    table_output: &mut String,
    search_start: usize,
    plain: &str,
    styled: &str,
) -> Option<usize> {
    let (prefix, suffix) = styled_wrapper(styled, plain)?;

    let mut candidate_output = table_output.clone();
    let mut plain_index = 0usize;
    let mut output_index = search_start;
    let mut replaced_count = 0usize;
    let mut resume_index = None;
    let mut expected_separator_count = None;

    while plain_index < plain.len() {
        let remaining = &plain[plain_index..];
        let (segment_pos, segment_len) = find_segment_in_output(
            &candidate_output,
            output_index,
            remaining,
            expected_separator_count,
        )?;
        let segment = &remaining[..segment_len];
        let styled_segment = format!("{}{}{}", prefix, segment, suffix);

        let end = segment_pos + segment_len;
        candidate_output.replace_range(segment_pos..end, &styled_segment);

        output_index = segment_pos + styled_segment.len();
        if resume_index.is_none() {
            resume_index = Some(output_index);
        }
        if expected_separator_count.is_none() {
            expected_separator_count =
                Some(line_separator_count_before(&candidate_output, segment_pos));
        }
        plain_index += segment_len;
        replaced_count += 1;
    }

    if replaced_count == 0 {
        return None;
    }

    *table_output = candidate_output;
    Some(resume_index.unwrap_or(output_index))
}

fn find_segment_in_output(
    output: &str,
    search_start: usize,
    remaining_plain: &str,
    expected_separator_count: Option<usize>,
) -> Option<(usize, usize)> {
    const MIN_SEGMENT_LEN: usize = 3;
    let mut best_match: Option<(usize, usize)> = None;

    for segment_len in prefix_lengths_desc(remaining_plain) {
        if segment_len < MIN_SEGMENT_LEN && segment_len != remaining_plain.len() {
            continue;
        }

        let segment = &remaining_plain[..segment_len];
        let mut lookup_start = search_start;
        while let Some(rel_idx) = output[lookup_start..].find(segment) {
            let segment_pos = lookup_start + rel_idx;
            if expected_separator_count
                .is_none_or(|expected| line_separator_count_before(output, segment_pos) == expected)
            {
                match best_match {
                    None => best_match = Some((segment_pos, segment_len)),
                    Some((best_pos, best_len)) => {
                        if segment_pos < best_pos
                            || (segment_pos == best_pos && segment_len > best_len)
                        {
                            best_match = Some((segment_pos, segment_len));
                        }
                    }
                }
                break;
            }

            lookup_start = segment_pos + 1;
        }
    }

    if best_match.is_some() {
        return best_match;
    }

    if remaining_plain.chars().count() == 1 {
        let segment_len = remaining_plain.len();
        let mut lookup_start = search_start;
        while let Some(rel_idx) = output[lookup_start..].find(remaining_plain) {
            let segment_pos = lookup_start + rel_idx;
            if expected_separator_count
                .is_none_or(|expected| line_separator_count_before(output, segment_pos) == expected)
            {
                return Some((segment_pos, segment_len));
            }

            lookup_start = segment_pos + 1;
        }
    }

    None
}

fn prefix_lengths_desc(input: &str) -> Vec<usize> {
    let mut lengths: Vec<usize> = input.char_indices().skip(1).map(|(idx, _)| idx).collect();
    lengths.push(input.len());
    lengths.sort_unstable();
    lengths.reverse();
    lengths
}

pub(super) fn styled_wrapper<'a>(styled: &'a str, plain: &str) -> Option<(&'a str, &'a str)> {
    // Prefer the last occurrence to support wrappers where `plain` may also
    // appear in metadata prefixes (e.g. OSC 8 URLs).
    let plain_pos = styled.rfind(plain)?;
    let plain_end = plain_pos + plain.len();
    Some((&styled[..plain_pos], &styled[plain_end..]))
}

fn line_separator_count_before(output: &str, position: usize) -> usize {
    let line_start = output[..position]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(0);
    output[line_start..position]
        .chars()
        .filter(|ch| matches!(ch, '│' | '┆' | '┃'))
        .count()
}
