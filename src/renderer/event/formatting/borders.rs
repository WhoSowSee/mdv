use super::*;

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

    /// Helper: take a visible-width prefix from `s` that fits into `max_width`.
    /// Returns (prefix, rest). Uses display width and is unicode-safe.
    pub(in crate::renderer::event) fn take_prefix_by_width(
        &self,
        s: &str,
        max_width: usize,
    ) -> (String, String) {
        if max_width == 0 || s.is_empty() {
            return (String::new(), s.to_string());
        }

        let mut taken = String::new();
        let mut width = 0usize;
        let mut split_idx = 0usize;
        for (i, ch) in s.char_indices() {
            let ch_w = crate::utils::display_width(&ch.to_string());
            if width + ch_w > max_width {
                break;
            }
            taken.push(ch);
            width += ch_w;
            split_idx = i + ch.len_utf8();
        }
        let rest = s.get(split_idx..).unwrap_or("").to_string();
        (taken, rest)
    }
}
