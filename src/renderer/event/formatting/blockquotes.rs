use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn compute_line_start_context_width(&self) -> usize {
        let prefix = self.current_line_prefix();
        display_width(&strip_ansi(&prefix))
    }

    pub(in crate::renderer::event) fn compute_indented_block_context_width(&self) -> usize {
        let prefix = self.current_indented_block_prefix();
        display_width(&strip_ansi(&prefix))
    }
    pub(in crate::renderer::event) fn render_blockquote_prefix(&self) -> String {
        self.render_blockquote_prefix_for_level(self.blockquote_level)
    }

    pub(in crate::renderer::event) fn render_blockquote_prefix_for_level(
        &self,
        level: usize,
    ) -> String {
        if level == 0 {
            return String::new();
        }

        if self.output_style.is_disabled() {
            let mut prefix = String::with_capacity(level + 1);
            for idx in 0..level {
                prefix.push(match self.callout_stack.get(idx) {
                    Some(CalloutState::Active(_)) => '┃',
                    _ => '│',
                });
            }
            prefix.push(' ');
            return prefix;
        }

        let mut prefix = String::new();
        for idx in 0..level {
            let styled = match self.callout_stack.get(idx) {
                Some(CalloutState::Active(info)) => self.style_callout_border("┃", info.kind),
                _ => create_style(self.theme, ThemeElement::Quote).apply("│", self.output_style),
            };
            prefix.push_str(&styled);
        }
        prefix.push(' ');
        prefix
    }

    pub(in crate::renderer::event) fn should_indent_after_blockquote_prefix(
        &self,
        level: usize,
    ) -> bool {
        if level == 0 {
            return false;
        }

        matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) && self
            .callout_stack
            .iter()
            .take(level)
            .any(|state| matches!(state, CalloutState::Active(_)))
    }

    pub(in crate::renderer::event) fn current_line_prefix_for_blockquote_level(
        &self,
        level: usize,
    ) -> String {
        self.current_line_prefix_for_blockquote_level_with_options(level, true)
    }

    pub(in crate::renderer::event) fn current_line_prefix_for_blockquote_level_with_options(
        &self,
        level: usize,
        include_list_indent: bool,
    ) -> String {
        let mut prefix = String::new();
        if level > 0 {
            let base_indent = if self.current_heading_start.is_some() {
                self.heading_indent
            } else {
                self.content_indent
            };
            let indent_after_prefix = self.should_indent_after_blockquote_prefix(level);
            if base_indent > 0 && !indent_after_prefix {
                prefix.push_str(&" ".repeat(base_indent));
            }
            prefix.push_str(&self.render_blockquote_prefix_for_level(level));
            if base_indent > 0 && indent_after_prefix {
                prefix.push_str(&" ".repeat(base_indent));
            }
            if include_list_indent && !self.list_stack.is_empty() {
                let list_indent = self
                    .calculate_list_content_indent()
                    .saturating_sub(self.content_indent);
                if list_indent > 0 {
                    prefix.push_str(&" ".repeat(list_indent));
                }
            }
        } else if include_list_indent && !self.list_stack.is_empty() {
            let list_content_indent = self.calculate_list_content_indent();
            prefix.push_str(&" ".repeat(list_content_indent));
        } else {
            let base_indent = if self.current_heading_start.is_some() {
                self.heading_indent
            } else {
                self.content_indent
            };
            if base_indent > 0 {
                prefix.push_str(&" ".repeat(base_indent));
            }
        }
        prefix
    }

    pub(in crate::renderer::event) fn current_rule_prefix_for_blockquote_level(
        &self,
        level: usize,
    ) -> String {
        let mut prefix = String::new();
        if level > 0 {
            prefix.push_str(&self.render_blockquote_prefix_for_level(level));
            if !self.list_stack.is_empty() {
                let list_indent = self.calculate_list_content_indent();
                if list_indent > 0 {
                    prefix.push_str(&" ".repeat(list_indent));
                }
            }
        } else if !self.list_stack.is_empty() {
            let list_content_indent = self.calculate_list_content_indent();
            if list_content_indent > 0 {
                prefix.push_str(&" ".repeat(list_content_indent));
            }
        }
        prefix
    }
}
