use crate::error::MdvError;
use crate::terminal::{AnsiStyle, ansi256_to_rgb, calculate_luminosity};
use crate::user_themes::parse_embedded_theme;
use anyhow::{Context, Result, anyhow, bail};
use crossterm::style::Color as CrosstermColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Serializable color type for themes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Black,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    Grey,
    DarkGrey,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    AnsiValue(u8),
    Rgb { r: u8, g: u8, b: u8 },
    Reset,
}

impl From<Color> for CrosstermColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => CrosstermColor::Black,
            Color::DarkRed => CrosstermColor::DarkRed,
            Color::DarkGreen => CrosstermColor::DarkGreen,
            Color::DarkYellow => CrosstermColor::DarkYellow,
            Color::DarkBlue => CrosstermColor::DarkBlue,
            Color::DarkMagenta => CrosstermColor::DarkMagenta,
            Color::DarkCyan => CrosstermColor::DarkCyan,
            Color::Grey => CrosstermColor::Grey,
            Color::DarkGrey => CrosstermColor::DarkGrey,
            Color::Red => CrosstermColor::Red,
            Color::Green => CrosstermColor::Green,
            Color::Yellow => CrosstermColor::Yellow,
            Color::Blue => CrosstermColor::Blue,
            Color::Magenta => CrosstermColor::Magenta,
            Color::Cyan => CrosstermColor::Cyan,
            Color::White => CrosstermColor::White,
            Color::AnsiValue(n) => CrosstermColor::AnsiValue(n),
            Color::Rgb { r, g, b } => CrosstermColor::Rgb { r, g, b },
            Color::Reset => CrosstermColor::Reset,
        }
    }
}

/// Theme configuration for markdown rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,

    // Text colors
    pub text: Color,
    pub text_light: Color,
    #[serde(default = "default_line_number_color")]
    pub line_number: Color,
    #[serde(default = "default_line_number_color")]
    pub line_number_separator: Color,

    // Header colors (H1-H6)
    pub h1: Color,
    pub h2: Color,
    pub h3: Color,
    pub h4: Color,
    pub h5: Color,
    pub h6: Color,

    // Special elements
    pub code: Color,
    pub quote: Color,
    pub link: Color,
    pub emphasis: Color,
    pub strong: Color,
    pub strikethrough: Color,

    // Background and borders
    pub highlight_background: Color,
    pub background: Option<Color>,
    pub border: Color,

    // List and table elements
    pub list_marker: Color,
    pub table_header: Color,
    pub table_border: Color,

    // Error and warning
    pub error: Color,
    pub warning: Color,

    // Code syntax highlighting colors
    pub syntax: SyntaxTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub number: Color,
    pub operator: Color,
    pub function: Color,
    pub variable: Color,
    pub type_name: Color,
}

impl Default for Theme {
    fn default() -> Self {
        BUILTIN_THEMES
            .get("terminal")
            .expect("embedded terminal theme must exist")
            .clone()
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Theme::default().syntax
    }
}

const BUILTIN_THEME_FILES: [(&str, &str); 9] = [
    (
        "terminal",
        include_str!("../assets/config/themes/terminal.yaml"),
    ),
    (
        "monokai",
        include_str!("../assets/config/themes/monokai.yaml"),
    ),
    (
        "solarized-dark",
        include_str!("../assets/config/themes/solarized-dark.yaml"),
    ),
    ("nord", include_str!("../assets/config/themes/nord.yaml")),
    (
        "tokyonight",
        include_str!("../assets/config/themes/tokyonight.yaml"),
    ),
    (
        "kanagawa",
        include_str!("../assets/config/themes/kanagawa.yaml"),
    ),
    (
        "gruvbox",
        include_str!("../assets/config/themes/gruvbox.yaml"),
    ),
    (
        "material-ocean",
        include_str!("../assets/config/themes/material-ocean.yaml"),
    ),
    (
        "catppuccin",
        include_str!("../assets/config/themes/catppuccin.yaml"),
    ),
];

