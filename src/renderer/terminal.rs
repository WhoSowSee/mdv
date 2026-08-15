use super::event::EventRenderer;
use super::syntax_set::load_full_syntax_set;
use super::syntax_theme::{CodeHighlightTheme, build_syntect_theme, default_theme_set};
use crate::cli::{LineNumberOptions, LineNumberTarget};
use crate::config::Config;
use crate::theme::{
    Theme, ThemeElement, ThemeManager, apply_custom_code_theme, apply_custom_theme, create_style,
};
use crate::user_themes;
use anyhow::Result;
use pulldown_cmark::Event;
use std::sync::Arc;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Terminal renderer for markdown content
pub struct TerminalRenderer {
    config: Config,
    theme: Theme,
    syntax_set: Arc<SyntaxSet>,
    code_theme: CodeHighlightTheme,
}

impl TerminalRenderer {
    pub fn new(config: &Config) -> Result<Self> {
        let theme_manager = build_theme_manager(config);
        let mut theme = theme_manager.get_theme(&config.theme)?.clone();
        if let Some(overrides) = &config.custom_theme {
            apply_custom_theme(&mut theme, overrides)?;
        }

        if let Some(overrides) = &config.custom_code_theme {
            apply_custom_code_theme(&mut theme, overrides)?;
        }

        theme.inline_style.apply_overrides(&config.inline_style);

        if (config.custom_theme.is_some() || config.custom_code_theme.is_some())
            && !theme.name.ends_with("+custom")
        {
            theme.name = format!("{}+custom", theme.name);
        }

        let syntax_set = load_full_syntax_set(config.syntaxes_dir.as_deref())?;
        let theme_set = default_theme_set();

        let code_theme = if config.custom_code_theme.is_some() {
            if config.code_theme.is_some() {
                log::info!(
                    "Ignoring '--code-theme' because '--custom-code-theme' overrides are applied."
                );
            }
            build_syntect_theme(&theme)
        } else {
            match config.code_theme.as_ref() {
                Some(requested_theme) => {
                    resolve_code_theme(requested_theme, &theme, &theme_manager, theme_set)
                }
                None => build_syntect_theme(&theme),
            }
        };

        Ok(Self {
            config: config.clone(),
            theme,
            syntax_set,
            code_theme,
        })
    }

    pub fn render(&self, events: Vec<Event<'static>>) -> Result<String> {
        let output = match self.config.line_numbers {
            None => self.render_events(&self.config, events)?,
            Some(options) => match options.target {
                LineNumberTarget::Rendered => {
                    self.render_with_rendered_line_numbers(events, options)?
                }
                LineNumberTarget::Source => {
                    self.render_with_source_line_numbers(events, options)?
                }
            },
        };

        Ok(apply_left_margin(&output, self.config.margin.left))
    }

    pub(crate) const fn pager_status_bar_transparent(&self) -> bool {
        self.theme.pager_status_bar_transparent
    }

    fn render_events(&self, config: &Config, events: Vec<Event<'static>>) -> Result<String> {
        config.validate_horizontal_margins()?;
        let mut renderer =
            EventRenderer::new(config, &self.theme, &self.syntax_set, &self.code_theme);
        renderer.render_events(events)
    }

    fn render_with_source_line_numbers(
        &self,
        events: Vec<Event<'static>>,
        options: LineNumberOptions,
    ) -> Result<String> {
        let Some(max_line) = super::line_numbers::max_source_line(&events) else {
            return self.render_events(&self.config, events);
        };

        let mut render_config = self.config.clone();
        render_config.line_number_gutter_width =
            super::line_numbers::gutter_width(max_line, options);
        let output = self.render_events(&render_config, events)?;
        Ok(self.apply_line_numbers(&output, max_line, options))
    }

