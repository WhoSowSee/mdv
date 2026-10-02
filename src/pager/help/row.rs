use crate::terminal::HELP_DESCRIPTION_COLOR;
use minus::{PromptError, PromptLine, PromptSpan, PromptStyle};
use std::ops::Range;

pub(super) struct HelpRow {
    text: String,
    descriptions: Vec<Range<usize>>,
}

impl HelpRow {
    pub(super) fn new() -> Self {
        Self {
            text: "  ".to_owned(),
            descriptions: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, item: &str) {
        if let Some((_, label)) = item.split_once("  ")
            && !label.trim().is_empty()
        {
            let start = self.text.len() + item.len() - label.trim_start().len();
            let end = self.text.len() + item.trim_end().len();
            self.descriptions.push(start..end);
        }
        self.text.push_str(item);
    }

    pub(super) fn render(&self, style: PromptStyle) -> Result<PromptLine, PromptError> {
        self.render_range(0..self.text.len(), style)
    }

    pub(super) fn wrap(
        &self,
        width: usize,
        style: PromptStyle,
    ) -> Result<Vec<PromptLine>, PromptError> {
        let wrapped =
            crate::utils::wrap_text_with_mode(&self.text, width, crate::utils::WrapMode::Word);
        let mut offset = 0;
        wrapped
            .lines()
            .map(|part| {
                // Plain wrapping only removes whitespace between contiguous source slices.
                let start = offset
                    + self.text[offset..]
                        .find(part)
                        .expect("wrapped help fragment belongs to the source row");
                offset = start + part.len();
                self.render_range(start..offset, style)
            })
            .collect()
    }

    fn render_range(
        &self,
        range: Range<usize>,
        style: PromptStyle,
    ) -> Result<PromptLine, PromptError> {
        let mut line = PromptLine::new().fill_style(style);
        let mut cursor = range.start;
        for description in &self.descriptions {
            let start = description.start.max(range.start);
            let end = description.end.min(range.end);
            if start >= end {
                continue;
            }
            if cursor < start {
                line = line.left(PromptSpan::new(&self.text[cursor..start], style)?);
            }
            line = line.left(PromptSpan::new(
                &self.text[start..end],
                style.foreground(HELP_DESCRIPTION_COLOR),
            )?);
            cursor = end;
        }
        if cursor < range.end {
            line = line.left(PromptSpan::new(&self.text[cursor..range.end], style)?);
        }
        Ok(line)
    }
}