static BUILTIN_THEMES: LazyLock<HashMap<String, Theme>> = LazyLock::new(|| {
    BUILTIN_THEME_FILES
        .iter()
        .map(|(name, source)| {
            let theme = parse_embedded_theme(name, source)
                .unwrap_or_else(|error| panic!("invalid embedded theme '{name}': {error:#}"));
            ((*name).to_string(), theme)
        })
        .collect()
});

/// Theme manager for loading and managing themes
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

/// Apply overrides specified as `key=value` pairs (semicolon or newline separated)
pub fn apply_custom_theme(theme: &mut Theme, overrides: &str) -> Result<()> {
    for (key, value) in parse_override_pairs(overrides)? {
        apply_theme_override(theme, &key, &value)
            .with_context(|| format!("Failed to apply override '{}={}'", key, value))?;
    }
    Ok(())
}

/// Apply overrides for syntax highlighting colors using the same format as [`apply_custom_theme`]
pub fn apply_custom_code_theme(theme: &mut Theme, overrides: &str) -> Result<()> {
    for (key, value) in parse_override_pairs(overrides)? {
        apply_code_theme_override(&mut theme.syntax, &key, &value)
            .with_context(|| format!("Failed to apply syntax override '{}={}'", key, value))?;
    }
    Ok(())
}

fn parse_override_pairs(input: &str) -> Result<Vec<(String, String)>> {
    let mut pairs = Vec::new();

    for raw in input.split([';', '\n']) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (key, value) = trimmed
            .split_once('=')
            .ok_or_else(|| anyhow!("Override pair '{}' must contain '='", trimmed))?;

        let key = key.trim();
        let value = value.trim();

        if key.is_empty() {
            bail!("Found empty key in override '{}'.", trimmed);
        }

        if value.is_empty() {
            bail!("Key '{}' has an empty value in override.", key);
        }

        pairs.push((key.to_string(), value.to_string()));
    }

    if pairs.is_empty() {
        bail!("Override string is empty.");
    }

    Ok(pairs)
}

fn apply_theme_override(theme: &mut Theme, key: &str, value: &str) -> Result<()> {
    let normalized_key = normalize_key(key);

    match normalized_key.as_str() {
        "text" => theme.text = parse_color_spec(value)?,
        "text_light" | "textlight" => theme.text_light = parse_color_spec(value)?,
        "line_number" | "linenumber" => theme.line_number = parse_color_spec(value)?,
        "line_number_separator" | "linenumberseparator" => {
            theme.line_number_separator = parse_color_spec(value)?
        }
        "h1" => theme.h1 = parse_color_spec(value)?,
        "h2" => theme.h2 = parse_color_spec(value)?,
        "h3" => theme.h3 = parse_color_spec(value)?,
        "h4" => theme.h4 = parse_color_spec(value)?,
        "h5" => theme.h5 = parse_color_spec(value)?,
        "h6" => theme.h6 = parse_color_spec(value)?,
        "code" => theme.code = parse_color_spec(value)?,
        "quote" => theme.quote = parse_color_spec(value)?,
        "link" => theme.link = parse_color_spec(value)?,
        "emphasis" => theme.emphasis = parse_color_spec(value)?,
        "strong" => theme.strong = parse_color_spec(value)?,
        "strikethrough" | "strike" | "del" => theme.strikethrough = parse_color_spec(value)?,
        "highlight_background" | "highlight_bg" => {
            theme.highlight_background = parse_color_spec(value)?
        }
        "background" | "bg" => {
            if is_none_value(value) {
                theme.background = None;
            } else {
                theme.background = Some(parse_color_spec(value)?);
            }
        }
        "border" => theme.border = parse_color_spec(value)?,
        "list_marker" | "listmarker" => theme.list_marker = parse_color_spec(value)?,
        "table_header" | "tableheader" => theme.table_header = parse_color_spec(value)?,
        "table_border" | "tableborder" => theme.table_border = parse_color_spec(value)?,
        "error" => theme.error = parse_color_spec(value)?,
        "warning" => theme.warning = parse_color_spec(value)?,
        other => bail!("Unknown key for custom theme: '{}'.", other),
    }

    Ok(())
}

