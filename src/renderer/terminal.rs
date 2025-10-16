use super::event::EventRenderer;
use super::syntax_set::load_full_syntax_set;
use super::syntax_theme::{build_syntect_theme, default_theme_set};
use crate::config::Config;
use crate::theme::{Theme, ThemeManager, apply_custom_code_theme, apply_custom_theme};
use anyhow::Result;
use pulldown_cmark::Event;
use syntect::highlighting::{Theme as SyntectTheme, ThemeSet};
use syntect::parsing::SyntaxSet;

/// Terminal renderer for markdown content
pub struct TerminalRenderer {
    config: Config,
    theme: Theme,
    syntax_set: &'static SyntaxSet,
    code_theme: SyntectTheme,
}

impl TerminalRenderer {
    pub fn new(config: &Config) -> Result<Self> {
        let theme_manager = ThemeManager::new();
        let mut theme = theme_manager.get_theme(&config.theme)?.clone();

        if let Some(overrides) = &config.custom_theme {
            apply_custom_theme(&mut theme, overrides)?;
        }

        if let Some(overrides) = &config.custom_code_theme {
            apply_custom_code_theme(&mut theme, overrides)?;
        }

        if config.custom_theme.is_some() || config.custom_code_theme.is_some() {
            if !theme.name.ends_with("+custom") {
                theme.name = format!("{}+custom", theme.name);
            }
        }

        let syntax_set = load_full_syntax_set();
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

    pub fn render(&self, events: Vec<Event>) -> Result<String> {
        let mut renderer = EventRenderer::new(
            &self.config,
            &self.theme,
            &self.syntax_set,
            &self.code_theme,
        );
        renderer.render_events(events)
    }

    pub fn to_html(&self, events: Vec<Event>) -> Result<String> {
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
        Ok(html_output)
    }
}

fn resolve_code_theme(
    requested_theme: &str,
    main_theme: &Theme,
    theme_manager: &ThemeManager,
    theme_set: &ThemeSet,
) -> SyntectTheme {
    if let Some(theme) = theme_set.themes.get(requested_theme) {
        return theme.clone();
    }

    if let Some((actual_name, theme)) = theme_set
        .themes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(requested_theme))
    {
        if actual_name.as_str() != requested_theme {
            log::info!(
                "Using syntax theme '{}' for '--code-theme {}'.",
                actual_name,
                requested_theme
            );
        }
        return theme.clone();
    }

    if let Some(builtin_theme) = find_builtin_theme(theme_manager, requested_theme) {
        if !builtin_theme.name.eq_ignore_ascii_case(requested_theme) {
            log::info!(
                "Using built-in theme '{}' for '--code-theme {}'.",
                builtin_theme.name,
                requested_theme
            );
        }
        return build_syntect_theme(builtin_theme);
    }

    log::warn!(
        "Code theme '{}' not found; falling back to '{}'.",
        requested_theme,
        main_theme.name
    );
    build_syntect_theme(main_theme)
}

fn find_builtin_theme<'a>(
    theme_manager: &'a ThemeManager,
    requested_theme: &str,
) -> Option<&'a Theme> {
    if let Ok(theme) = theme_manager.get_theme(requested_theme) {
        return Some(theme);
    }

    let requested_lower = requested_theme.to_ascii_lowercase();
    theme_manager
        .list_themes()
        .into_iter()
        .find(|name| name.to_ascii_lowercase() == requested_lower)
        .and_then(|matched_name| theme_manager.get_theme(matched_name).ok())
}
