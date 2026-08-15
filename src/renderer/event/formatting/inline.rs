use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn reset_explicit_blank_line_streak(&mut self) {
        self.explicit_blank_line_streak = 0;
    }

    pub(in crate::renderer::event) fn handle_explicit_blank_line(&mut self) {
        let prefix = self.current_line_prefix();
        let use_prefix = !prefix.is_empty();

        if self.has_trailing_blank_line() {
            if self.explicit_blank_line_streak > 0 {
                if use_prefix {
                    self.output.push('\n');
                    self.output.push_str(&prefix);
                    self.output.push('\n');
                } else {
                    self.output.push('\n');
                }
            }
        } else {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            if use_prefix {
                self.output.push_str(&prefix);
            }
            self.output.push('\n');
        }
        self.explicit_blank_line_streak = self.explicit_blank_line_streak.saturating_add(1);
    }

    pub(in crate::renderer::event) fn note_paragraph_content(&mut self) {
        if self.table_state.is_some() || self.current_paragraph_start.is_none() {
            return;
        }

        self.reset_explicit_blank_line_streak();

        if !self.current_paragraph_has_content {
            if self.current_paragraph_has_leading_break {
                self.trim_trailing_blank_lines();
                self.ensure_contextual_blank_line();
                self.current_paragraph_has_leading_break = false;
            }
            self.current_paragraph_has_content = true;
        }
    }

    /// Apply current formatting stack to text
    ///
    /// Ensures consistent precedence when multiple styles are active at once
    /// (e.g. Strong + Emphasis). Color precedence: Highlight > Code > Heading > StrongEmphasis > Strong > Emphasis > Strikethrough > TextLight > Text.
    pub(in crate::renderer::event) fn apply_formatting(&self, text: &str) -> String {
        self.apply_formatting_with_highlight(text, false)
    }

    pub(in crate::renderer::event) fn apply_formatting_with_highlight(
        &self,
        text: &str,
        highlighted: bool,
    ) -> String {
        if self.formatting_stack.is_empty() && !highlighted {
            return text.to_string();
        }

        let has_strong = self.formatting_stack.contains(&ThemeElement::Strong);
        let has_emphasis = self.formatting_stack.contains(&ThemeElement::Emphasis);
        let has_strike = self.formatting_stack.contains(&ThemeElement::Strikethrough);
        let has_code = self.formatting_stack.contains(&ThemeElement::Code);
        let has_text_light = self.formatting_stack.contains(&ThemeElement::TextLight);
        let has_underline = self.formatting_stack.contains(&ThemeElement::Underline);
        let heading = self.formatting_stack.iter().rev().copied().find(|element| {
            matches!(
                element,
                ThemeElement::H1
                    | ThemeElement::H2
                    | ThemeElement::H3
                    | ThemeElement::H4
                    | ThemeElement::H5
                    | ThemeElement::H6
            )
        });

        let semantic_kind = if has_code {
            Some(InlineStyleKind::Code)
        } else if has_strong && has_emphasis {
            Some(InlineStyleKind::StrongEmphasis)
        } else if has_strong {
            Some(InlineStyleKind::Strong)
        } else if has_emphasis {
            Some(InlineStyleKind::Emphasis)
        } else if has_strike {
            Some(InlineStyleKind::Strikethrough)
        } else {
            None
        };

        let mut style = if has_code {
            AnsiStyle::new().fg(self.theme.code.clone().into())
        } else if let Some(heading) = heading {
            create_style(self.theme, heading)
        } else if let Some(kind) = semantic_kind {
            let color = self
                .theme
                .inline_foreground(kind)
                .expect("semantic inline styles must define a foreground");
            AnsiStyle::new().fg(color.clone().into())
        } else if has_text_light {
            create_style(self.theme, ThemeElement::TextLight)
        } else {
            create_style(self.theme, ThemeElement::Text)
        };

        if highlighted && let Some(color) = self.theme.inline_foreground(InlineStyleKind::Highlight)
        {
            style = style.fg(color.clone().into());
        }

        let background_kind = if highlighted {
            Some(InlineStyleKind::Highlight)
        } else {
            semantic_kind
        };
        if let Some(background) =
            background_kind.and_then(|kind| self.theme.inline_background(kind))
        {
            style = style.bg(background.clone().into());
        }

        let mut attributes = InlineStyle::plain();
        if has_code {
            attributes.merge_attributes(self.theme.inline_style.get(InlineStyleKind::Code));
        }
        if has_strong && has_emphasis {
            attributes
                .merge_attributes(self.theme.inline_style.get(InlineStyleKind::StrongEmphasis));
        } else {
            if has_strong {
                attributes.merge_attributes(self.theme.inline_style.get(InlineStyleKind::Strong));
            }
            if has_emphasis {
                attributes.merge_attributes(self.theme.inline_style.get(InlineStyleKind::Emphasis));
            }
        }
        if has_strike {
            attributes
                .merge_attributes(self.theme.inline_style.get(InlineStyleKind::Strikethrough));
        }
        if highlighted {
            attributes.merge_attributes(self.theme.inline_style.get(InlineStyleKind::Highlight));
        }
        if has_underline {
            attributes.underline = true;
        }
        style = attributes.apply_attributes(style);

        style.apply(text, self.config.no_colors)
    }

    pub(in crate::renderer::event) fn sync_inline_backticks(&mut self, highlighted: bool) -> bool {
        let desired = self.desired_backtick_style(highlighted);
        if desired == self.active_backtick_style {
            return false;
        }

        self.close_inline_backticks();
        if let Some(kind) = desired {
            self.active_backtick_style = Some(kind);
            return true;
        }
        false
    }

    pub(in crate::renderer::event) fn close_inline_backticks(&mut self) {
        if self.active_backtick_style.take().is_some() {
            self.push_inline_backtick();
        }
    }

    pub(in crate::renderer::event) fn desired_backtick_style(
        &self,
        highlighted: bool,
    ) -> Option<InlineStyleKind> {
        let enabled = |kind| self.theme.inline_style.get(kind).backticks;
        if highlighted && enabled(InlineStyleKind::Highlight) {
            return Some(InlineStyleKind::Highlight);
        }

        let has_strong = self.formatting_stack.contains(&ThemeElement::Strong);
        let has_emphasis = self.formatting_stack.contains(&ThemeElement::Emphasis);
        if has_strong && has_emphasis {
            return enabled(InlineStyleKind::StrongEmphasis)
                .then_some(InlineStyleKind::StrongEmphasis);
        }
        if has_strong && enabled(InlineStyleKind::Strong) {
            return Some(InlineStyleKind::Strong);
        }
        if has_emphasis && enabled(InlineStyleKind::Emphasis) {
            return Some(InlineStyleKind::Emphasis);
        }
        if self.formatting_stack.contains(&ThemeElement::Strikethrough)
            && enabled(InlineStyleKind::Strikethrough)
        {
            return Some(InlineStyleKind::Strikethrough);
        }
        None
    }

    pub(in crate::renderer::event) fn push_inline_backtick(&mut self) {
        if self.in_link {
            self.current_link_text.push('`');
        } else if let Some(table) = self.table_state.as_mut() {
            table.current_cell.push('`');
        } else {
            self.output.push('`');
        }
    }
}