fn apply_code_theme_override(syntax: &mut SyntaxTheme, key: &str, value: &str) -> Result<()> {
    let normalized_key = normalize_key(key);

    match normalized_key.as_str() {
        "keyword" => syntax.keyword = parse_color_spec(value)?,
        "string" => syntax.string = parse_color_spec(value)?,
        "comment" => syntax.comment = parse_color_spec(value)?,
        "number" => syntax.number = parse_color_spec(value)?,
        "operator" => syntax.operator = parse_color_spec(value)?,
        "function" => syntax.function = parse_color_spec(value)?,
        "variable" => syntax.variable = parse_color_spec(value)?,
        "type_name" | "typename" | "type" => syntax.type_name = parse_color_spec(value)?,
        other => bail!("Unknown key for custom syntax theme: '{}'.", other),
    }

    Ok(())
}

fn normalize_key(key: &str) -> String {
    key.trim()
        .replace(['-', ' '], "_")
        .replace("__", "_")
        .to_ascii_lowercase()
}

fn default_line_number_color() -> Color {
    Color::Grey
}

pub(crate) fn parse_color_value(value: &str) -> Result<Color> {
    parse_color_spec(value)
}

fn parse_color_spec(value: &str) -> Result<Color> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("Color cannot be an empty string.");
    }

    if trimmed.starts_with('#') {
        return parse_hex_color(trimmed);
    }

    let lower = trimmed.to_ascii_lowercase();

    if let Ok(value) = trimmed.parse::<i16>() {
        if (0..=255).contains(&value) {
            return Ok(Color::AnsiValue(value as u8));
        } else {
            bail!("ANSI value '{}' must be in the range 0..=255.", value);
        }
    }

    if let Some(inner) = lower.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let (r, g, b) = parse_rgb_components(inner)?;
        return Ok(Color::Rgb { r, g, b });
    }

    if trimmed.contains(',') {
        let (r, g, b) = parse_rgb_components(trimmed)?;
        return Ok(Color::Rgb { r, g, b });
    }

    if let Some(inner) = lower
        .strip_prefix("ansi(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let value = inner.trim().parse::<u8>().map_err(|_| {
            anyhow!(
                "Value '{}': expected a number in the range 0..=255 for ansi().",
                inner
            )
        })?;
        return Ok(Color::AnsiValue(value));
    }

    match lower.as_str() {
        "reset" => Ok(Color::Reset),
        name => parse_named_color(name).ok_or_else(|| anyhow!("Unknown color value '{}'.", value)),
    }
}

fn parse_hex_color(value: &str) -> Result<Color> {
    let hex = value.trim_start_matches('#');

    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| anyhow!("Failed to parse R component from '{}'.", value))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| anyhow!("Failed to parse G component from '{}'.", value))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| anyhow!("Failed to parse B component from '{}'.", value))?;
            (r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16)
                .map_err(|_| anyhow!("Failed to parse R component from '{}'.", value))?;
            let g = u8::from_str_radix(&hex[1..2], 16)
                .map_err(|_| anyhow!("Failed to parse G component from '{}'.", value))?;
            let b = u8::from_str_radix(&hex[2..3], 16)
                .map_err(|_| anyhow!("Failed to parse B component from '{}'.", value))?;
            (r * 17, g * 17, b * 17)
        }
        _ => bail!("Color '{}' must contain 3 or 6 hexadecimal digits.", value),
    };

    Ok(Color::Rgb { r, g, b })
}

fn parse_rgb_components(value: &str) -> Result<(u8, u8, u8)> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 3 {
        bail!(
            "Color '{}' must contain three comma-separated RGB components.",
            value
        );
    }

    let mut rgb = [0u8; 3];
    for (idx, part) in parts.iter().enumerate() {
        let component = part.trim();
        let parsed = component
            .parse::<i16>()
            .map_err(|_| anyhow!("Component '{}' must be an integer in 0..=255.", component))?;
        if !(0..=255).contains(&parsed) {
            bail!("Component '{}' is out of range 0..=255.", component);
        }
        rgb[idx] = parsed as u8;
    }

    Ok((rgb[0], rgb[1], rgb[2]))
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name {
        "black" => Some(Color::Black),
        "darkred" => Some(Color::DarkRed),
        "dark_green" | "darkgreen" => Some(Color::DarkGreen),
        "darkyellow" | "dark_yellow" => Some(Color::DarkYellow),
        "darkblue" | "dark_blue" => Some(Color::DarkBlue),
        "darkmagenta" | "dark_magenta" => Some(Color::DarkMagenta),
        "darkcyan" | "dark_cyan" => Some(Color::DarkCyan),
        "grey" | "gray" => Some(Color::Grey),
        "darkgrey" | "darkgray" | "dark_grey" | "dark_gray" => Some(Color::DarkGrey),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        _ => None,
    }
}

