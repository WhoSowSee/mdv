use super::{
    CodeBlockStyle, CowStr, EventRenderer, Result, ThemeElement, WrapMode, create_style,
};
use crate::math::{MathMode, render_math};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_inline_math(&mut self, math: CowStr) -> Result<()> {
        let rendered = render_math(math.as_ref(), MathMode::Inline);
        if rendered.trim().is_empty() {
            return Ok(());
        }

        let style = create_style(self.theme, ThemeElement::Code);
        let styled = style.apply(&rendered, self.config.no_colors);

        if let Some(ref mut table) = self.table_state {
            table.current_cell.push_str(&styled);
            return Ok(());
        }

        self.note_paragraph_content();

        if !self.config.is_text_wrapping_enabled() {
            self.output.push_str(&styled);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(());
        }

        let terminal_width = self.config.get_terminal_width();
        let wrap_mode = self.config.text_wrap_mode();

        let mut remaining = rendered.clone();

        while !remaining.is_empty() {
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };
            let current_line_width = crate::utils::display_width(&current_line_clean);
            let available = terminal_width.saturating_sub(current_line_width);

            if available == 0 {
                self.push_newline_with_context();
                continue;
            }

            let line_indent_width = self.compute_line_start_context_width();
            let effective_indent = line_indent_width.min(current_line_width);
            let has_line_content = current_line_width > effective_indent;
            let remaining_width = crate::utils::display_width(&remaining);

            match wrap_mode {
                WrapMode::Word => {
                    if remaining_width <= available {
                        let styled_chunk = style.apply(&remaining, self.config.no_colors);
                        self.output.push_str(&styled_chunk);
                        remaining.clear();
                    } else if has_line_content {
                        self.push_newline_with_context();
                    } else {
                        let (chunk, rest) = self.take_prefix_by_width(&remaining, available);
                        let styled_chunk = style.apply(&chunk, self.config.no_colors);
                        self.output.push_str(&styled_chunk);
                        remaining = rest;
                        if !remaining.is_empty() {
                            self.push_newline_with_context();
                        }
                    }
                }
                WrapMode::Character | WrapMode::None => {
                    let (chunk, rest) = self.take_prefix_by_width(&remaining, available);
                    let styled_chunk = style.apply(&chunk, self.config.no_colors);
                    self.output.push_str(&styled_chunk);
                    remaining = rest;
                    if !remaining.is_empty() {
                        self.push_newline_with_context();
                    }
                }
            }
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_display_math(&mut self, math: CowStr) -> Result<()> {
        if self.table_state.is_some() {
            let inline = render_math(math.as_ref(), MathMode::Inline);
            if !inline.trim().is_empty() {
                let style = create_style(self.theme, ThemeElement::Code);
                let styled = style.apply(&inline, self.config.no_colors);
                if let Some(ref mut table) = self.table_state {
                    table.current_cell.push_str(&styled);
                }
            }
            return Ok(());
        }

        let rendered = render_math(math.as_ref(), MathMode::Display);
        self.render_math_block(&rendered, None)
    }

    pub(super) fn handle_math_code_block(
        &mut self,
        raw_math: &str,
        language_hint: Option<&str>,
    ) -> Result<()> {
        if self.table_state.is_some() {
            let inline = render_math(raw_math, MathMode::Inline);
            if !inline.trim().is_empty() {
                let style = create_style(self.theme, ThemeElement::Code);
                let styled = style.apply(&inline, self.config.no_colors);
                if let Some(ref mut table) = self.table_state {
                    table.current_cell.push_str(&styled);
                }
            }
            return Ok(());
        }

        let rendered = render_math(raw_math, MathMode::Display);
        let label = if self.config.no_code_language {
            None
        } else {
            match language_hint {
                Some(hint) if hint.eq_ignore_ascii_case("latex") => Some("LaTeX"),
                Some(hint) if hint.eq_ignore_ascii_case("tex") => Some("TeX"),
                _ => Some("Math"),
            }
        };
        self.render_math_block(&rendered, label)
    }

    fn render_math_block(&mut self, rendered: &str, label: Option<&str>) -> Result<()> {
        let mut rendered = rendered.trim_end().to_string();
        if rendered.trim().is_empty() {
            if !self.config.show_empty_elements {
                return Ok(());
            }
            rendered.clear();
        }

        let style = create_style(self.theme, ThemeElement::Code);
        let lines: Vec<&str> = if rendered.is_empty() {
            vec![""]
        } else {
            rendered.lines().collect()
        };
        let highlighted = lines
            .iter()
            .map(|line| style.apply(line, self.config.no_colors))
            .collect::<Vec<_>>()
            .join("\n");

        let should_wrap = self.config.is_text_wrapping_enabled();
        let wrap_mode = self.config.text_wrap_mode();
        let terminal_width = self.config.get_terminal_width();

        self.ensure_contextual_blank_line();

        match self.config.code_block_style {
            CodeBlockStyle::Simple => {
                self.render_code_block_simple(
                    &highlighted,
                    label,
                    false,
                    should_wrap,
                    wrap_mode,
                    terminal_width,
                    &rendered,
                )?;
            }
            CodeBlockStyle::Pretty => {
                self.render_code_block_pretty(
                    &highlighted,
                    label,
                    false,
                    should_wrap,
                    wrap_mode,
                    terminal_width,
                    &rendered,
                )?;
            }
        }

        self.ensure_contextual_blank_line();
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
