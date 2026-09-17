use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_math_container(&mut self, lines: &[String], source_marker: Option<&str>) {
        match self.config.math_block_style {
            MathBlockStyle::Basic => self.render_prefixed_math_lines(lines, "  ", source_marker),
            MathBlockStyle::Simple => self.render_math_simple(lines, source_marker),
            MathBlockStyle::Pretty => self.render_math_pretty(lines, source_marker),
        }
    }

    fn render_math_simple(&mut self, lines: &[String], source_marker: Option<&str>) {
        let border = create_style(self.theme, ThemeElement::MathBorder);
        let prefix = border.apply("│ ", self.config.no_colors);
        self.render_prefixed_math_lines(lines, &prefix, source_marker);
    }

    fn render_prefixed_math_lines(
        &mut self,
        lines: &[String],
        prefix: &str,
        source_marker: Option<&str>,
    ) {
        for (index, line) in lines.iter().enumerate() {
            self.push_indented_block_prefix();
            self.push_math_layout_marker();
            if index == 0
                && let Some(marker) = source_marker
            {
                self.output.push_str(marker);
            }
            self.output.push_str(prefix);
            self.output.push_str(line);
            self.output.push('\n');
        }
    }

    fn render_math_pretty(&mut self, lines: &[String], source_marker: Option<&str>) {
        let width = lines
            .iter()
            .map(|line| display_width(&crate::utils::strip_ansi(line)))
            .max()
            .unwrap_or(0)
            .max(1);
        let inner_width = width.saturating_add(4);
        let border = create_style(self.theme, ThemeElement::MathBorder);
        let top = format!("╭{}╮", "─".repeat(inner_width.saturating_sub(2)));
        let bottom = format!("╰{}╯", "─".repeat(inner_width.saturating_sub(2)));

        self.push_math_pretty_indent();
        self.output
            .push_str(&border.apply(&top, self.config.no_colors));
        self.output.push('\n');

        for (index, line) in lines.iter().enumerate() {
            let clean_width = display_width(&crate::utils::strip_ansi(line));
            let padding = width.saturating_sub(clean_width);
            self.push_math_pretty_indent();
            if index == 0
                && let Some(marker) = source_marker
            {
                self.output.push_str(marker);
            }
            self.output
                .push_str(&border.apply("│ ", self.config.no_colors));
            self.output.push_str(line);
            self.output.push_str(&" ".repeat(padding));
            self.output
                .push_str(&border.apply(" │", self.config.no_colors));
            self.output.push('\n');
        }

        self.push_math_pretty_indent();
        self.output
            .push_str(&border.apply(&bottom, self.config.no_colors));
        self.output.push('\n');
    }

    fn push_math_pretty_indent(&mut self) {
        if !self.callout_is_pretty() {
            self.push_indented_block_prefix();
        }
        self.push_math_layout_marker();
    }

    fn push_math_layout_marker(&mut self) {
        if self.callout_is_pretty() {
            self.output.push_str(PROTECTED_MATH_LAYOUT_MARKER);
        }
    }
}