fn is_none_value(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("null")
}

/// Calculate overall luminosity of a theme
fn calculate_theme_luminosity(theme: &Theme) -> f64 {
    let colors = [&theme.h1, &theme.h2, &theme.h3, &theme.h4, &theme.h5];
    let mut total_lum = 0.0;
    let mut count = 0;

    for color in colors {
        if let Some((r, g, b)) = color_to_rgb(color) {
            total_lum += calculate_luminosity(r, g, b);
            count += 1;
        }
    }

    if count > 0 {
        total_lum / count as f64
    } else {
        0.5 // Default middle luminosity
    }
}

/// Convert Color to RGB tuple if possible
fn color_to_rgb(color: &Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::AnsiValue(n) => Some(ansi256_to_rgb(*n)),
        Color::Rgb { r, g, b } => Some((*r, *g, *b)),
        Color::Black => Some((0, 0, 0)),
        Color::DarkRed => Some((128, 0, 0)),
        Color::DarkGreen => Some((0, 128, 0)),
        Color::DarkYellow => Some((128, 128, 0)),
        Color::DarkBlue => Some((0, 0, 128)),
        Color::DarkMagenta => Some((128, 0, 128)),
        Color::DarkCyan => Some((0, 128, 128)),
        Color::Grey => Some((192, 192, 192)),
        Color::DarkGrey => Some((128, 128, 128)),
        Color::Red => Some((255, 0, 0)),
        Color::Green => Some((0, 255, 0)),
        Color::Yellow => Some((255, 255, 0)),
        Color::Blue => Some((0, 0, 255)),
        Color::Magenta => Some((255, 0, 255)),
        Color::Cyan => Some((0, 255, 255)),
        Color::White => Some((255, 255, 255)),
        Color::Reset => None,
    }
}

/// List all available themes from the given manager.
pub fn list_themes(manager: &ThemeManager) {
    let themes = manager.get_themes_by_luminosity();

    println!("Available themes:");
    println!();

    for (name, theme, luminosity) in themes {
        println!(
            "  {:<20} - {} (luminosity: {:.3})",
            name, theme.description, luminosity
        );
    }
}

