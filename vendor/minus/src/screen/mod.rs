//! Provides functions for getting analysis of the text data inside minus.
//!
//! This module is still a work is progress and is subject to change.
use crate::{
    LineNumbers,
    minus_core::{self, utils::LinesRowMap},
};
#[cfg(feature = "search")]
use regex::Regex;

use std::{borrow::Cow, fmt};

#[cfg(feature = "search")]
use {crate::search, std::collections::BTreeSet};

pub type Row = String;
pub type Rows = Vec<String>;
pub type Line<'a> = &'a str;
pub type TextBlock<'a> = &'a str;
pub type OwnedTextBlock = String;

pub(crate) struct FormattedRow<'a> {
    pub(crate) row: Cow<'a, str>,
    show_line_numbers: bool,
    line_number: Option<usize>,
    padding: usize,
}

impl FormattedRow<'_> {
    fn raw_row(&self) -> &str {
        &self.row
    }

    fn fmt_prefix(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.show_line_numbers {
            return Ok(());
        }

        match self.line_number {
            Some(line_number) => {
                let line_number = line_number + 1;
                let number_width = minus_core::utils::digits(line_number) + 1;
                let left_padding = self.padding.saturating_sub(number_width);

                write!(f, "{:left_padding$}", "")?;
                if cfg!(not(test)) {
                    write!(f, "{}", crossterm::style::Attribute::Bold)?;
                }
                write!(f, "{line_number}.")?;
                if cfg!(not(test)) {
                    write!(f, "{}", crossterm::style::Attribute::Reset)?;
                }
                f.write_str(" ")
            }
            None => write!(f, "{:>width$} ", "", width = self.padding),
        }
    }
}

impl fmt::Display for FormattedRow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_prefix(f)?;
        f.write_str(self.raw_row())
    }
}

#[cfg(feature = "search")]
pub(crate) struct SearchFormattedRow<'a, 'b> {
    row: FormattedRow<'a>,
    search_term: Option<&'b Regex>,
    is_match: bool,
}

#[cfg(feature = "search")]
impl fmt::Display for SearchFormattedRow<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.row.fmt_prefix(f)?;

        if self.is_match {
            write!(
                f,
                "{}",
                search::highlight_matches_args(
                    self.row.raw_row(),
                    self.search_term.unwrap(),
                    false
                )
            )
        } else {
            f.write_str(self.row.raw_row())
        }
    }
}

/// Cached terminal-ready representation of pager content.
pub struct Screen {
    pub(crate) orig_text: OwnedTextBlock,
    pub(crate) formatted_lines: Rows,
    pub(crate) line_count: usize,
    pub(crate) max_line_length: usize,
    /// Unterminated lines
    /// Keeps track of the number of lines at the last of [`Self::formatted_lines`] which are not
    /// terminated by a newline
    pub(crate) unterminated: usize,
    /// Whether to Line wrap lines
    ///
    /// Its negation gives the state of whether horizontal scrolling is allowed.
    pub(crate) line_wrapping: bool,
}

impl Screen {
    /// Get the actual number of physical rows that the text that will actually occupy on the
    /// terminal
    #[must_use]
    pub const fn formatted_lines_count(&self) -> usize {
        self.formatted_lines.len()
    }

    /// Get the number of [`Lines`](std::str::Lines) in the text.
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Get the length of the longest [Line] in the text.
    #[must_use]
    pub const fn get_max_line_length(&self) -> usize {
        self.max_line_length
    }

    pub(crate) fn push_screen_buf(
        &mut self,
        text: TextBlock,
        line_numbers: LineNumbers,
        cols: u16,
        #[cfg(feature = "search")] search_term: Option<&Regex>,
    ) -> FormatResult {
        // An unterminated tail becomes the attachment for the incoming first line.
        let clean_append = self.orig_text.ends_with('\n') || self.orig_text.is_empty();
        let old_lc = self.line_count();

        let formatted_lines_count = self.formatted_lines.len();

        // Reformat the previous tail when this block completes an unterminated line.
        self.formatted_lines
            .truncate(self.formatted_lines.len() - self.unterminated);

        let append_props = {
            let attachment = if clean_append {
                None
            } else {
                self.orig_text.lines().last()
            };

            let append_opts = FormatOpts {
                buffer: &mut self.formatted_lines,
                text,
                attachment,
                line_numbers,
                formatted_lines_count,
                lines_count: old_lc,
                prev_unterminated: self.unterminated,
                cols: cols.into(),
                line_wrapping: self.line_wrapping,
                #[cfg(feature = "search")]
                search_term,
            };
            format_text_block(append_opts)
        };
        self.orig_text.push_str(text);

        let (num_unterminated, lines_formatted, max_line_length) = (
            append_props.num_unterminated,
            append_props.lines_formatted,
            append_props.max_line_length,
        );

        self.line_count = old_lc + lines_formatted.saturating_sub(usize::from(!clean_append));
        if max_line_length > self.max_line_length {
            self.max_line_length = max_line_length;
        }

        self.unterminated = num_unterminated;
        append_props
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            line_wrapping: true,
            orig_text: String::with_capacity(100 * 1024),
            formatted_lines: Vec::with_capacity(500 * 1024),
            line_count: 0,
            max_line_length: 0,
            unterminated: 0,
        }
    }
}

