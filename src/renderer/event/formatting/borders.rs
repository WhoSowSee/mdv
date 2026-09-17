use super::*;
use unicode_segmentation::UnicodeSegmentation;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn render_code_block_border(&self) -> String {
        self.render_pipe_prefix(1, Some(CrosstermColor::White))
    }

    pub(in crate::renderer::event) fn render_pipe_prefix(
        &self,
        count: usize,
        color: Option<CrosstermColor>,
    ) -> String {
        if count == 0 {
            return String::new();
        }
        let prefix = format!("{} ", "│".repeat(count));
        if self.config.no_colors {
            return prefix;
        }
        if let Some(color) = color {
            let style = AnsiStyle::new().fg(color);
            style.apply(&prefix, self.config.no_colors)
        } else {
            prefix
        }
    }

    /// Returns a prefix and remainder without splitting grapheme clusters.
    /// A first cluster wider than a positive limit is consumed intact.
    pub(in crate::renderer::event) fn take_prefix_by_width(
        &self,
        s: &str,
        max_width: usize,
    ) -> (String, String) {
        if max_width == 0 || s.is_empty() {
            return (String::new(), s.to_string());
        }

        let mut width = 0usize;
        let mut split_idx = 0usize;
        for (i, grapheme) in s.grapheme_indices(true) {
            let grapheme_width = display_width(grapheme);
            if width + grapheme_width > max_width && split_idx > 0 {
                break;
            }
            width += grapheme_width;
            split_idx = i + grapheme.len();
        }
        (s[..split_idx].to_string(), s[split_idx..].to_string())
    }
}
