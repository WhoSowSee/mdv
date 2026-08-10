use crate::{LineNumbers, PagerState};
use crossterm::style::{Attribute, Color, ContentStyle};
use std::{fmt::Write, sync::Arc};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const RESET_STYLE: &str = "\x1b[0m";

/// Runs synchronously while pager state is locked.
pub type PromptRenderer = Arc<
    dyn for<'a> Fn(&PromptContext<'a>) -> Result<PromptLine, PromptError> + Send + Sync + 'static,
>;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PromptError {
    #[error("prompt content must fit on one line")]
    MultilineText,
    #[error("prompt content contains unsupported control character {0:?}")]
    ControlCharacter(char),
}

pub struct PromptContext<'a> {
    state: &'a PagerState,
}

impl<'a> PromptContext<'a> {
    pub(crate) const fn new(state: &'a PagerState) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn prompt(&self) -> &'a str {
        &self.state.prompt
    }

    #[must_use]
    pub fn message(&self) -> Option<&'a str> {
        self.state.message.as_deref()
    }

    #[must_use]
    pub fn display_text(&self) -> &'a str {
        self.message().unwrap_or_else(|| self.prompt())
    }

    #[must_use]
    pub const fn columns(&self) -> usize {
        self.state.cols
    }

    #[must_use]
    pub const fn rows(&self) -> usize {
        self.state.rows
    }

    #[must_use]
    pub const fn content_rows(&self) -> usize {
        self.state.content_rows()
    }

    #[must_use]
    pub const fn panel_rows(&self) -> usize {
        self.state.prompt_panel_rows()
    }

    #[must_use]
    pub const fn upper_mark(&self) -> usize {
        self.state.upper_mark
    }

    #[must_use]
    pub const fn left_mark(&self) -> usize {
        self.state.left_mark
    }

    #[must_use]
    pub const fn formatted_lines(&self) -> usize {
        self.state.screen.formatted_lines_count()
    }

    #[must_use]
    pub const fn logical_lines(&self) -> usize {
        self.state.screen.line_count()
    }

    #[must_use]
    pub const fn max_scroll_offset(&self) -> usize {
        self.state.max_upper_mark()
    }

    #[must_use]
    pub fn scroll_percentage(&self) -> u8 {
        let max_offset = self.max_scroll_offset();
        if max_offset == 0 {
            return 100;
        }

        let offset = self.upper_mark().min(max_offset) as u128;
        let percentage = (offset * 100 + max_offset as u128 / 2) / max_offset as u128;
        #[allow(clippy::cast_possible_truncation)]
        let percentage = percentage as u8;
        percentage
    }

    #[must_use]
    pub fn numeric_prefix(&self) -> &'a str {
        &self.state.prefix_num
    }

    #[must_use]
    pub const fn line_numbers(&self) -> LineNumbers {
        self.state.line_numbers
    }

    #[must_use]
    pub const fn line_wrapping(&self) -> bool {
        self.state.screen.line_wrapping
    }

    #[must_use]
    pub const fn follow_output(&self) -> bool {
        self.state.follow_output
    }

    #[must_use]
    #[cfg(feature = "search")]
    pub fn search_position(&self) -> Option<(usize, usize)> {
        let total = self.state.search_state.search_idx.len();
        (total > 0).then_some((self.state.search_state.search_mark + 1, total))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptColor {
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
    Rgb { r: u8, g: u8, b: u8 },
    AnsiValue(u8),
}

impl From<PromptColor> for Color {
    fn from(color: PromptColor) -> Self {
        match color {
            PromptColor::Black => Self::Black,
            PromptColor::DarkGrey => Self::DarkGrey,
            PromptColor::Red => Self::Red,
            PromptColor::DarkRed => Self::DarkRed,
            PromptColor::Green => Self::Green,
            PromptColor::DarkGreen => Self::DarkGreen,
            PromptColor::Yellow => Self::Yellow,
            PromptColor::DarkYellow => Self::DarkYellow,
            PromptColor::Blue => Self::Blue,
            PromptColor::DarkBlue => Self::DarkBlue,
            PromptColor::Magenta => Self::Magenta,
            PromptColor::DarkMagenta => Self::DarkMagenta,
            PromptColor::Cyan => Self::Cyan,
            PromptColor::DarkCyan => Self::DarkCyan,
            PromptColor::White => Self::White,
            PromptColor::Grey => Self::Grey,
            PromptColor::Rgb { r, g, b } => Self::Rgb { r, g, b },
            PromptColor::AnsiValue(value) => Self::AnsiValue(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptAttribute {
    Bold,
    Dim,
    Italic,
    Underlined,
    Reverse,
    Hidden,
    CrossedOut,
}

impl PromptAttribute {
    const ALL: [Self; 7] = [
        Self::Bold,
        Self::Dim,
        Self::Italic,
        Self::Underlined,
        Self::Reverse,
        Self::Hidden,
        Self::CrossedOut,
    ];

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

impl From<PromptAttribute> for Attribute {
    fn from(attribute: PromptAttribute) -> Self {
        match attribute {
            PromptAttribute::Bold => Self::Bold,
            PromptAttribute::Dim => Self::Dim,
            PromptAttribute::Italic => Self::Italic,
            PromptAttribute::Underlined => Self::Underlined,
            PromptAttribute::Reverse => Self::Reverse,
            PromptAttribute::Hidden => Self::Hidden,
            PromptAttribute::CrossedOut => Self::CrossedOut,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PromptStyle {
    foreground: Option<PromptColor>,
    background: Option<PromptColor>,
    attributes: u8,
}

impl PromptStyle {
    #[must_use]
    pub const fn foreground(mut self, color: PromptColor) -> Self {
        self.foreground = Some(color);
        self
    }

    #[must_use]
    pub const fn background(mut self, color: PromptColor) -> Self {
        self.background = Some(color);
        self
    }

    #[must_use]
    pub const fn attribute(mut self, attribute: PromptAttribute) -> Self {
        self.attributes |= attribute.bit();
        self
    }

    fn content_style(self) -> ContentStyle {
        let mut style = ContentStyle {
            foreground_color: self.foreground.map(Into::into),
            background_color: self.background.map(Into::into),
            ..ContentStyle::default()
        };
        for attribute in PromptAttribute::ALL {
            if self.attributes & attribute.bit() != 0 {
                style.attributes.set(attribute.into());
            }
        }
        style
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptSpan {
    text: String,
    style: PromptStyle,
}

#[allow(clippy::missing_errors_doc)]
impl PromptSpan {
    /// Rejects line breaks and terminal control characters.
    pub fn new(text: impl Into<String>, style: PromptStyle) -> Result<Self, PromptError> {
        let text = text.into();
        validate_single_line(&text)?;
        if let Some(character) = text.chars().find(|character| character.is_control()) {
            return Err(PromptError::ControlCharacter(character));
        }
        Ok(Self { text, style })
    }

    fn width(&self) -> usize {
        UnicodeWidthStr::width(self.text.as_str())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PromptLine {
    left: Vec<PromptSpan>,
    right: Vec<PromptSpan>,
    fill_style: PromptStyle,
    truncation_indicator: Option<PromptSpan>,
}

#[allow(clippy::missing_errors_doc)]
impl PromptLine {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
            fill_style: PromptStyle {
                foreground: None,
                background: None,
                attributes: 0,
            },
            truncation_indicator: None,
        }
    }

    pub fn plain(text: impl Into<String>) -> Result<Self, PromptError> {
        Ok(Self::new().left(PromptSpan::new(text, PromptStyle::default())?))
    }

    #[must_use]
    pub fn left(mut self, span: PromptSpan) -> Self {
        self.left.push(span);
        self
    }

    #[must_use]
    pub fn right(mut self, span: PromptSpan) -> Self {
        self.right.push(span);
        self
    }

    #[must_use]
    pub const fn fill_style(mut self, style: PromptStyle) -> Self {
        self.fill_style = style;
        self
    }

    #[must_use]
    pub fn truncation_indicator(mut self, span: PromptSpan) -> Self {
        self.truncation_indicator = Some(span);
        self
    }

    #[must_use]
    pub fn render(&self, columns: usize) -> String {
        let spans = self.layout(columns);
        let mut output = String::with_capacity(columns.saturating_add(spans.len() * 16));
        for span in spans {
            let _ = write!(output, "{}", span.style.content_style().apply(span.text));
        }
        output.push_str(RESET_STYLE);
        output
    }

    #[must_use]
    pub fn render_plain(&self, columns: usize) -> String {
        self.layout(columns)
            .into_iter()
            .map(|span| span.text)
            .collect()
    }

    fn layout(&self, columns: usize) -> Vec<PromptSpan> {
        if columns == 0 {
            return Vec::new();
        }

        let right_width = spans_width(&self.right);
        if right_width >= columns {
            let right = take_suffix(&self.right, columns);
            let filler_width = columns.saturating_sub(spans_width(&right));
            if filler_width == 0 {
                return right;
            }

            let mut output = vec![PromptSpan {
                text: " ".repeat(filler_width),
                style: self.fill_style,
            }];
            output.extend(right);
            return output;
        }

        let left_limit = columns - right_width;
        let mut output = if spans_width(&self.left) > left_limit {
            self.truncated_left(left_limit)
        } else {
            self.left.clone()
        };
        let filler_width = columns
            .saturating_sub(spans_width(&output))
            .saturating_sub(right_width);
        if filler_width > 0 {
            output.push(PromptSpan {
                text: " ".repeat(filler_width),
                style: self.fill_style,
            });
        }
        output.extend(self.right.clone());
        output
    }

    fn truncated_left(&self, width: usize) -> Vec<PromptSpan> {
        let indicator = self
            .truncation_indicator
            .as_ref()
            .map_or_else(Vec::new, |span| {
                take_prefix(std::slice::from_ref(span), width)
            });
        let content_width = width.saturating_sub(spans_width(&indicator));
        let mut left = take_prefix(&self.left, content_width);
        left.extend(indicator);
        left
    }
}

pub fn validate_single_line(text: &str) -> Result<(), PromptError> {
    if text.contains(['\n', '\r']) {
        Err(PromptError::MultilineText)
    } else {
        Ok(())
    }
}

fn spans_width(spans: &[PromptSpan]) -> usize {
    spans.iter().map(PromptSpan::width).sum()
}

fn take_prefix(spans: &[PromptSpan], width: usize) -> Vec<PromptSpan> {
    let mut remaining = width;
    let mut output = Vec::new();
    for span in spans {
        if remaining == 0 {
            break;
        }
        let text = take_text_prefix(&span.text, remaining);
        let used = UnicodeWidthStr::width(text.as_str());
        if !text.is_empty() {
            output.push(PromptSpan {
                text,
                style: span.style,
            });
        }
        remaining = remaining.saturating_sub(used);
        if used < span.width() {
            break;
        }
    }
    output
}

fn take_suffix(spans: &[PromptSpan], width: usize) -> Vec<PromptSpan> {
    let mut remaining = width;
    let mut reversed = Vec::new();
    for span in spans.iter().rev() {
        if remaining == 0 {
            break;
        }
        let text = take_text_suffix(&span.text, remaining);
        let used = UnicodeWidthStr::width(text.as_str());
        if !text.is_empty() {
            reversed.push(PromptSpan {
                text,
                style: span.style,
            });
        }
        remaining = remaining.saturating_sub(used);
        if used < span.width() {
            break;
        }
    }
    reversed.reverse();
    reversed
}

fn take_text_prefix(text: &str, width: usize) -> String {
    let mut used = 0;
    text.graphemes(true)
        .take_while(|grapheme| {
            let next = used + UnicodeWidthStr::width(*grapheme);
            if next > width {
                return false;
            }
            used = next;
            true
        })
        .collect()
}

fn take_text_suffix(text: &str, width: usize) -> String {
    let mut used = 0;
    let mut graphemes = text
        .graphemes(true)
        .rev()
        .take_while(|grapheme| {
            let next = used + UnicodeWidthStr::width(*grapheme);
            if next > width {
                return false;
            }
            used = next;
            true
        })
        .collect::<Vec<_>>();
    graphemes.reverse();
    graphemes.concat()
}

#[cfg(test)]
mod tests {
    use super::{PromptAttribute, PromptColor, PromptError, PromptLine, PromptSpan, PromptStyle};

    #[test]
    fn prompt_span_rejects_multiline_and_control_text() {
        let style = PromptStyle::default();

        assert_eq!(
            PromptSpan::new("first\nsecond", style),
            Err(PromptError::MultilineText)
        );
        assert_eq!(
            PromptSpan::new("unsafe\x1b[31m", style),
            Err(PromptError::ControlCharacter('\x1b'))
        );
    }

    #[test]
    fn prompt_line_handles_unicode_width_and_right_alignment() {
        let line = PromptLine::new()
            .left(PromptSpan::new("界界界", PromptStyle::default()).unwrap())
            .right(PromptSpan::new("XY", PromptStyle::default()).unwrap())
            .truncation_indicator(PromptSpan::new("…", PromptStyle::default()).unwrap());

        assert_eq!(line.render_plain(5), "界…XY");

        let narrow =
            PromptLine::new().right(PromptSpan::new("界", PromptStyle::default()).unwrap());
        assert_eq!(narrow.render_plain(1), " ");
    }

    #[test]
    fn prompt_line_pads_to_width_and_resets_styles() {
        let fill = PromptStyle::default().background(PromptColor::Rgb {
            r: 36,
            g: 36,
            b: 36,
        });
        let brand = PromptStyle::default()
            .foreground(PromptColor::AnsiValue(154))
            .attribute(PromptAttribute::Bold);
        let line = PromptLine::new()
            .left(PromptSpan::new("MDV", brand).unwrap())
            .right(PromptSpan::new("HELP", fill).unwrap())
            .fill_style(fill);

        assert_eq!(line.render_plain(10), "MDV   HELP");
        let rendered = line.render(10);
        assert!(rendered.contains("\x1b["));
        assert!(rendered.ends_with("\x1b[0m"));
    }
}