// A block contains lines; wrapping maps them to rows, while unterminated tails are replaced on append.

pub(crate) trait AppendableBuffer {
    fn push_fmt<D>(&mut self, row: D)
    where
        D: fmt::Display;
}

impl AppendableBuffer for Rows {
    fn push_fmt<D>(&mut self, row: D)
    where
        D: fmt::Display,
    {
        self.push(row.to_string());
    }
}

impl AppendableBuffer for &mut Rows {
    fn push_fmt<D>(&mut self, row: D)
    where
        D: fmt::Display,
    {
        self.push(row.to_string());
    }
}

pub(crate) struct ReusableRows<'a> {
    rows: &'a mut Rows,
    used: usize,
}

impl<'a> ReusableRows<'a> {
    pub(crate) const fn new(rows: &'a mut Rows) -> Self {
        Self { rows, used: 0 }
    }

    pub(crate) fn finish(self) {
        self.rows.truncate(self.used);
    }
}

impl AppendableBuffer for &mut ReusableRows<'_> {
    fn push_fmt<D>(&mut self, row: D)
    where
        D: fmt::Display,
    {
        if self.used == self.rows.len() {
            self.rows.push(String::new());
        }

        let slot = &mut self.rows[self.used];
        slot.clear();
        fmt::write(slot, format_args!("{row}")).unwrap();
        self.used += 1;
    }
}

pub(crate) struct FormatOpts<'a, B>
where
    B: AppendableBuffer,
{
    pub buffer: B,
    pub text: TextBlock<'a>,
    pub attachment: Option<TextBlock<'a>>,
    pub line_numbers: LineNumbers,
    pub lines_count: usize,
    pub formatted_lines_count: usize,
    pub cols: usize,
    pub prev_unterminated: usize,
    #[cfg(feature = "search")]
    pub search_term: Option<&'a regex::Regex>,

    pub line_wrapping: bool,
}

