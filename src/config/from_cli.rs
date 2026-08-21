use super::*;

impl Config {
    pub fn from_cli(cli: &Cli, matches: &ArgMatches) -> Result<Self> {
        let mut config = Self::load_config_files(cli, matches)?;
        if let Some(name) = cli.preset.as_deref() {
            preset::apply_named_preset(&mut config, name)?;
        }

        if let Some(syntaxes_dir) = config.syntaxes_dir.take() {
            config.syntaxes_dir = Some(resolve_config_relative_path(
                &syntaxes_dir,
                config.config_dir.as_deref(),
            ));
        }

        if let Some(no_colors) = mdv_no_color_override() {
            config.no_colors = no_colors;
        }

        if cli.no_colors {
            config.no_colors = true;
        }

        if let Some(cols) = cli.cols
            && arg_has_user_value(matches, "cols")
        {
            config.cols = Some(cols);
            config.cols_from_cli = true;
        }

        if let Some(margin) = cli.margin
            && arg_has_user_value(matches, "margin")
        {
            config.margin = margin;
        }

        if let Some(tab_length) = cli.tab_length
            && arg_has_user_value(matches, "tab_length")
        {
            config.tab_length = tab_length;
        }

        if let Some(wrap) = cli.wrap_mode
            && arg_has_user_value(matches, "wrap_mode")
        {
            config.wrap = wrap;
        }

        if let Some(table_wrap) = cli.table_wrap_mode
            && arg_has_user_value(matches, "table_wrap_mode")
        {
            config.table_wrap = table_wrap;
        }
        if cli.pretty_table {
            config.pretty_table = true;
        }
        if cli.reflow {
            config.reflow = true;
        }

        if cli.theme_info.is_some() {
            config.theme_info = true;
        }

        if cli.no_code_guessing {
            config.code_guessing = false;
        }

        if let Some(syntaxes_dir) = &cli.syntaxes_dir
            && arg_has_user_value(matches, "syntaxes_dir")
        {
            config.syntaxes_dir = Some(expand_tilde(syntaxes_dir));
        }

        if let Some(theme) = &cli.theme
            && arg_has_user_value(matches, "theme")
        {
            config.theme = theme.clone();
        }

        if let Some(code_theme) = &cli.code_theme
            && arg_has_user_value(matches, "code_theme")
        {
            config.code_theme = Some(code_theme.clone());
        }

        if let Some(custom_theme) = &cli.custom_theme
            && arg_has_user_value(matches, "custom_theme")
        {
            config.custom_theme = Some(custom_theme.clone());
        }

        if let Some(inline_style) = &cli.inline_style
            && arg_has_user_value(matches, "inline_style")
        {
            config.inline_style.merge(inline_style);
        }

        if let Some(custom_code_theme) = &cli.custom_code_theme
            && arg_has_user_value(matches, "custom_code_theme")
        {
            config.custom_code_theme = Some(custom_code_theme.clone());
        }

        if let Some(custom_callout) = &cli.custom_callout
            && arg_has_user_value(matches, "custom_callout")
        {
            config.custom_callout = Some(custom_callout.clone());
        }

        if let Some(custom_code_block) = &cli.custom_code_block
            && arg_has_user_value(matches, "custom_code_block")
        {
            config.custom_code_block = Some(custom_code_block.clone());
        }

        if let Some(link_style) = cli.link_style.clone()
            && arg_has_user_value(matches, "link_style")
        {
            config.link_style = link_style;
        }

        if let Some(link_truncation) = cli.link_truncation.clone()
            && arg_has_user_value(matches, "link_truncation")
        {
            config.link_truncation = link_truncation;
        }

        if let Some(footnote_style) = cli.footnote_style
            && arg_has_user_value(matches, "footnote_style")
        {
            config.footnote_style = footnote_style;
        }

        if let Some(missing_style) = cli.missing_footnote_style
            && arg_has_user_value(matches, "missing_footnote_style")
        {
            config.missing_footnote_style = missing_style;
        }

        if let Some(heading_layout) = cli.heading_layout.clone()
            && arg_has_user_value(matches, "heading_layout")
        {
            config.heading_layout = heading_layout;
        }
        if cli.show_heading_markers {
            config.show_heading_markers = true;
        }
        if cli.smart_indent {
            config.smart_indent = true;
        }
        if cli.table_smart_indent {
            config.table_smart_indent = true;
        }
        if let Some(spacing) = &cli.block_spacing {
            config.block_spacing.merge(spacing);
        }

        if cli.hide_comments {
            config.hide_comments = true;
        }

        if let Some(mode) = cli.front_matter
            && arg_has_user_value(matches, "front_matter")
        {
            config.front_matter = mode;
        }

        if cli.render_html {
            config.render_html = true;
        }

        if let Some(options) = cli.line_numbers {
            config.line_numbers = Some(options.unwrap_or_default());
        }

        if let Some(options) = cli.code_line_numbers {
            config.code_line_numbers = Some(options.unwrap_or_default());
        }

        if cli.show_empty_elements {
            config.show_empty_elements = true;
        }

        if let Some(style) = cli.code_block_style
            && arg_has_user_value(matches, "code_block_style")
        {
            config.code_block_style = style;
        }
        if let Some(style) = cli.style_callout
            && arg_has_user_value(matches, "style_callout")
        {
            config.callout_style = style;
        }
        if let Some(shape) = cli.pretty_checkbox
            && arg_has_user_value(matches, "pretty_checkbox")
        {
            config.pretty_checkbox = Some(shape);
        }
        if let Some(style) = cli.pretty_list
            && arg_has_user_value(matches, "pretty_list")
        {
            config.pretty_list = Some(style);
        }
        if let Some(style) = cli.pretty_definition
            && arg_has_user_value(matches, "pretty_definition")
        {
            config.pretty_definition = Some(style);
        }

        if let Some(marker) = &cli.uniform_list_marker
            && arg_has_user_value(matches, "uniform_list_marker")
        {
            config.uniform_list_marker = Some(marker.clone());
        }

        if let Some(raw) = &cli.custom_list
            && arg_has_user_value(matches, "custom_list")
        {
            config.custom_list = Some(raw.clone());
        }

        if let Some(raw) = &cli.custom_checkbox
            && arg_has_user_value(matches, "custom_checkbox")
        {
            config.custom_checkbox = Some(raw.clone());
        }

        if let Some(indent) = cli.code_wrap_indent
            && arg_has_user_value(matches, "code_wrap_indent")
        {
            config.code_wrap_indent = indent;
        }

        if let Some(from_text) = &cli.from_txt
            && arg_has_user_value(matches, "from_txt")
        {
            config.from_text = Some(from_text.clone());
        }

        if cli.reverse {
            config.reverse = true;
        }

        config.normalize_theme_settings();
        config.apply_custom_callouts()?;
        config.apply_custom_code_blocks()?;
        config.apply_checkbox_overrides()?;
        config.apply_list_markers()?;

        Ok(config)
    }
}