    fn render_with_rendered_line_numbers(
        &self,
        events: Vec<Event<'static>>,
        options: LineNumberOptions,
    ) -> Result<String> {
        let mut number_width = 1;

        loop {
            let mut render_config = self.config.clone();
            render_config.line_number_gutter_width =
                super::line_numbers::gutter_width_for_number_width(number_width, options);
            let output = self.render_events(&render_config, events.clone())?;
            let rendered_lines = super::line_numbers::rendered_line_count(&output);

            if rendered_lines == 0 {
                return Ok(output);
            }

            let required_width = rendered_lines.to_string().len();
            if required_width <= number_width {
                return Ok(self.apply_line_numbers(&output, rendered_lines, options));
            }

            number_width = required_width;
        }
    }

    fn apply_line_numbers(
        &self,
        output: &str,
        max_line: usize,
        options: LineNumberOptions,
    ) -> String {
        let number_style = create_style(&self.theme, ThemeElement::LineNumber);
        let separator_style = create_style(&self.theme, ThemeElement::LineNumberSeparator);
        super::line_numbers::apply_line_numbers(
            output,
            max_line,
            &number_style,
            &separator_style,
            options,
            self.config.no_colors,
        )
    }

    pub fn to_html(&self, events: Vec<Event<'static>>) -> Result<String> {
        let events = events.into_iter().filter_map(|event| {
            if crate::markdown::source_line_from_event(&event).is_some() {
                return None;
            }

            Some(match event {
                Event::Html(html) if html.as_ref().trim() == crate::markdown::BLANK_LINE_MARKER => {
                    Event::HardBreak
                }
                Event::InlineHtml(html)
                    if html.as_ref().trim() == crate::markdown::BLANK_LINE_MARKER =>
                {
                    Event::HardBreak
                }
                other => other,
            })
        });
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events);
        Ok(html_output)
    }
}

fn apply_left_margin(output: &str, margin: usize) -> String {
    if margin == 0 || output.is_empty() {
        return output.to_string();
    }

    let prefix = " ".repeat(margin);
    let mut indented = String::with_capacity(output.len() + prefix.len());
    for line in output.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        if !content.is_empty() {
            indented.push_str(&prefix);
        }
        indented.push_str(line);
    }
    indented
}

fn resolve_code_theme(
    requested_theme: &str,
    main_theme: &Theme,
    theme_manager: &ThemeManager,
    theme_set: &ThemeSet,
) -> CodeHighlightTheme {
    if let Some(theme) = theme_set.themes.get(requested_theme) {
        return CodeHighlightTheme::syntect_only(theme.clone());
    }

    if let Some((actual_name, theme)) = theme_set
        .themes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(requested_theme))
    {
        log::info!(
            "Using syntax theme '{}' for '--code-theme {}'.",
            actual_name,
            requested_theme
        );
        return CodeHighlightTheme::syntect_only(theme.clone());
    }

    if let Ok(builtin_theme) = theme_manager.get_theme(requested_theme) {
        return build_syntect_theme(builtin_theme);
    }

    log::warn!(
        "Code theme '{}' not found; falling back to '{}'.",
        requested_theme,
        main_theme.name
    );
    build_syntect_theme(main_theme)
}

pub(crate) fn build_theme_manager(config: &Config) -> ThemeManager {
    let mut manager = ThemeManager::new();
    if let Some(config_dir) = &config.config_dir {
        match user_themes::load_user_themes(config_dir, &manager) {
            Ok(themes) => {
                for theme in themes {
                    manager.add_theme(theme);
                }
            }
            Err(err) => {
                log::warn!(
                    "Failed to load user themes from '{}': {}",
                    config_dir.display(),
                    err
                );
            }
        }
    }
    manager
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_exposes_pager_status_bar_transparency() {
        let config = Config {
            custom_theme: Some("pager_status_bar_transparent=true".to_string()),
            ..Config::default()
        };

        let renderer = TerminalRenderer::new(&config).unwrap();

        assert!(renderer.pager_status_bar_transparent());
    }
}
