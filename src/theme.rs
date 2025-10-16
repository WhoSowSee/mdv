use crate::error::MdvError;
use crate::terminal::{AnsiStyle, ansi256_to_rgb, calculate_luminosity};
use anyhow::{Context, Result, anyhow, bail};
use crossterm::style::Color as CrosstermColor;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serializable color type for themes
#[derive(Debug, Clone, Serialize, Deserialize)]
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

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

/// Theme configuration for markdown rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,

    // Text colors
    pub text: Color,
    pub text_light: Color,

    // Header colors (H1-H6)
    pub h1: Color,
    pub h2: Color,
    pub h3: Color,
    pub h4: Color,
    pub h5: Color,
    pub h6: Color,

    // Special elements
    pub code: Color,
    pub code_block: Color,
    pub quote: Color,
    pub link: Color,
    pub emphasis: Color,
    pub strong: Color,
    pub strikethrough: Color,

    // Background and borders
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
        Self {
            name: "terminal".to_string(),
            description: "Terminal theme with standard colors".to_string(),
            text: Color::White,
            text_light: Color::Grey,
            h1: Color::Red,
            h2: Color::Green,
            h3: Color::Yellow,
            h4: Color::Blue,
            h5: Color::Magenta,
            h6: Color::Cyan,
            code: Color::AnsiValue(102),
            code_block: Color::AnsiValue(102),
            quote: Color::AnsiValue(109),
            link: Color::Blue,
            emphasis: Color::Yellow,
            strong: Color::Red,
            strikethrough: Color::DarkGrey,
            background: None,
            border: Color::Grey,
            list_marker: Color::Green,
            table_header: Color::Yellow,
            table_border: Color::Grey,
            error: Color::Red,
            warning: Color::Yellow,
            syntax: SyntaxTheme::default(),
        }
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            keyword: Color::AnsiValue(117),
            string: Color::AnsiValue(109),
            comment: Color::AnsiValue(59),
            number: Color::AnsiValue(109),
            operator: Color::AnsiValue(65),
            function: Color::AnsiValue(153),
            variable: Color::AnsiValue(231),
            type_name: Color::AnsiValue(117),
        }
    }
}

