#[cfg(feature = "search")]
use super::highlight_search_matches;
use super::{
    Cow, PagerState, Selection, char_index_at_display_column, grapheme_end_char_index,
    highlight_visible_range, slice_chars, strip_ansi,
};

impl PagerState {
    pub(crate) fn selection_from_coordinates(&self, x: u16, y: u16) -> Option<Selection> {
        let writable_rows = self.content_rows();
        let row_count = self.screen.formatted_lines_count();

        if row_count == 0 || usize::from(y) >= writable_rows {
            return None;
        }

        let absolute_row = self
            .upper_mark
            .saturating_add(usize::from(y))
            .min(row_count - 1);
        let raw_row = self.screen.formatted_lines.get(absolute_row)?;
        let prefix_width = self.line_number_padding();
        let (displayed_row, skipped_content_chars) = if self.screen.line_wrapping {
            (Cow::Borrowed(raw_row.as_str()), 0)
        } else {
            self.horizontal_scroll_view(raw_row)
        };
        let visible_row = strip_ansi(&displayed_row);
        let prefix_chars = char_index_at_display_column(&visible_row, prefix_width);
        let col = char_index_at_display_column(
            &visible_row,
            usize::from(x).saturating_sub(self.content_left()),
        )
        .saturating_sub(prefix_chars)
        .saturating_add(skipped_content_chars);

        Some(Selection { absolute_row, col })
    }

    pub(crate) fn select_all(&mut self) {
        self.clear_selection();
        if let Some(absolute_row) = self.screen.formatted_lines_count().checked_sub(1) {
            self.selection_anchor = Some(Selection {
                absolute_row: 0,
                col: 0,
            });
            self.selection = Some(Selection {
                absolute_row,
                col: strip_ansi(&self.screen.formatted_lines[absolute_row])
                    .chars()
                    .count(),
            });
        }
    }

