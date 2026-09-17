use super::RESET_STYLE;
use crossterm::style::{Attribute, Color};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptColor {
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
    Rgb { r: u8, g: u8, b: u8 },
    AnsiValue(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptAttribute {
    Bold,
    Dim,
    Italic,
    Underlined,
    Reverse,
    Hidden,
    CrossedOut,
}

impl PromptAttribute {
    const ALL: [Self; 7] = [
        Self::Bold,
        Self::Dim,
        Self::Italic,
        Self::Underlined,
        Self::Reverse,
        Self::Hidden,
        Self::CrossedOut,
    ];

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PromptStyle {
    pub(super) foreground: Option<PromptColor>,
    pub(super) background: Option<PromptColor>,
    pub(super) attributes: u8,
}

impl PromptStyle {
    #[must_use]
    pub const fn foreground(mut self, color: PromptColor) -> Self {
        self.foreground = Some(color);
        self
    }

    #[must_use]
    pub const fn background(mut self, color: PromptColor) -> Self {
        self.background = Some(color);
        self
    }

    #[must_use]
    pub const fn attribute(mut self, attribute: PromptAttribute) -> Self {
        self.attributes |= attribute.bit();
        self
    }

    pub(super) fn write_styled(self, output: &mut String, text: &str) {
        let mut styled = false;
        if let Some(color) = self.foreground {
            write_prompt_color(output, color, false);
            styled = true;
        }
        if let Some(color) = self.background {
            write_prompt_color(output, color, true);
            styled = true;
        }
        for attribute in PromptAttribute::ALL {
            if self.attributes & attribute.bit() != 0 {
                let code = match attribute {
                    PromptAttribute::Bold => 1,
                    PromptAttribute::Dim => 2,
                    PromptAttribute::Italic => 3,
                    PromptAttribute::Underlined => 4,
                    PromptAttribute::Reverse => 7,
                    PromptAttribute::Hidden => 8,
                    PromptAttribute::CrossedOut => 9,
                };
                let _ = write!(output, "\x1b[{code}m");
                styled = true;
            }
        }
        output.push_str(text);
        if styled {
            output.push_str(RESET_STYLE);
        }
    }
}

fn write_prompt_color(output: &mut String, color: PromptColor, background: bool) {
    let offset = if background { 10 } else { 0 };
    match color {
        PromptColor::Rgb { r, g, b } => {
            let base = if background { 48 } else { 38 };
            let _ = write!(output, "\x1b[{base};2;{r};{g};{b}m");
        }
        PromptColor::AnsiValue(value) => {
            let base = if background { 48 } else { 38 };
            let _ = write!(output, "\x1b[{base};5;{value}m");
        }
        named => {
            let foreground = match named {
                PromptColor::Black => 30,
                PromptColor::DarkRed => 31,
                PromptColor::DarkGreen => 32,
                PromptColor::DarkYellow => 33,
                PromptColor::DarkBlue => 34,
                PromptColor::DarkMagenta => 35,
                PromptColor::DarkCyan => 36,
                PromptColor::Grey => 37,
                PromptColor::DarkGrey => 90,
                PromptColor::Red => 91,
                PromptColor::Green => 92,
                PromptColor::Yellow => 93,
                PromptColor::Blue => 94,
                PromptColor::Magenta => 95,
                PromptColor::Cyan => 96,
                PromptColor::White => 97,
                PromptColor::Rgb { .. } | PromptColor::AnsiValue(_) => unreachable!(),
            };
            let _ = write!(output, "\x1b[{}m", foreground + offset);
        }
    }
}

impl From<PromptColor> for Color {
    fn from(color: PromptColor) -> Self {
        match color {
            PromptColor::Black => Self::Black,
            PromptColor::DarkGrey => Self::DarkGrey,
            PromptColor::Red => Self::Red,
            PromptColor::DarkRed => Self::DarkRed,
            PromptColor::Green => Self::Green,
            PromptColor::DarkGreen => Self::DarkGreen,
            PromptColor::Yellow => Self::Yellow,
            PromptColor::DarkYellow => Self::DarkYellow,
            PromptColor::Blue => Self::Blue,
            PromptColor::DarkBlue => Self::DarkBlue,
            PromptColor::Magenta => Self::Magenta,
            PromptColor::DarkMagenta => Self::DarkMagenta,
            PromptColor::Cyan => Self::Cyan,
            PromptColor::DarkCyan => Self::DarkCyan,
            PromptColor::White => Self::White,
            PromptColor::Grey => Self::Grey,
            PromptColor::Rgb { r, g, b } => Self::Rgb { r, g, b },
            PromptColor::AnsiValue(value) => Self::AnsiValue(value),
        }
    }
}

impl From<PromptAttribute> for Attribute {
    fn from(attribute: PromptAttribute) -> Self {
        match attribute {
            PromptAttribute::Bold => Self::Bold,
            PromptAttribute::Dim => Self::Dim,
            PromptAttribute::Italic => Self::Italic,
            PromptAttribute::Underlined => Self::Underlined,
            PromptAttribute::Reverse => Self::Reverse,
            PromptAttribute::Hidden => Self::Hidden,
            PromptAttribute::CrossedOut => Self::CrossedOut,
        }
    }
}
