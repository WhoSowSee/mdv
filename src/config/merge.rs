use super::*;

impl Config {
    pub(super) fn merge_with(&mut self, other: Self) {
        if other.no_colors {
            self.no_colors = other.no_colors;
        }

        if other.cols.is_some() {
            self.cols = other.cols;
        }

        if other.cols_from_cli {
            self.cols_from_cli = true;
        }

        if other.margin != HorizontalMargins::default() {
            self.margin = other.margin;
        }

        if other.tab_length != 4 {
            self.tab_length = other.tab_length;
        }

        if other.theme_info {
            self.theme_info = other.theme_info;
        }

        if !matches!(other.wrap, TextWrapMode::Char) {
            self.wrap = other.wrap;
        }

        if !matches!(other.table_wrap, TableWrapMode::Fit) {
            self.table_wrap = other.table_wrap;
        }
        if other.pretty_table {
            self.pretty_table = true;
        }
        if other.reflow {
            self.reflow = true;
        }
        // heading_layout defaults to Level; merge when non-default
        if !matches!(other.heading_layout, HeadingLayout::Level) {
            self.heading_layout = other.heading_layout;
        }
        if other.show_heading_markers {
            self.show_heading_markers = true;
        }
        if other.smart_indent {
            self.smart_indent = true;
        }
        if other.table_smart_indent {
            self.table_smart_indent = true;
        }
        self.block_spacing.merge(&other.block_spacing);

        if other.hide_comments {
            self.hide_comments = true;
        }
        if other.render_html {
            self.render_html = true;
        }
        if other.line_numbers.is_some() {
            self.line_numbers = other.line_numbers;
        }
        if other.code_line_numbers.is_some() {
            self.code_line_numbers = other.code_line_numbers;
        }
        if other.show_empty_elements {
            self.show_empty_elements = true;
        }
        if !other.code_guessing {
            self.code_guessing = false;
        }
        if other.syntaxes_dir.is_some() {
            self.syntaxes_dir = other.syntaxes_dir;
        }
        if other.code_block_style != CodeBlockStyleConfig::default() {
            self.code_block_style = other.code_block_style;
        }
        if other.callout_style != CalloutStyleConfig::default() {
            self.callout_style = other.callout_style;
        }
        if other.pretty_checkbox.is_some() {
            self.pretty_checkbox = other.pretty_checkbox;
        }
        if other.custom_checkbox.is_some() {
            self.custom_checkbox = other.custom_checkbox.clone();
        }
        if other.pretty_list.is_some() {
            self.pretty_list = other.pretty_list;
        }
        if other.pretty_definition.is_some() {
            self.pretty_definition = other.pretty_definition;
        }
        if other.uniform_list_marker.is_some() {
            self.uniform_list_marker = other.uniform_list_marker.clone();
        }
        if other.custom_list.is_some() {
            self.custom_list = other.custom_list.clone();
        }
        if !matches!(other.code_wrap_indent, CodeWrapIndent::Double) {
            self.code_wrap_indent = other.code_wrap_indent;
        }

        if other.theme != "terminal" {
            self.theme = other.theme;
        }

        if other.code_theme.is_some() {
            self.code_theme = other.code_theme;
        }

        if other.custom_theme.is_some() {
            self.custom_theme = other.custom_theme;
        }

        self.inline_style.merge(&other.inline_style);

        if other.custom_code_theme.is_some() {
            self.custom_code_theme = other.custom_code_theme;
        }
        if other.custom_callout.is_some() {
            self.custom_callout = other.custom_callout;
        }

        if other.custom_code_block.is_some() {
            self.custom_code_block = other.custom_code_block;
        }

        if other.custom_code_default_icon.is_some() {
            self.custom_code_default_icon = other.custom_code_default_icon;
        }

        if !matches!(other.link_style, LinkStyle::Clickable) {
            self.link_style = other.link_style;
        }

        if !matches!(other.link_truncation, LinkTruncationStyle::Wrap) {
            self.link_truncation = other.link_truncation;
        }

        if !matches!(other.footnote_style, FootnoteStyle::Endnotes) {
            self.footnote_style = other.footnote_style;
        }

        if !matches!(other.missing_footnote_style, MissingFootnoteStyle::Show) {
            self.missing_footnote_style = other.missing_footnote_style;
        }

        if other.from_text.is_some() {
            self.from_text = other.from_text;
        }

        if other.reverse {
            self.reverse = true;
        }
    }
}
