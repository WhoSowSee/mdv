use crossterm::style::Color;

/// ANSI color and style utilities
#[derive(Debug, Clone, Default)]
pub struct AnsiStyle {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
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

        if let Some(fg) = self.fg_color {
            push_color_sequence(&mut result, fg, 0);
        }

        if let Some(bg) = self.bg_color {
            push_color_sequence(&mut result, bg, 10);
        }

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
        result.push_str("\x1b[0m");

        result
    }
}

/// `offset` is 0 for foreground and 10 for background: every ANSI color code pairs that way.
fn push_color_sequence(out: &mut String, color: Color, offset: u8) {
    match color {
        Color::AnsiValue(n) => out.push_str(&format!("\x1b[{};5;{}m", 38 + offset, n)),
        Color::Rgb { r, g, b } => {
            out.push_str(&format!("\x1b[{};2;{};{};{}m", 38 + offset, r, g, b))
        }
        named => out.push_str(&format!("\x1b[{}m", color_to_ansi_code(named) + offset)),
    }
}

fn color_to_ansi_code(color: Color) -> u8 {
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
        Color::Reset => 39,
        Color::AnsiValue(_) | Color::Rgb { .. } => {
            unreachable!("indexed and RGB colors emit their own sequences")
        }
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

    #[test]
    fn apply_emits_named_indexed_and_reset_color_sequences() {
        assert_eq!(
            AnsiStyle::new()
                .fg(Color::DarkRed)
                .bg(Color::Blue)
                .apply("demo", false),
            "\x1b[31m\x1b[104mdemo\x1b[0m"
        );
        assert_eq!(
            AnsiStyle::new()
                .fg(Color::AnsiValue(42))
                .bg(Color::AnsiValue(84))
                .apply("demo", false),
            "\x1b[38;5;42m\x1b[48;5;84mdemo\x1b[0m"
        );
        assert_eq!(
            AnsiStyle::new()
                .fg(Color::Reset)
                .bg(Color::Reset)
                .apply("demo", false),
            "\x1b[39m\x1b[49mdemo\x1b[0m"
        );
    }
}
