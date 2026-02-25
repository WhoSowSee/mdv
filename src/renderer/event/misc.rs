use super::{CowStr, EventRenderer, PRETTY_ACCENT_COLOR, Result, ThemeElement, create_style};
use crate::terminal::AnsiStyle;
use crate::utils::{display_width, strip_ansi};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_html(&mut self, html: CowStr) -> Result<()> {
        if html.as_ref().trim() == crate::markdown::BLANK_LINE_MARKER {
            self.handle_explicit_blank_line();
            return Ok(());
        }
        self.render_html_fragment(&html)
    }

    pub(super) fn handle_inline_html(&mut self, html: CowStr) -> Result<()> {
        if html.as_ref().trim() == crate::markdown::BLANK_LINE_MARKER {
            self.handle_explicit_blank_line();
            return Ok(());
        }
        self.render_html_fragment(&html)
    }

    fn render_html_fragment(&mut self, html: &CowStr) -> Result<()> {
        let html_str = html.as_ref();
        let trimmed = html_str.trim();

        let is_comment = trimmed.starts_with("<!--") && trimmed.ends_with("-->");
        if !is_comment || self.config.hide_comments {
            return Ok(());
        }

        self.note_paragraph_content();

        let style = create_style(self.theme, ThemeElement::Text);
        let mut followup_prefix: Option<String> = None;
        let line_start = self
            .output
            .rfind('\n')
            .map(|idx| idx.saturating_add(1))
            .unwrap_or(0);
        let current_line = &self.output[line_start..];

        if current_line.is_empty() {
            let prefix = self.current_line_prefix();
            if !prefix.is_empty() {
                self.push_indent_for_line_start();
                followup_prefix = Some(prefix);
            }
        } else {
            let prefix = self.current_line_prefix();
            if !prefix.is_empty() && current_line == prefix {
                followup_prefix = Some(prefix);
            }
        }

        let mut segments = html_str.split('\n').peekable();
        let mut first_segment = true;

        while let Some(segment) = segments.next() {
            if !first_segment {
                self.output.push('\n');
                if let Some(prefix) = followup_prefix.as_ref()
                    && !prefix.is_empty()
                    && (!segment.is_empty() || segments.peek().is_some())
                {
                    self.output.push_str(prefix);
                }
            } else {
                first_segment = false;
            }

            if segment.is_empty() {
                continue;
            }

            let rendered = style.apply(segment, self.config.no_colors);
            self.output.push_str(&rendered);
        }

        self.commit_pending_heading_placeholder_if_content();

        Ok(())
    }

    pub(super) fn handle_horizontal_rule(&mut self) -> Result<()> {
        self.reset_explicit_blank_line_streak();
        let prefix = self.current_rule_prefix();
        let prefix_width = display_width(&strip_ansi(&prefix));
        let width = self
            .effective_text_width()
            .saturating_sub(prefix_width)
            .max(1);
        let rule = if width >= 2 {
            format!("◈{}◈", "─".repeat(width.saturating_sub(2)))
        } else {
            "─".repeat(width)
        };
        let styled_rule = AnsiStyle::new()
            .fg(PRETTY_ACCENT_COLOR)
            .apply(&rule, self.config.no_colors);

        if !self.output.is_empty() {
            self.ensure_contextual_blank_line_with_prefix(&prefix);
        }

        if !prefix.is_empty() {
            self.output.push_str(&prefix);
        }
        self.output.push_str(&styled_rule);
        self.output.push('\n');
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_footnote_reference(&mut self, name: CowStr) -> Result<()> {
        self.note_paragraph_content();
        self.register_footnote_reference(name.as_ref());

        let marker = format!("[^{}]", name);
        let should_highlight = self.should_highlight_footnote_reference(name.as_ref());
        if let Some(ref mut table) = self.table_state {
            if should_highlight {
                let style = create_style(self.theme, ThemeElement::Link);
                let styled_marker = style.apply(&marker, self.config.no_colors);
                table
                    .inline_references
                    .push((marker.clone(), styled_marker));
            }
            table.current_cell.push_str(&marker);
        } else {
            let rendered_marker = if should_highlight {
                let style = create_style(self.theme, ThemeElement::Link);
                style.apply(&marker, self.config.no_colors)
            } else {
                marker
            };
            self.output.push_str(&rendered_marker);
        }

        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_task_list_marker(&mut self, checked: bool) -> Result<()> {
        self.note_paragraph_content();
        self.pending_task_marker = false;
        self.pending_task_marker_buffer.clear();
        let marker = if checked { "[✓] " } else { "[ ] " };
        let style = create_style(self.theme, ThemeElement::ListMarker);
        let styled_marker = style.apply(marker, self.config.no_colors);
        self.output.push_str(&styled_marker);
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
