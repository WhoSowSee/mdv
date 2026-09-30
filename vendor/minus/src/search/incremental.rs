use super::{
    ANSI_REGEX, Cow, IncrementalPreview, IncrementalSearchOpts, Regex, SearchOpts, SearchRange,
    Write, display, highlight_search_matches, screen,
};

pub fn search_ranges(line: &str, query: &Regex) -> Vec<SearchRange> {
    let stripped = ANSI_REGEX.replace_all(line, "");
    query
        .find_iter(&stripped)
        .map(|matched| {
            let start = stripped[..matched.start()].chars().count();
            let end = start + stripped[matched.start()..matched.end()].chars().count();
            SearchRange { start, end }
        })
        .collect()
}

pub(super) fn preview_line<'a>(
    iso: &IncrementalSearchOpts<'a>,
    query: &Regex,
    line_idx: usize,
    line: &'a str,
    visible_lines: &mut Vec<Cow<'a, str>>,
    upper_mark: &mut Option<usize>,
    wrapped: bool,
) {
    if upper_mark.is_none() && !query.is_match(&ANSI_REGEX.replace_all(line, "")) {
        return;
    }

    let selecting_current = upper_mark.is_none();
    let row_start = *iso.lines_to_row_map.get(line_idx).unwrap_or(&0);
    let mut match_row_idx = None;
    let formatted_rows = screen::format_line(
        line,
        iso.line_number_digits(),
        line_idx,
        iso.line_numbers,
        iso.cols,
        iso.screen.line_wrapping,
    );

    let formatted_rows = screen::rows_with_search_ranges(formatted_rows, Some(query))
        .enumerate()
        .map(|(i, (sfr, ranges))| {
            let absolute_row = row_start + i;
            if ranges.is_empty() {
                (
                    iso.screen.formatted_lines.get(absolute_row).map_or_else(
                        || Cow::Owned(sfr.to_string()),
                        |s| Cow::Borrowed(s.as_str()),
                    ),
                    ranges,
                    absolute_row,
                )
            } else {
                if wrapped || absolute_row >= iso.initial_upper_mark {
                    match_row_idx = Some(absolute_row);
                }
                (Cow::Owned(sfr.to_string()), ranges, absolute_row)
            }
        })
        .collect::<Vec<_>>();

    let mut formatted_rows = formatted_rows
        .into_iter()
        .map(|(row, ranges, absolute_row)| {
            if ranges.is_empty() {
                return row;
            }
            let current_range = if selecting_current && match_row_idx == Some(absolute_row) {
                ranges.first().copied()
            } else {
                None
            };
            if iso.output_styling {
                Cow::Owned(highlight_search_matches(
                    &row,
                    query,
                    current_range,
                    iso.content_start_chars(),
                    iso.color_depth,
                ))
            } else {
                row
            }
        })
        .collect::<Vec<_>>();

    if upper_mark.is_none() {
        if match_row_idx.is_none() {
            return;
        }
        let match_row_idx = match_row_idx.unwrap();
        let skip_rows = match_row_idx.saturating_sub(row_start);
        *upper_mark = Some(match_row_idx);
        visible_lines.extend(formatted_rows.drain(skip_rows..));
    } else {
        visible_lines.append(&mut formatted_rows);
    }

    if visible_lines.len() >= iso.writable_rows {
        visible_lines.truncate(iso.writable_rows);
    }
}

pub(super) fn incremental_preview<'a>(
    iso: &IncrementalSearchOpts<'a>,
    query: &'a Regex,
) -> Option<IncrementalPreview<'a>> {
    if iso.writable_rows == 0 {
        return None;
    }

    let start_line_idx = iso
        .lines_to_row_map
        .row_to_line(iso.initial_upper_mark)?
        .saturating_sub(1);

    let mut visible_lines: Vec<Cow<str>> = Vec::with_capacity(iso.writable_rows);
    let mut upper_mark = None;
    let mut viewport_upper_mark = None;

    for (line_idx, line) in iso
        .screen
        .orig_text
        .lines()
        .enumerate()
        .skip(start_line_idx)
    {
        preview_line(
            iso,
            query,
            line_idx,
            line,
            &mut visible_lines,
            &mut upper_mark,
            false,
        );
        if visible_lines.len() >= iso.writable_rows {
            break;
        }
    }

    // Backfill near-EOF matches so the preview still occupies a full viewport.
    if let Some(um) = upper_mark
        && visible_lines.len() < iso.writable_rows
    {
        let start = iso
            .screen
            .formatted_lines_count()
            .saturating_sub(iso.writable_rows);
        let to_insert = um.saturating_sub(start);
        let shift = visible_lines.len();

        visible_lines.extend(
            iso.screen
                .formatted_lines
                .iter()
                .skip(start)
                .take(to_insert)
                .map(Into::into),
        );
        visible_lines.rotate_left(shift);
        viewport_upper_mark = Some(start);
    }

    if upper_mark.is_none() {
        for (line_idx, line) in iso
            .screen
            .orig_text
            .lines()
            .enumerate()
            .take(start_line_idx)
        {
            preview_line(
                iso,
                query,
                line_idx,
                line,
                &mut visible_lines,
                &mut upper_mark,
                true,
            );
            if visible_lines.len() >= iso.writable_rows {
                break;
            }
        }
    }

    Some(IncrementalPreview {
        rows: visible_lines,
        upper_mark: viewport_upper_mark.or(upper_mark)?,
    })
}

pub(super) fn run_incremental_search<'a, F, O>(
    out: &mut O,
    so: &'a SearchOpts<'a>,
    incremental_search_condition: F,
) -> crate::Result<Option<usize>>
where
    O: Write,
    F: Fn(&'a SearchOpts) -> bool,
{
    let Some(iso) = so.incremental_search_options.as_ref() else {
        return Ok(None);
    };
    let screen = iso.screen;
    let line_numbers = iso.line_numbers;
    let initial_upper_mark = iso.initial_upper_mark;
    let initial_left_mark = iso.initial_left_mark;

    let should_proceed = so.compiled_regex.is_some() && incremental_search_condition(so);

    // Failed or disabled previews restore the exact pre-search viewport.
    let reset_screen = |out: &mut O, so: &SearchOpts<'_>| -> crate::Result {
        display::write_text_checked(
            out,
            &screen.formatted_lines,
            initial_upper_mark,
            so.rows.into(),
            so.cols.into(),
            screen.line_wrapping,
            initial_left_mark,
            line_numbers,
            screen.line_count(),
            iso.color_depth,
        )?;
        Ok(())
    };

    if !should_proceed {
        reset_screen(out, so)?;
        return Ok(None);
    }

    let query = so.compiled_regex.as_ref().unwrap();

    let Some(preview) = incremental_preview(iso, query) else {
        reset_screen(out, so)?;
        return Ok(None);
    };

    display::write_text_checked(
        out,
        &preview.rows,
        0,
        so.rows.into(),
        so.cols.into(),
        iso.screen.line_wrapping,
        iso.initial_left_mark,
        iso.line_numbers,
        iso.screen.line_count(),
        iso.color_depth,
    )?;

    Ok(Some(preview.upper_mark))
}