#[derive(Debug)]
pub(crate) struct FormatResult {
    pub lines_formatted: usize,
    pub rows_formatted: usize,
    pub num_unterminated: usize,
    #[cfg(feature = "search")]
    pub append_search_idx: BTreeSet<usize>,
    pub lines_to_row_map: LinesRowMap,
    pub max_line_length: usize,
    pub clean_append: bool,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn format_text_block<B>(mut opts: FormatOpts<'_, B>) -> FormatResult
where
    B: AppendableBuffer,
{
    // Attachments merge an unterminated tail before row spans and tracking metadata are recomputed.
    let to_format = if let Some(attached_text) = opts.attachment {
        opts.lines_count = opts.lines_count.saturating_sub(1);
        opts.formatted_lines_count = opts
            .formatted_lines_count
            .saturating_sub(opts.prev_unterminated);
        let mut s = String::with_capacity(opts.text.len() + attached_text.len());
        s.push_str(attached_text);
        s.push_str(opts.text);

        s
    } else {
        opts.text.to_string()
    };

    let lines = to_format
        .lines()
        .enumerate()
        .collect::<Vec<(usize, &str)>>();

    let to_format_size = lines.len();

    let mut fr = FormatResult {
        lines_formatted: to_format_size,
        rows_formatted: 0,
        num_unterminated: opts.prev_unterminated,
        #[cfg(feature = "search")]
        append_search_idx: BTreeSet::new(),
        lines_to_row_map: LinesRowMap::new(),
        max_line_length: 0,
        clean_append: opts.attachment.is_none(),
    };

    let line_number_digits = minus_core::utils::digits(opts.lines_count + to_format_size);

    if lines.is_empty() {
        return fr;
    }

    let mut formatted_row_count = opts.formatted_lines_count;

    let (last_idx, last_line_text) = lines.last().copied().unwrap();
    for (idx, line) in lines.iter().take(lines.len().saturating_sub(1)) {
        fr.lines_to_row_map.insert(formatted_row_count, true);
        fr.max_line_length = fr.max_line_length.max(line.len());

        let rows = format_line(
            line,
            line_number_digits,
            opts.lines_count + idx,
            opts.line_numbers,
            opts.cols,
            opts.line_wrapping,
        );

        #[cfg(feature = "search")]
        let rows = format_search_rows(rows, opts.search_term);

        #[cfg(feature = "search")]
        {
            formatted_row_count += collect_rows(
                &mut opts.buffer,
                rows,
                formatted_row_count,
                &mut fr.append_search_idx,
            );
        }

        #[cfg(not(feature = "search"))]
        {
            formatted_row_count += collect_rows(&mut opts.buffer, rows);
        }
    }

    let last_line = format_line(
        last_line_text,
        line_number_digits,
        opts.lines_count + last_idx,
        opts.line_numbers,
        opts.cols,
        opts.line_wrapping,
    );
    #[cfg(feature = "search")]
    let last_line = format_search_rows(last_line, opts.search_term);

    let last_line_rows = last_line.size_hint().1.unwrap();

    fr.lines_to_row_map.insert(formatted_row_count, true);
    fr.max_line_length = fr.max_line_length.max(last_line_text.len());

    #[cfg(feature = "search")]
    {
        formatted_row_count += collect_rows(
            &mut opts.buffer,
            last_line,
            formatted_row_count,
            &mut fr.append_search_idx,
        );
    }

    #[cfg(not(feature = "search"))]
    {
        formatted_row_count += collect_rows(&mut opts.buffer, last_line);
    }

    fr.num_unterminated = if opts.text.ends_with('\n') {
        0
    } else {
        last_line_rows
    };
    fr.rows_formatted = formatted_row_count - opts.formatted_lines_count;

    fr
}

pub(crate) fn format_line(
    line: Line<'_>,
    len_line_number: usize,
    line_number: usize,
    show_line_numbers: LineNumbers,
    cols: usize,
    line_wrapping: bool,
) -> impl Iterator<Item = FormattedRow<'_>> {
    assert!(
        !line.contains('\n'),
        "Newlines found in appending line {line:?}",
    );
    let line_numbers = matches!(
        show_line_numbers,
        LineNumbers::Enabled | LineNumbers::AlwaysOn
    );

    let padding = len_line_number + LineNumbers::EXTRA_PADDING + 1;

    let enumerated_rows = if line_wrapping {
        let cols_avail = if line_numbers {
            cols.saturating_sub(padding + 2)
        } else {
            cols
        };
        textwrap::wrap(line, cols_avail)
    } else {
        vec![Cow::from(line)]
    }
    .into_iter()
    .enumerate();

    enumerated_rows.map(move |(i, row)| FormattedRow {
        row,
        show_line_numbers: line_numbers,
        line_number: if line_numbers && i == 0 {
            Some(line_number)
        } else {
            None
        },
        padding,
    })
}

#[cfg(feature = "search")]
pub(crate) fn format_search_rows<'a>(
    rows: impl Iterator<Item = FormattedRow<'a>> + 'a,
    search_term: Option<&'a Regex>,
) -> impl Iterator<Item = (SearchFormattedRow<'a, 'a>, bool)> + 'a {
    rows.map(move |row| {
        let is_match = search_term.is_some_and(|st| st.is_match(row.raw_row()));
        (
            SearchFormattedRow {
                row,
                search_term,
                is_match,
            },
            is_match,
        )
    })
}

#[cfg(feature = "search")]
fn collect_rows<B, I, D>(
    buffer: &mut B,
    rows: I,
    formatted_idx: usize,
    search_idx: &mut BTreeSet<usize>,
) -> usize
where
    B: AppendableBuffer,
    I: IntoIterator<Item = (D, bool)>,
    D: fmt::Display,
{
    let mut row_count = 0;
    for (wrap_idx, (row, is_match)) in rows.into_iter().enumerate() {
        if is_match {
            search_idx.insert(formatted_idx + wrap_idx);
        }
        buffer.push_fmt(row);
        row_count = wrap_idx + 1;
    }
    row_count
}

#[cfg(not(feature = "search"))]
fn collect_rows<B, I, D>(buffer: &mut B, rows: I) -> usize
where
    B: AppendableBuffer,
    I: IntoIterator<Item = D>,
    D: fmt::Display,
{
    let mut row_count = 0;
    for row in rows {
        buffer.push_fmt(row);
        row_count += 1;
    }
    row_count
}

pub(crate) fn format_lines_into(
    buffer: &mut Rows,
    text: &String,
    line_numbers: LineNumbers,
    cols: usize,
    line_wrapping: bool,
    #[cfg(feature = "search")] search_term: Option<&regex::Regex>,
) -> FormatResult {
    let mut reusable_rows = ReusableRows::new(buffer);
    let format_opts = FormatOpts {
        buffer: &mut reusable_rows,
        text,
        attachment: None,
        line_numbers,
        formatted_lines_count: 0,
        lines_count: 0,
        prev_unterminated: 0,
        cols,
        #[cfg(feature = "search")]
        search_term,
        line_wrapping,
    };
    let fr = format_text_block(format_opts);
    reusable_rows.finish();
    fr
}

#[cfg(test)]
mod tests;