/// Built-in themes
static BUILTIN_THEMES: Lazy<HashMap<String, Theme>> = Lazy::new(|| {
    let mut themes = HashMap::new();

    // Terminal theme
    themes.insert("terminal".to_string(), Theme::default());

    // Monokai theme
    themes.insert(
        "monokai".to_string(),
        Theme {
            name: "monokai".to_string(),
            description: "Monokai color scheme".to_string(),
            text: rgb(248, 248, 242),
            text_light: rgb(117, 113, 94),
            h1: rgb(249, 38, 114),
            h2: rgb(166, 226, 46),
            h3: rgb(230, 219, 116),
            h4: rgb(102, 217, 239),
            h5: rgb(253, 151, 31),
            h6: rgb(174, 129, 255),
            code: rgb(230, 219, 116),
            code_block: rgb(248, 248, 242),
            quote: rgb(117, 113, 94),
            link: rgb(102, 217, 239),
            emphasis: rgb(253, 151, 31),
            strong: rgb(249, 38, 114),
            strikethrough: rgb(117, 113, 94),
            background: Some(rgb(39, 40, 34)),
            border: rgb(73, 72, 62),
            list_marker: rgb(166, 226, 46),
            table_header: rgb(253, 151, 31),
            table_border: rgb(73, 72, 62),
            error: rgb(249, 38, 114),
            warning: rgb(253, 151, 31),
            syntax: SyntaxTheme {
                keyword: rgb(249, 38, 114),
                string: rgb(230, 219, 116),
                comment: rgb(117, 113, 94),
                number: rgb(174, 129, 255),
                operator: rgb(249, 38, 114),
                function: rgb(166, 226, 46),
                variable: rgb(248, 248, 242),
                type_name: rgb(102, 217, 239),
            },
        },
    );

    // Solarized Dark theme
    themes.insert(
        "solarized-dark".to_string(),
        Theme {
            name: "solarized-dark".to_string(),
            description: "Solarized Dark color scheme".to_string(),
            text: rgb(131, 148, 150),
            text_light: rgb(88, 110, 117),
            h1: rgb(220, 50, 47),
            h2: rgb(203, 75, 22),
            h3: rgb(181, 137, 0),
            h4: rgb(38, 139, 210),
            h5: rgb(108, 113, 196),
            h6: rgb(42, 161, 152),
            code: rgb(42, 161, 152),
            code_block: rgb(131, 148, 150),
            quote: rgb(88, 110, 117),
            link: rgb(38, 139, 210),
            emphasis: rgb(203, 75, 22),
            strong: rgb(220, 50, 47),
            strikethrough: rgb(88, 110, 117),
            background: Some(rgb(0, 43, 54)),
            border: rgb(88, 110, 117),
            list_marker: rgb(133, 153, 0),
            table_header: rgb(181, 137, 0),
            table_border: rgb(88, 110, 117),
            error: rgb(220, 50, 47),
            warning: rgb(181, 137, 0),
            syntax: SyntaxTheme {
                keyword: rgb(133, 153, 0),
                string: rgb(42, 161, 152),
                comment: rgb(88, 110, 117),
                number: rgb(181, 137, 0),
                operator: rgb(220, 50, 47),
                function: rgb(38, 139, 210),
                variable: rgb(131, 148, 150),
                type_name: rgb(108, 113, 196),
            },
        },
    );

    // Nord theme
    themes.insert(
        "nord".to_string(),
        Theme {
            name: "nord".to_string(),
            description: "Nord color scheme".to_string(),
            text: rgb(236, 239, 244),
            text_light: rgb(216, 222, 233),
            h1: rgb(136, 192, 208),
            h2: rgb(143, 188, 187),
            h3: rgb(129, 161, 193),
            h4: rgb(94, 129, 172),
            h5: rgb(191, 97, 106),
            h6: rgb(208, 135, 112),
            code: rgb(235, 203, 139),
            code_block: rgb(236, 239, 244),
            quote: rgb(76, 86, 106),
            link: rgb(136, 192, 208),
            emphasis: rgb(163, 190, 140),
            strong: rgb(180, 142, 173),
            strikethrough: rgb(67, 76, 94),
            background: Some(rgb(46, 52, 64)),
            border: rgb(76, 86, 106),
            list_marker: rgb(163, 190, 140),
            table_header: rgb(136, 192, 208),
            table_border: rgb(76, 86, 106),
            error: rgb(191, 97, 106),
            warning: rgb(235, 203, 139),
            syntax: SyntaxTheme {
                keyword: rgb(129, 161, 193),
                string: rgb(163, 190, 140),
                comment: rgb(76, 86, 106),
                number: rgb(180, 142, 173),
                operator: rgb(129, 161, 193),
                function: rgb(136, 192, 208),
                variable: rgb(236, 239, 244),
                type_name: rgb(143, 188, 187),
            },
        },
    );

    // Tokyonight theme
    themes.insert(
        "tokyonight".to_string(),
        Theme {
            name: "tokyonight".to_string(),
            description: "Tokyonight color scheme".to_string(),
            text: rgb(192, 202, 245),
            text_light: rgb(169, 177, 214),
            h1: rgb(122, 162, 247),
            h2: rgb(158, 206, 106),
            h3: rgb(187, 154, 247),
            h4: rgb(125, 207, 255),
            h5: rgb(247, 118, 142),
            h6: rgb(224, 175, 104),
            code: rgb(255, 158, 100),
            code_block: rgb(192, 202, 245),
            quote: rgb(59, 66, 97),
            link: rgb(125, 207, 255),
            emphasis: rgb(169, 177, 214),
            strong: rgb(122, 162, 247),
            strikethrough: rgb(84, 92, 126),
            background: Some(rgb(26, 27, 38)),
            border: rgb(59, 66, 97),
            list_marker: rgb(158, 206, 106),
            table_header: rgb(125, 207, 255),
            table_border: rgb(59, 66, 97),
            error: rgb(247, 118, 142),
            warning: rgb(224, 175, 104),
            syntax: SyntaxTheme {
                keyword: rgb(122, 162, 247),
                string: rgb(158, 206, 106),
                comment: rgb(86, 95, 137),
                number: rgb(255, 158, 100),
                operator: rgb(125, 207, 255),
                function: rgb(187, 154, 247),
                variable: rgb(192, 202, 245),
                type_name: rgb(224, 175, 104),
            },
        },
    );

    // Kanagawa theme
    themes.insert(
        "kanagawa".to_string(),
        Theme {
            name: "kanagawa".to_string(),
            description: "Kanagawa color scheme".to_string(),
            text: rgb(220, 215, 186),
            text_light: rgb(200, 192, 147),
            h1: rgb(126, 156, 216),
            h2: rgb(122, 168, 159),
            h3: rgb(147, 138, 169),
            h4: rgb(149, 127, 184),
            h5: rgb(255, 160, 102),
            h6: rgb(228, 104, 118),
            code: rgb(192, 163, 110),
            code_block: rgb(220, 215, 186),
            quote: rgb(84, 84, 109),
            link: rgb(126, 156, 216),
            emphasis: rgb(200, 192, 147),
            strong: rgb(147, 138, 169),
            strikethrough: rgb(114, 113, 105),
            background: Some(rgb(31, 31, 40)),
            border: rgb(42, 42, 55),
            list_marker: rgb(122, 168, 159),
            table_header: rgb(200, 192, 147),
            table_border: rgb(42, 42, 55),
            error: rgb(228, 104, 118),
            warning: rgb(255, 158, 59),
            syntax: SyntaxTheme {
                keyword: rgb(126, 156, 216),
                string: rgb(152, 187, 108),
                comment: rgb(114, 113, 105),
                number: rgb(255, 160, 102),
                operator: rgb(147, 138, 169),
                function: rgb(122, 168, 159),
                variable: rgb(220, 215, 186),
                type_name: rgb(192, 163, 110),
            },
        },
    );

    // Gruvbox theme
    themes.insert(
        "gruvbox".to_string(),
        Theme {
            name: "gruvbox".to_string(),
            description: "Gruvbox Dark color scheme".to_string(),
            text: rgb(235, 219, 178),
            text_light: rgb(168, 153, 132),
            h1: rgb(250, 189, 47),
            h2: rgb(184, 187, 38),
            h3: rgb(142, 192, 124),
            h4: rgb(131, 165, 152),
            h5: rgb(211, 134, 155),
            h6: rgb(254, 128, 25),
            code: rgb(142, 192, 124),
            code_block: rgb(60, 56, 54),
            quote: rgb(146, 131, 116),
            link: rgb(131, 165, 152),
            emphasis: rgb(211, 134, 155),
            strong: rgb(251, 73, 52),
            strikethrough: rgb(102, 92, 84),
            background: Some(rgb(40, 40, 40)),
            border: rgb(102, 92, 84),
            list_marker: rgb(184, 187, 38),
            table_header: rgb(184, 187, 38),
            table_border: rgb(102, 92, 84),
            error: rgb(251, 73, 52),
            warning: rgb(254, 128, 25),
            syntax: SyntaxTheme {
                keyword: rgb(251, 73, 52),
                string: rgb(184, 187, 38),
                comment: rgb(146, 131, 116),
                number: rgb(211, 134, 155),
                operator: rgb(254, 128, 25),
                function: rgb(142, 192, 124),
                variable: rgb(235, 219, 178),
                type_name: rgb(131, 165, 152),
            },
        },
    );

    // Material Ocean theme
    themes.insert(
        "material-ocean".to_string(),
        Theme {
            name: "material-ocean".to_string(),
            description: "Material Theme Ocean color scheme".to_string(),
            text: rgb(238, 255, 255),
            text_light: rgb(176, 190, 197),
            h1: rgb(130, 170, 255),
            h2: rgb(128, 203, 196),
            h3: rgb(195, 232, 141),
            h4: rgb(255, 203, 107),
            h5: rgb(247, 140, 108),
            h6: rgb(199, 146, 234),
            code: rgb(255, 203, 107),
            code_block: rgb(238, 255, 255),
            quote: rgb(84, 110, 122),
            link: rgb(130, 170, 255),
            emphasis: rgb(247, 140, 108),
            strong: rgb(199, 146, 234),
            strikethrough: rgb(84, 110, 122),
            background: Some(rgb(15, 17, 26)),
            border: rgb(28, 34, 48),
            list_marker: rgb(195, 232, 141),
            table_header: rgb(130, 170, 255),
            table_border: rgb(28, 34, 48),
            error: rgb(240, 113, 120),
            warning: rgb(255, 203, 107),
            syntax: SyntaxTheme {
                keyword: rgb(199, 146, 234),
                string: rgb(195, 232, 141),
                comment: rgb(84, 110, 122),
                number: rgb(247, 140, 108),
                operator: rgb(137, 221, 255),
                function: rgb(130, 170, 255),
                variable: rgb(238, 255, 255),
                type_name: rgb(128, 203, 196),
            },
        },
    );

    // Catppucin theme
    themes.insert(
        "catppucin".to_string(),
        Theme {
            name: "catppucin".to_string(),
            description: "Catppucin color scheme".to_string(),
            text: rgb(205, 214, 244),
            text_light: rgb(186, 194, 222),
            h1: rgb(180, 190, 254),
            h2: rgb(137, 180, 250),
            h3: rgb(148, 226, 213),
            h4: rgb(166, 227, 161),
            h5: rgb(249, 226, 175),
            h6: rgb(242, 205, 205),
            code: rgb(245, 194, 231),
            code_block: rgb(205, 214, 244),
            quote: rgb(108, 112, 134),
            link: rgb(137, 220, 235),
            emphasis: rgb(245, 194, 231),
            strong: rgb(203, 166, 247),
            strikethrough: rgb(108, 112, 134),
            background: Some(rgb(30, 30, 46)),
            border: rgb(49, 50, 68),
            list_marker: rgb(166, 227, 161),
            table_header: rgb(137, 180, 250),
            table_border: rgb(49, 50, 68),
            error: rgb(243, 139, 168),
            warning: rgb(250, 179, 135),
            syntax: SyntaxTheme {
                keyword: rgb(203, 166, 247),
                string: rgb(166, 227, 161),
                comment: rgb(108, 112, 134),
                number: rgb(250, 179, 135),
                operator: rgb(137, 220, 235),
                function: rgb(137, 180, 250),
                variable: rgb(205, 214, 244),
                type_name: rgb(148, 226, 213),
            },
        },
    );

    themes
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
        self.themes
            .get(name)
            .ok_or_else(|| MdvError::ThemeError(format!("Theme '{}' not found", name)).into())
    }

    pub fn list_themes(&self) -> Vec<&String> {
        let mut names: Vec<&String> = self.themes.keys().collect();
        names.sort();
        names
    }

    pub fn add_theme(&mut self, theme: Theme) {
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

    for raw in input.split(|c| c == ';' || c == '\n') {
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
        "h1" => theme.h1 = parse_color_spec(value)?,
        "h2" => theme.h2 = parse_color_spec(value)?,
        "h3" => theme.h3 = parse_color_spec(value)?,
        "h4" => theme.h4 = parse_color_spec(value)?,
        "h5" => theme.h5 = parse_color_spec(value)?,
        "h6" => theme.h6 = parse_color_spec(value)?,
        "code" => theme.code = parse_color_spec(value)?,
        "code_block" | "codeblock" => theme.code_block = parse_color_spec(value)?,
        "quote" => theme.quote = parse_color_spec(value)?,
        "link" => theme.link = parse_color_spec(value)?,
        "emphasis" => theme.emphasis = parse_color_spec(value)?,
        "strong" => theme.strong = parse_color_spec(value)?,
        "strikethrough" | "strike" | "del" => theme.strikethrough = parse_color_spec(value)?,
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

/// List all available themes
pub fn list_themes() {
    let manager = ThemeManager::new();
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
        ThemeElement::H1 => &theme.h1,
        ThemeElement::H2 => &theme.h2,
        ThemeElement::H3 => &theme.h3,
        ThemeElement::H4 => &theme.h4,
        ThemeElement::H5 => &theme.h5,
        ThemeElement::H6 => &theme.h6,
        ThemeElement::Code => &theme.code,
        ThemeElement::CodeBlock => &theme.text, // Use normal text color for code blocks
        ThemeElement::Quote => &theme.quote,
        ThemeElement::Link => &theme.link,
        ThemeElement::Emphasis => &theme.emphasis,
        ThemeElement::Strong => &theme.strong,
        ThemeElement::Strikethrough => &theme.strikethrough,
        ThemeElement::Border => &theme.border,
        ThemeElement::ListMarker => &theme.list_marker,
        ThemeElement::TableHeader => &theme.table_header,
        ThemeElement::TableBorder => &theme.table_border,
        ThemeElement::Error => &theme.error,
        ThemeElement::Warning => &theme.warning,
    };

    let mut style = AnsiStyle::new();

    // For inline code, use foreground color only (no background)
    // For code blocks, use normal text color (no special styling)
    match element {
        ThemeElement::Code => {
            // Inline code: foreground color only, no background
            style = style.fg(color.clone().into());
        }
        ThemeElement::CodeBlock => {
            // Code block: use normal text color, no background, no special styling
            style = style.fg(color.clone().into());
        }
        _ => {
            // All other elements: use foreground color
            style = style.fg(color.clone().into());
        }
    }

    // Add attributes for specific elements
    match element {
        ThemeElement::Strong | ThemeElement::H1 => style = style.bold(),
        ThemeElement::Emphasis => style = style.italic(),
        ThemeElement::Strikethrough => style = style.strikethrough(),
        _ => {}
    }

    style
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeElement {
    Text,
    TextLight,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Code,
    CodeBlock,
    Quote,
    Link,
    Emphasis,
    Strong,
    Strikethrough,
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
        assert!(manager.get_theme("nonexistent").is_err());
    }

    #[test]
    fn test_theme_luminosity() {
        let theme = Theme::default();
        let lum = calculate_theme_luminosity(&theme);
        assert!(lum >= 0.0 && lum <= 1.0);
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
            "h1=#ffffff; link=187,154,247; background=none; strong=rgb(10,20,30)",
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
    fn test_apply_custom_theme_invalid_key() {
        let mut theme = Theme::default();
        let result = apply_custom_theme(&mut theme, "unknown=#ffffff");
        assert!(result.is_err());
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
