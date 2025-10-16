use crate::config::mdv_no_color_override;
use crate::error::MdvError;
use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    style::Color,
    terminal::{Clear, ClearType, size},
};
#[allow(unused_imports)]
use std::io::{self, Write};

/// ANSI color codes and terminal utilities
pub struct Terminal {
    pub width: usize,
    pub height: usize,
    pub supports_color: bool,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let (width, height) = size().map_err(|e| MdvError::TerminalError(e.to_string()))?;

        let supports_color = supports_color();

        Ok(Self {
            width: width as usize,
            height: height as usize,
            supports_color,
        })
    }

    pub fn clear_screen(&self) -> Result<()> {
        io::stdout()
            .execute(Clear(ClearType::All))
            .map_err(|e| MdvError::TerminalError(e.to_string()))?;
        Ok(())
    }
}

/// Check if terminal supports color output
pub fn supports_color() -> bool {
    // Check various environment variables that indicate color support
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("color") || term.contains("256") || term == "xterm" {
            return true;
        }
    }

    if std::env::var("COLORTERM").is_ok() {
        return true;
    }

    // Honor mdv-specific override for disabling color output
    if let Some(no_color_override) = mdv_no_color_override() {
        if no_color_override {
            return false;
        }
    }

    // Default to true for most modern terminals
    true
}

/// ANSI color and style utilities
#[derive(Debug, Clone)]
pub struct AnsiStyle {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Default for AnsiStyle {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

impl AnsiStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn apply(&self, text: &str, no_colors: bool) -> String {
        if no_colors {
            return text.to_string();
        }

        let mut result = String::new();

        // Apply foreground color
        if let Some(fg) = self.fg_color {
            match fg {
                Color::AnsiValue(n) => {
                    // Use 256-color palette format for foreground
                    result.push_str(&format!("\x1b[38;5;{}m", n));
                }
                Color::Rgb { r, g, b } => {
                    // Use truecolor escape for foreground
                    result.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                }
                _ => {
                    result.push_str(&format!("\x1b[{}m", color_to_ansi_fg(fg)));
                }
            }
        }

        // Apply background color
        if let Some(bg) = self.bg_color {
            match bg {
                Color::AnsiValue(n) => {
                    // Use 256-color palette format for background
                    result.push_str(&format!("\x1b[48;5;{}m", n));
                }
                Color::Rgb { r, g, b } => {
                    // Use truecolor escape for background
                    result.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
                }
                _ => {
                    result.push_str(&format!("\x1b[{}m", color_to_ansi_bg(bg)));
                }
            }
        }

        // Apply attributes
        if self.bold {
            result.push_str("\x1b[1m");
        }
        if self.italic {
            result.push_str("\x1b[3m");
        }
        if self.underline {
            result.push_str("\x1b[4m");
        }
        if self.strikethrough {
            result.push_str("\x1b[9m");
        }

        result.push_str(text);

        // Reset all styles
        result.push_str("\x1b[0m");

        result
    }
}

fn color_to_ansi_fg(color: Color) -> u8 {
    match color {
        Color::Black => 30,
        Color::DarkRed => 31,
        Color::DarkGreen => 32,
        Color::DarkYellow => 33,
        Color::DarkBlue => 34,
        Color::DarkMagenta => 35,
        Color::DarkCyan => 36,
        Color::Grey => 37,
        Color::DarkGrey => 90,
        Color::Red => 91,
        Color::Green => 92,
        Color::Yellow => 93,
        Color::Blue => 94,
        Color::Magenta => 95,
        Color::Cyan => 96,
        Color::White => 97,
        Color::AnsiValue(n) => n,
        Color::Rgb { .. } => unreachable!("RGB colors are handled as truecolor sequences"),
        Color::Reset => 39,
    }
}

fn color_to_ansi_bg(color: Color) -> u8 {
    match color {
        Color::Black => 40,
        Color::DarkRed => 41,
        Color::DarkGreen => 42,
        Color::DarkYellow => 43,
        Color::DarkBlue => 44,
        Color::DarkMagenta => 45,
        Color::DarkCyan => 46,
        Color::Grey => 47,
        Color::DarkGrey => 100,
        Color::Red => 101,
        Color::Green => 102,
        Color::Yellow => 103,
        Color::Blue => 104,
        Color::Magenta => 105,
        Color::Cyan => 106,
        Color::White => 107,
        Color::AnsiValue(n) => n + 10, // Background colors are +10 from foreground
        Color::Rgb { .. } => unreachable!("RGB colors are handled as truecolor sequences"),
        Color::Reset => 49,
    }
}

/// Convert 256-color palette index to RGB approximation
pub fn ansi256_to_rgb(color: u8) -> (u8, u8, u8) {
    match color {
        // Standard colors (0-15)
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Dark Red
        2 => (0, 128, 0),      // Dark Green
        3 => (128, 128, 0),    // Dark Yellow
        4 => (0, 0, 128),      // Dark Blue
        5 => (128, 0, 128),    // Dark Magenta
        6 => (0, 128, 128),    // Dark Cyan
        7 => (192, 192, 192),  // Light Gray
        8 => (128, 128, 128),  // Dark Gray
        9 => (255, 0, 0),      // Red
        10 => (0, 255, 0),     // Green
        11 => (255, 255, 0),   // Yellow
        12 => (0, 0, 255),     // Blue
        13 => (255, 0, 255),   // Magenta
        14 => (0, 255, 255),   // Cyan
        15 => (255, 255, 255), // White

        // 216-color cube (16-231)
        16..=231 => {
            let n = color - 16;
            let r = n / 36;
            let g = (n % 36) / 6;
            let b = n % 6;

            let to_rgb = |c| if c == 0 { 0 } else { 55 + c * 40 };
            (to_rgb(r), to_rgb(g), to_rgb(b))
        }

        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Calculate luminosity of a color for theme sorting
pub fn calculate_luminosity(r: u8, g: u8, b: u8) -> f64 {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    // Use relative luminance formula
    0.299 * r + 0.587 * g + 0.114 * b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;

    #[test]
    fn test_ansi_style() {
        let style = AnsiStyle::new().fg(Color::Red).bold();
        let result = style.apply("test", false);
        assert!(result.contains("test"));
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_no_colors() {
        let style = AnsiStyle::new().fg(Color::Red).bold();
        let result = style.apply("test", true);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_ansi256_to_rgb() {
        assert_eq!(ansi256_to_rgb(0), (0, 0, 0));
        assert_eq!(ansi256_to_rgb(15), (255, 255, 255));
        assert_eq!(ansi256_to_rgb(196), (255, 0, 0)); // Bright red in 216-color cube
    }

    #[test]
    fn apply_emits_truecolor_foreground_sequence() {
        let style = AnsiStyle::new().fg(Color::Rgb {
            r: 10,
            g: 20,
            b: 30,
        });
        let applied = style.apply("demo", false);
        assert!(applied.starts_with("\x1b[38;2;10;20;30m"));
        assert!(applied.ends_with("demo\x1b[0m"));
    }

    #[test]
    fn apply_emits_truecolor_background_sequence() {
        let style = AnsiStyle::new().bg(Color::Rgb { r: 1, g: 2, b: 3 });
        let applied = style.apply("demo", false);
        assert!(applied.starts_with("\x1b[48;2;1;2;3m"));
        assert!(applied.ends_with("demo\x1b[0m"));
    }
}