    pub(crate) const fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
    }

    pub(crate) fn selection_row_span(&self) -> Option<(usize, usize)> {
        let (start, end) = self.normalized_selection()?;
        Some((start.absolute_row, end.absolute_row))
    }

    /// Omits ANSI and OSC sequences from the active selection.
    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.normalized_selection()?;
        let start_line = self.lines_to_row_map.row_to_line(start.absolute_row)?;
        let end_line = self.lines_to_row_map.row_to_line(end.absolute_row)?;
        let mut lines = self.screen.orig_text.lines().skip(start_line);

        let mut selected = Vec::with_capacity(end_line.saturating_sub(start_line) + 1);
        for line_idx in start_line..=end_line {
            let line = strip_ansi(lines.next()?);
            let line = line.as_ref();
            let line_len = line.chars().count();
            let start_col = if line_idx == start_line {
                self.selection_col_in_line(start, line_idx, line)
                    .min(line_len)
            } else {
                0
            };
            let end_col = if line_idx == end_line {
                let end_char = self
                    .selection_col_in_line(end, line_idx, line)
                    .min(line_len);
                grapheme_end_char_index(line, end_char)
            } else {
                line_len
            };

            selected.push(slice_chars(line, start_col, end_col).to_string());
        }

        Some(selected.join("\n"))
    }

    pub(crate) fn render_rows_for_display(&self, start: usize, end: usize) -> Vec<Cow<'_, str>> {
        #[cfg(feature = "search")]
        let current_search_match = self
            .search_state
            .search_matches
            .get(self.search_state.search_mark)
            .map(|search_match| {
                (
                    search_match.row,
                    search_match.range.start,
                    search_match.range.end,
                )
            });
        #[cfg(not(feature = "search"))]
        let current_search_match = None;

        (start..end)
            .filter_map(|absolute_row| {
                self.render_row_for_display(absolute_row, current_search_match)
            })
            .collect()
    }

    pub(super) fn render_row_for_display(
        &self,
        absolute_row: usize,
        current_search_match: Option<(usize, usize, usize)>,
    ) -> Option<Cow<'_, str>> {
        #[cfg(not(feature = "search"))]
        let _ = current_search_match;
        let raw_row = self.screen.formatted_lines.get(absolute_row)?;
        let prefix_width = self.line_number_padding();
        let (row, skipped_chars) = if self.screen.line_wrapping {
            (Cow::Borrowed(raw_row.as_str()), 0)
        } else {
            self.horizontal_scroll_view(raw_row)
        };
        #[cfg(feature = "search")]
        let row = if self.output_styling && self.source_line_is_highlighted(absolute_row) {
            Cow::Owned(crate::search::highlight_line_navigation_target(
                &row,
                prefix_width,
                self.color_depth,
            ))
        } else {
            row
        };
        let row = if self.output_styling
            && let Some((start_col, end_col)) = self.selection_bounds_for_row(absolute_row)
        {
            let visible_start = start_col.saturating_sub(skipped_chars);
            let visible_end = end_col.saturating_sub(skipped_chars);
            highlight_visible_range(
                row,
                prefix_width.saturating_add(visible_start),
                prefix_width.saturating_add(visible_end),
                self.color_depth,
                self.highlight_styles.selection,
            )
        } else {
            row
        };

        #[cfg(feature = "search")]
        let first_match_on_row = self
            .search_state
            .search_matches
            .partition_point(|search_match| search_match.row < absolute_row);
        #[cfg(feature = "search")]
        let row = if self.output_styling
            && self
                .search_state
                .search_matches
                .get(first_match_on_row)
                .is_some_and(|search_match| search_match.row == absolute_row)
        {
            let search_term = self.search_state.search_term.as_ref()?;
            let current_range = current_search_match
                .filter(|(row, _, _)| *row == absolute_row)
                .and_then(|(_, start, end)| {
                    crate::search::SearchRange { start, end }.after_skipping(skipped_chars)
                });
            Cow::Owned(highlight_search_matches(
                &row,
                search_term,
                current_range,
                prefix_width,
                self.color_depth,
                self.highlight_styles,
            ))
        } else {
            row
        };

        let row = self.color_depth.adapt(row);
        if self.content_left() > 0 {
            Some(format!("{}{}", " ".repeat(self.content_left()), row).into())
        } else {
            Some(row)
        }
    }

    pub(super) fn selection_bounds_for_row(&self, absolute_row: usize) -> Option<(usize, usize)> {
        let (start, end) = self.normalized_selection()?;

        if absolute_row < start.absolute_row || absolute_row > end.absolute_row {
            return None;
        }

        let start_col = if absolute_row == start.absolute_row {
            start.col
        } else {
            0
        };
        let end_col = if absolute_row == end.absolute_row {
            end.col.saturating_add(1)
        } else {
            usize::MAX
        };
        Some((start_col, end_col))
    }

    pub(super) fn normalized_selection(&self) -> Option<(Selection, Selection)> {
        let s_start = self.selection_anchor?;
        let s_end = self.selection?;

        Some(
            if s_start.absolute_row > s_end.absolute_row
                || (s_start.absolute_row == s_end.absolute_row && s_start.col > s_end.col)
            {
                (s_end, s_start)
            } else {
                (s_start, s_end)
            },
        )
    }

    pub(super) fn selection_col_in_line(
        &self,
        selection: Selection,
        line_idx: usize,
        line: &str,
    ) -> usize {
        if !self.screen.line_wrapping {
            return selection.col;
        }

        let Some(&line_start_row) = self.lines_to_row_map.get(line_idx) else {
            return selection.col;
        };
        let row_in_line = selection.absolute_row.saturating_sub(line_start_row);
        let cols_avail = self.wrapped_cols_available();
        let wrapped_rows = textwrap::wrap(line, cols_avail.max(1));
        let preceding_chars = wrapped_rows
            .iter()
            .take(row_in_line)
            .map(|row| row.chars().count())
            .sum::<usize>();

        preceding_chars.saturating_add(selection.col)
    }
}
