use super::*;

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
