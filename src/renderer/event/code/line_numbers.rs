use super::*;
use crate::cli::LineNumberTarget;

pub(super) struct CodeLineLayout {
    pub(super) lines: Vec<CodeLine>,
    number_width: usize,
}

pub(super) struct CodeLine {
    pub(super) text: String,
    pub(super) visible_width: usize,
    pub(super) number: Option<usize>,
}

impl<'a> EventRenderer<'a> {
    pub(super) fn layout_code_lines(
        &self,
        input: CodeBlockRenderInput<'_>,
        available_width: usize,
        pretty: bool,
    ) -> CodeLineLayout {
        let highlighted_lines = input.highlighted.lines().collect::<Vec<_>>();
        let raw_lines = input.raw_code.lines().collect::<Vec<_>>();
        let options = self.config.code_line_numbers;
        let local_number_width = options.map_or(0, |options| match options.target {
            LineNumberTarget::Rendered => 1,
            LineNumberTarget::Source => highlighted_lines.len().max(1).to_string().len(),
        });
        let mut number_width = local_number_width.max(self.config.code_line_number_width);

        loop {
            let gutter_width = options.map_or(0, |options| {
                crate::renderer::line_numbers::gutter_width_for_number_width(number_width, options)
            });
            let text_width = available_width.saturating_sub(gutter_width);
            let mut lines = Vec::new();
            let mut rendered_number = 1usize;

            for (source_index, highlighted_line) in highlighted_lines.iter().enumerate() {
                let raw_line = raw_lines.get(source_index).copied();
                let segments = if pretty {
                    self.wrap_code_line_segments_pretty(
                        highlighted_line,
                        raw_line,
                        text_width,
                        input.should_wrap,
                        input.wrap_mode,
                    )
                } else {
                    self.wrap_code_line_segments(
                        highlighted_line,
                        raw_line,
                        text_width,
                        input.should_wrap,
                        input.wrap_mode,
                    )
                };

                for (segment_index, segment) in segments.into_iter().enumerate() {
                    let number = match options.map(|options| options.target) {
                        Some(LineNumberTarget::Rendered) => Some(rendered_number),
                        Some(LineNumberTarget::Source) if segment_index == 0 => {
                            Some(source_index + 1)
                        }
                        Some(LineNumberTarget::Source) | None => None,
                    };
                    lines.push(CodeLine {
                        text: segment.text,
                        visible_width: segment.visible_width + gutter_width,
                        number,
                    });
                    rendered_number += 1;
                }
            }

            if lines.is_empty() && (pretty || options.is_some()) {
                lines.push(CodeLine {
                    text: String::new(),
                    visible_width: gutter_width,
                    number: options.map(|_| 1),
                });
            }

            if options.is_some_and(|options| options.target == LineNumberTarget::Rendered) {
                let required_width = lines.len().to_string().len();
                if required_width > number_width {
                    number_width = required_width;
                    continue;
                }
            }

            return CodeLineLayout {
                lines,
                number_width,
            };
        }
    }

    pub(super) fn format_code_line(
        &self,
        layout: &CodeLineLayout,
        line: &CodeLine,
        content: &str,
    ) -> String {
        let Some(options) = self.config.code_line_numbers else {
            return content.to_string();
        };
        let number_style = create_style(self.theme, ThemeElement::LineNumber);
        let separator_style = create_style(self.theme, ThemeElement::LineNumberSeparator);
        let mut rendered = crate::renderer::line_numbers::format_gutter(
            line.number,
            layout.number_width,
            &number_style,
            &separator_style,
            options,
            self.config.no_colors,
        );
        rendered.push_str(content);
        rendered
    }

    pub(super) fn record_code_line_number_width(&mut self, layout: &CodeLineLayout) {
        self.max_code_line_number_width = self.max_code_line_number_width.max(layout.number_width);
    }
}
