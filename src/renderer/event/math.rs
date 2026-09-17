use super::{CowStr, EventRenderer, MathBlockStyle, Result, ThemeElement, create_style};
use crate::block_spacing::BlockElement;
use crate::math::{MathMode, RenderedMath, render_math_detailed, render_table_math_detailed};
use crate::utils::display_width;

pub(in crate::renderer::event) const PROTECTED_MATH_LAYOUT_MARKER: &str = "\u{2062}\u{2062}";

pub(in crate::renderer::event) fn strip_protected_math_layout(text: &str) -> String {
    text.replace(PROTECTED_MATH_LAYOUT_MARKER, "")
}

pub(in crate::renderer::event) fn has_protected_math_layout(text: &str) -> bool {
    text.contains(PROTECTED_MATH_LAYOUT_MARKER)
}

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_inline_math(&mut self, math: CowStr) -> Result<()> {
        if self.in_link {
            let rendered = self.render_math(math.as_ref(), MathMode::Inline);
            if rendered.trim().is_empty() {
                return Ok(());
            }
            self.current_link_text.push_str(&rendered);
            return Ok(());
        }

        if self.pending_callout_label_override {
            let rendered = self.render_math(math.as_ref(), MathMode::Inline);
            self.pending_callout_label_buffer.push_str(&rendered);
            return Ok(());
        }

        if self.table_state.is_some() {
            let rendered = self.render_table_math(math.as_ref(), false);
            self.append_math_to_table(&rendered);
            return Ok(());
        }

        let rendered = self.render_math(math.as_ref(), MathMode::Inline);
        if rendered.trim().is_empty() {
            return Ok(());
        }

        let style = create_style(self.theme, ThemeElement::Math);
        self.note_paragraph_content();
        let terminal_width = self.effective_text_width();
        self.push_styled_inline_atom(&rendered, &style, terminal_width);
        Ok(())
    }

    pub(super) fn handle_display_math(&mut self, math: CowStr) -> Result<()> {
        if self.table_state.is_some() {
            let rendered = self.render_table_math(math.as_ref(), true);
            self.append_math_to_table(&rendered);
            return Ok(());
        }

        let source_marker = self.take_pending_source_line_marker();

        if let Some(start) = self.current_paragraph_start
            && !self.current_paragraph_has_content
            && start <= self.output.len()
            && !Self::line_has_visible_text(&crate::utils::strip_ansi(
                &crate::renderer::line_numbers::strip_internal_markers(&self.output[start..]).0,
            ))
        {
            self.output.truncate(start);
            self.current_paragraph_has_leading_break = false;
            self.suppress_next_paragraph_break = true;
        }

        let rendered = self.render_math(math.as_ref(), MathMode::Display);
        self.render_math_block(&rendered, source_marker.as_deref());
        Ok(())
    }

    pub(super) fn handle_math_code_block(
        &mut self,
        raw_math: &str,
        source_marker: Option<&str>,
    ) -> Result<()> {
        if self.table_state.is_some() {
            let rendered = self.render_table_math(raw_math, true);
            self.append_math_to_table(&rendered);
            return Ok(());
        }

        let rendered = self.render_math(raw_math, MathMode::Display);
        self.render_math_block(&rendered, source_marker);
        Ok(())
    }

    fn render_math_block(&mut self, rendered: &str, source_marker: Option<&str>) {
        let mut rendered = rendered.trim_end().to_string();
        if rendered.trim().is_empty() {
            if !self.config.show_empty_elements {
                return;
            }
            rendered.clear();
        }

        let style = create_style(self.theme, ThemeElement::Math);
        let styled_lines = if rendered.is_empty() {
            vec![style.apply("", self.config.no_colors)]
        } else {
            rendered
                .lines()
                .map(|line| style.apply(line, self.config.no_colors))
                .collect()
        };

        let spacing = self.config.block_spacing.spacing(BlockElement::DisplayMath);
        self.ensure_contextual_blank_lines(spacing.top);
        self.render_math_container(&styled_lines, source_marker);
        self.ensure_contextual_blank_lines(spacing.bottom);
        self.commit_pending_heading_placeholder_if_content();
    }

    fn append_math_to_table(&mut self, rendered: &str) {
        if !rendered.trim().is_empty() {
            let style = create_style(self.theme, ThemeElement::Math);
            let styled = style.apply(rendered, self.config.no_colors);
            if let Some(table) = self.table_state.as_mut() {
                table.current_cell.push_str(&styled);
            }
        }
    }

    fn render_math(&self, source: &str, mode: MathMode) -> String {
        let RenderedMath {
            mut output,
            diagnostics,
            structured,
        } = render_math_detailed(source, mode);
        self.math_diagnostics.report(source, &diagnostics);
        if mode == MathMode::Display {
            let available = self.math_content_width();
            let max_width = output.lines().map(display_width).max().unwrap_or(0);
            if max_width > available && !structured && !output.contains('\n') {
                output =
                    crate::math::wrap_flat_math(&output, available, self.config.text_wrap_mode());
            }
            let remaining_width = output.lines().map(display_width).max().unwrap_or(0);
            if remaining_width > available && self.config.is_text_wrapping_enabled() {
                self.math_diagnostics
                    .report_overflow(source, remaining_width, available);
            }
        }
        output
    }

    fn render_table_math(&self, source: &str, force_display: bool) -> String {
        let RenderedMath {
            output,
            diagnostics,
            ..
        } = render_table_math_detailed(source, force_display);
        self.math_diagnostics.report(source, &diagnostics);
        output
    }

    fn math_content_width(&self) -> usize {
        let decoration = match self.config.math_block_style {
            MathBlockStyle::Basic | MathBlockStyle::Simple => 2,
            MathBlockStyle::Pretty => 4,
        };
        let context_width = if matches!(self.config.math_block_style, MathBlockStyle::Pretty)
            && self.callout_is_pretty()
        {
            0
        } else {
            self.compute_indented_block_context_width()
        };
        self.effective_text_width()
            .saturating_sub(context_width + decoration)
            .max(1)
    }
}

mod block;
