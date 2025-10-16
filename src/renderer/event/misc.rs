use super::{CowStr, EventRenderer, PRETTY_ACCENT_COLOR, Result, ThemeElement, create_style};
use crate::terminal::AnsiStyle;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_html(&mut self, html: CowStr) -> Result<()> {
        self.render_html_fragment(&html)
    }

    pub(super) fn handle_inline_html(&mut self, html: CowStr) -> Result<()> {
        self.render_html_fragment(&html)
    }

    fn render_html_fragment(&mut self, html: &CowStr) -> Result<()> {
        let html_str = html.as_ref();
        let trimmed = html_str.trim();

        let is_comment = trimmed.starts_with("<!--") && trimmed.ends_with("-->");
        if !is_comment || self.config.hide_comments {
            return Ok(());
        }

        let style = create_style(self.theme, ThemeElement::Text);
        let rendered = style.apply(html_str, self.config.no_colors);
        self.output.push_str(&rendered);
        self.commit_pending_heading_placeholder_if_content();

        Ok(())
    }

    pub(super) fn handle_horizontal_rule(&mut self) -> Result<()> {
        let width = self.config.get_terminal_width();
        let rule = format!("◈{}◈", "─".repeat(width.saturating_sub(2)));
        let styled_rule = AnsiStyle::new()
            .fg(PRETTY_ACCENT_COLOR)
            .apply(&rule, self.config.no_colors);

        if !self.output.is_empty() {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            if self.has_trailing_blank_line() {
                self.normalize_trailing_blank_line();
            } else {
                self.output.push('\n');
            }
        }
        self.output.push_str(&styled_rule);
        self.output.push('\n');
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_footnote_reference(&mut self, name: CowStr) -> Result<()> {
        let style = create_style(self.theme, ThemeElement::Link);
        let footnote = style.apply(&format!("[^{}]", name), self.config.no_colors);
        self.output.push_str(&footnote);
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_task_list_marker(&mut self, checked: bool) -> Result<()> {
        let marker = if checked { "[✓] " } else { "[ ] " };
        let style = create_style(self.theme, ThemeElement::ListMarker);
        let styled_marker = style.apply(marker, self.config.no_colors);
        self.output.push_str(&styled_marker);
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }
}
