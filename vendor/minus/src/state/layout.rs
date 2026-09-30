use super::{
    AppendStyle, Cow, EOF_SCROLL_MARGIN_ROWS, LineNumbers, PagerState, PromptError, TryInto,
    display, minus_core, screen, slice_chars, strip_ansi,
};

impl PagerState {
    pub(crate) fn content_columns(&self) -> usize {
        self.cols.saturating_sub(self.content_left()).max(1)
    }

    pub(crate) fn content_left(&self) -> usize {
        #[cfg(feature = "search")]
        if self.toc_visible() && self.cols >= 72 {
            return self.toc_width();
        }
        0
    }

    #[cfg(not(feature = "search"))]
    pub(crate) const fn toc_visible(&self) -> bool {
        false
    }

    pub(crate) fn reformat_display(&mut self) -> Result<(), PromptError> {
        let columns = self.content_columns();
        let format_result = screen::format_lines_into(
            &mut self.screen.formatted_lines,
            &self.screen.orig_text,
            self.line_numbers,
            columns,
            self.screen.line_wrapping,
            #[cfg(feature = "search")]
            self.search_state.search_term.as_ref(),
        );

        #[cfg(feature = "search")]
        {
            self.search_state.search_matches = format_result.search_matches;
            let first_match = self
                .search_state
                .search_matches
                .partition_point(|search_match| search_match.row < self.upper_mark);
            self.search_state.search_mark = if first_match == self.search_state.search_matches.len()
            {
                0
            } else {
                first_match
            };
        }
        self.lines_to_row_map = format_result.lines_to_row_map;
        self.screen.max_line_length = format_result.max_line_length;

        self.screen.unterminated = format_result.num_unterminated;
        self.format_prompt()
    }

    #[must_use]
    pub const fn content_rows(&self) -> usize {
        self.rows
            .saturating_sub(self.prompt_panel_rows().saturating_add(1))
    }

    #[must_use]
    pub const fn prompt_panel_rows(&self) -> usize {
        let available = self.rows.saturating_sub(1);
        if self.prompt_panel.len() < available {
            self.prompt_panel.len()
        } else {
            available
        }
    }

    #[must_use]
    pub const fn max_upper_mark(&self) -> usize {
        let content_rows = self.content_rows();
        let available_trailing_rows = content_rows.saturating_sub(1);
        let trailing_rows = if available_trailing_rows < EOF_SCROLL_MARGIN_ROWS {
            available_trailing_rows
        } else {
            EOF_SCROLL_MARGIN_ROWS
        };
        self.screen
            .formatted_lines_count()
            .saturating_add(trailing_rows)
            .saturating_sub(content_rows)
    }

    pub(crate) const fn prompt_row(&self) -> usize {
        self.content_rows()
    }

    pub(super) fn horizontal_scroll_view<'a>(&self, row: &'a str) -> (Cow<'a, str>, usize) {
        let (first_end, second_start, second_end) = display::get_horizontal_scroll_bounds(
            row,
            self.content_columns(),
            self.left_mark,
            self.line_numbers.is_on(),
            self.screen.line_count(),
        );
        let skipped_chars = self.horizontal_scroll_char_offset(row);

        if self.left_mark < row.len() {
            if self.line_numbers.is_on() {
                (
                    format!("{}{}", &row[..first_end], &row[second_start..second_end]).into(),
                    skipped_chars,
                )
            } else {
                (row[second_start..second_end].into(), skipped_chars)
            }
        } else {
            (Cow::Borrowed(""), skipped_chars)
        }
    }

    pub(super) fn horizontal_scroll_char_offset(&self, row: &str) -> usize {
        let visible = strip_ansi(row);
        let visible_len = visible.chars().count();
        let content_start = self.line_number_padding().min(visible_len);
        let content = slice_chars(&visible, content_start, visible_len);
        let mut byte_offset = self.left_mark.min(content.len());
        while byte_offset < content.len() && !content.is_char_boundary(byte_offset) {
            byte_offset += 1;
        }
        content[..byte_offset].chars().count()
    }

    pub(crate) const fn line_number_padding(&self) -> usize {
        if self.line_numbers.is_on() {
            minus_core::utils::digits(self.screen.line_count()) + LineNumbers::EXTRA_PADDING + 2
        } else {
            0
        }
    }

    pub(super) fn wrapped_cols_available(&self) -> usize {
        if self.line_numbers.is_on() {
            self.content_columns()
                .saturating_sub(self.line_number_padding().saturating_add(1))
        } else {
            self.content_columns()
        }
    }

    pub(crate) fn append_str(&mut self, text: &str) -> Result<AppendStyle, PromptError> {
        let columns = self.content_columns();
        let old_lc = self.screen.line_count();
        let old_lc_dgts = minus_core::utils::digits(old_lc);
        let mut append_result = self.screen.push_screen_buf(
            text,
            self.line_numbers,
            columns.try_into().unwrap(),
            #[cfg(feature = "search")]
            self.search_state.search_term.as_ref(),
        );
        let new_lc = self.screen.line_count();
        let new_lc_dgts = minus_core::utils::digits(new_lc);
        let total_rows = self.screen.formatted_lines_count();
        #[cfg(feature = "search")]
        {
            let first_reformatted_row = total_rows.saturating_sub(append_result.rows_formatted);
            if !append_result.clean_append {
                self.search_state
                    .search_matches
                    .retain(|search_match| search_match.row < first_reformatted_row);
            }
            self.search_state
                .search_matches
                .extend(append_result.search_matches);
        }
        self.lines_to_row_map.append(
            &mut append_result.lines_to_row_map,
            append_result.clean_append,
        );

        if self.line_numbers.is_on() && (new_lc_dgts != old_lc_dgts && old_lc_dgts != 0) {
            self.reformat_display()?;
            return Ok(AppendStyle::FullRedraw);
        }

        self.format_prompt()?;
        Ok(AppendStyle::PartialUpdate((
            total_rows - append_result.rows_formatted,
            total_rows,
        )))
    }
}
