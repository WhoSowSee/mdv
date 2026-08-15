use super::*;

/// Loads and manages built-in and user themes.
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: BUILTIN_THEMES.clone(),
        }
    }

    pub fn get_theme(&self, name: &str) -> Result<&Theme> {
        if let Some(theme) = self.themes.get(name) {
            return Ok(theme);
        }

        self.themes
            .iter()
            .find(|(stored_name, _)| stored_name.eq_ignore_ascii_case(name))
            .map(|(_, theme)| theme)
            .ok_or_else(|| MdvError::ThemeError(format!("Theme '{}' not found", name)).into())
    }

    pub fn list_themes(&self) -> Vec<&String> {
        let mut names: Vec<&String> = self.themes.keys().collect();
        names.sort();
        names
    }

    pub fn add_theme(&mut self, theme: Theme) {
        let key_to_remove = self
            .themes
            .keys()
            .find(|existing| existing.eq_ignore_ascii_case(&theme.name) && *existing != &theme.name)
            .cloned();

        if let Some(existing_key) = key_to_remove {
            self.themes.remove(&existing_key);
        }

        self.themes.insert(theme.name.clone(), theme);
    }

    pub fn load_theme_from_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = serde_yaml::from_str(&content)
            .map_err(|e| MdvError::ThemeError(format!("Failed to parse YAML theme file: {}", e)))?;

        self.add_theme(theme);
        Ok(())
    }

    /// Get themes sorted by luminosity (for theme browsing)
    pub fn get_themes_by_luminosity(&self) -> Vec<(&String, &Theme, f64)> {
        let mut themes_with_lum: Vec<(&String, &Theme, f64)> = self
            .themes
            .iter()
            .map(|(name, theme)| {
                let lum = calculate_theme_luminosity(theme);
                (name, theme, lum)
            })
            .collect();

        themes_with_lum.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        themes_with_lum
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