/// Create a style from theme colors
pub fn create_style(theme: &Theme, element: ThemeElement) -> AnsiStyle {
    let color = match element {
        ThemeElement::Text => &theme.text,
        ThemeElement::TextLight => &theme.text_light,
        ThemeElement::LineNumber => &theme.line_number,
        ThemeElement::LineNumberSeparator => &theme.line_number_separator,
        ThemeElement::H1 => &theme.h1,
        ThemeElement::H2 => &theme.h2,
        ThemeElement::H3 => &theme.h3,
        ThemeElement::H4 => &theme.h4,
        ThemeElement::H5 => &theme.h5,
        ThemeElement::H6 => &theme.h6,
        ThemeElement::Code => &theme.code,
        ThemeElement::Quote => &theme.quote,
        ThemeElement::Link => &theme.link,
        ThemeElement::Emphasis => &theme.emphasis,
        ThemeElement::Strong => &theme.strong,
        ThemeElement::Strikethrough => &theme.strikethrough,
        ThemeElement::Underline => &theme.text,
        ThemeElement::Border => &theme.border,
        ThemeElement::ListMarker => &theme.list_marker,
        ThemeElement::TableHeader => &theme.table_header,
        ThemeElement::TableBorder => &theme.table_border,
        ThemeElement::Error => &theme.error,
        ThemeElement::Warning => &theme.warning,
    };

    let mut style = AnsiStyle::new().fg(color.clone().into());

    match element {
        ThemeElement::Strong | ThemeElement::H1 => style = style.bold(),
        ThemeElement::Emphasis => style = style.italic(),
        ThemeElement::Strikethrough => style = style.strikethrough(),
        ThemeElement::Underline => style = style.underline(),
        _ => {}
    }

    style
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeElement {
    Text,
    TextLight,
    LineNumber,
    LineNumberSeparator,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Code,
    Quote,
    Link,
    Emphasis,
    Strong,
    Strikethrough,
    Underline,
    Border,
    ListMarker,
    TableHeader,
    TableBorder,
    Error,
    Warning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager() {
        let manager = ThemeManager::new();
        assert!(manager.get_theme("terminal").is_ok());
        assert!(manager.get_theme("monokai").is_ok());
        assert!(manager.get_theme("catppuccin").is_ok());
        assert!(manager.get_theme("Catppuccin").is_ok());
        assert!(manager.get_theme("Terminal").is_ok());
        assert!(manager.get_theme("MoNoKaI").is_ok());
        assert!(manager.get_theme("nonexistent").is_err());
    }

    #[test]
    fn test_theme_luminosity() {
        let theme = Theme::default();
        let lum = calculate_theme_luminosity(&theme);
        assert!((0.0..=1.0).contains(&lum));
    }

    #[test]
    fn test_create_style() {
        let theme = Theme::default();
        let style = create_style(&theme, ThemeElement::H1);
        // Should have bold attribute for H1
        assert!(style.bold);
    }

    #[test]
    fn test_apply_custom_theme_overrides() {
        let mut theme = Theme::default();
        apply_custom_theme(
            &mut theme,
            "h1=#ffffff; link=187,154,247; background=none; strong=rgb(10,20,30); highlight_bg=#112233; line_number=#010203; line_number_separator=#040506",
        )
        .expect("custom theme overrides should be applied");

        assert!(matches!(
            theme.h1,
            Color::Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        ));
        assert!(matches!(
            theme.link,
            Color::Rgb {
                r: 187,
                g: 154,
                b: 247
            }
        ));
        assert!(matches!(
            theme.strong,
            Color::Rgb {
                r: 10,
                g: 20,
                b: 30
            }
        ));
        assert!(theme.background.is_none());
        assert!(matches!(
            theme.highlight_background,
            Color::Rgb {
                r: 0x11,
                g: 0x22,
                b: 0x33
            }
        ));
        assert_eq!(theme.line_number, Color::Rgb { r: 1, g: 2, b: 3 });
        assert_eq!(theme.line_number_separator, Color::Rgb { r: 4, g: 5, b: 6 });
    }

    #[test]
    fn test_apply_custom_code_theme_overrides() {
        let mut theme = Theme::default();
        apply_custom_code_theme(&mut theme, "keyword=#123456;type=42,42,42")
            .expect("custom code theme overrides should be applied");

        assert!(matches!(
            theme.syntax.keyword,
            Color::Rgb {
                r: 18,
                g: 52,
                b: 86
            }
        ));
        assert!(matches!(
            theme.syntax.type_name,
            Color::Rgb {
                r: 42,
                g: 42,
                b: 42
            }
        ));
    }

    #[test]
    fn removed_code_block_override_is_rejected() {
        let mut theme = Theme::default();
        let error = apply_custom_theme(&mut theme, "code_block=#ffffff")
            .expect_err("removed code_block override must be rejected");
        let error_chain = format!("{error:#}");
        assert!(
            error_chain.contains("Unknown key for custom theme: 'code_block'."),
            "unexpected error: {error_chain}"
        );
    }

    #[test]
    fn test_apply_custom_theme_plain_ansi_value() {
        let mut theme = Theme::default();
        apply_custom_theme(&mut theme, "border=123").expect("plain ANSI value should be accepted");
        assert!(matches!(theme.border, Color::AnsiValue(123)));
    }

    #[test]
    fn test_apply_custom_theme_ansi_function() {
        let mut theme = Theme::default();
        apply_custom_theme(&mut theme, "border=ansi(42)")
            .expect("ansi() notation should be accepted");
        assert!(matches!(theme.border, Color::AnsiValue(42)));
    }

    #[test]
    fn test_apply_custom_theme_rejects_ansi_without_parens() {
        let mut theme = Theme::default();
        let result = apply_custom_theme(&mut theme, "border=ansi42");
        assert!(result.is_err());
    }
}
