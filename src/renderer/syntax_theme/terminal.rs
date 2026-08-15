use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FgSpec {
    Reset,
    Named(u8),
    Palette(u8),
    Truecolor(u8, u8, u8),
}

impl FgSpec {
    fn from_color(color: &Color) -> Self {
        match color {
            Color::Reset => FgSpec::Reset,
            Color::AnsiValue(index) => FgSpec::Palette(*index),
            Color::Rgb { r, g, b } => FgSpec::Truecolor(*r, *g, *b),
            other => FgSpec::Named(named_fg_code(other)),
        }
    }

    fn write(&self, out: &mut String) {
        match self {
            FgSpec::Reset => out.push_str("\x1b[39m"),
            FgSpec::Named(code) => {
                let _ = write!(out, "\x1b[{}m", code);
            }
            FgSpec::Palette(index) => {
                let _ = write!(out, "\x1b[38;5;{}m", index);
            }
            FgSpec::Truecolor(r, g, b) => {
                let _ = write!(out, "\x1b[38;2;{};{};{}m", r, g, b);
            }
        }
    }
}

fn named_fg_code(color: &Color) -> u8 {
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
        // Unreachable: AnsiValue/Rgb/Reset are handled by FgSpec::from_color.
        Color::AnsiValue(_) | Color::Rgb { .. } | Color::Reset => 39,
    }
}

fn write_font_style_diff(out: &mut String, prev: FontStyle, cur: FontStyle) {
    const FLAGS: [(FontStyle, &str, &str); 3] = [
        (FontStyle::BOLD, "\x1b[1m", "\x1b[22m"),
        (FontStyle::ITALIC, "\x1b[3m", "\x1b[23m"),
        (FontStyle::UNDERLINE, "\x1b[4m", "\x1b[24m"),
    ];
    for (flag, on, off) in FLAGS {
        match (prev.contains(flag), cur.contains(flag)) {
            (false, true) => out.push_str(on),
            (true, false) => out.push_str(off),
            _ => {}
        }
    }
}

/// Render highlighted fragments, restoring palette codes from `palette` instead of
/// always emitting truecolor. Transparent fragments (`a == 0`) become `\x1b[39m`.
pub(crate) fn as_terminal_escaped(
    ranges: &[(Style, &str)],
    palette: &HashMap<(u8, u8, u8), Color>,
) -> String {
    let mut out = String::new();
    let mut prev_fg = None;
    let mut prev_font = FontStyle::default();
    for (style, text) in ranges {
        let spec = if style.foreground.a == 0 {
            FgSpec::Reset
        } else {
            let key = (style.foreground.r, style.foreground.g, style.foreground.b);
            match palette.get(&key) {
                Some(color) => FgSpec::from_color(color),
                None => {
                    FgSpec::Truecolor(style.foreground.r, style.foreground.g, style.foreground.b)
                }
            }
        };
        if Some(spec) != prev_fg {
            spec.write(&mut out);
            prev_fg = Some(spec);
        }
        if style.font_style != prev_font {
            write_font_style_diff(&mut out, prev_font, style.font_style);
            prev_font = style.font_style;
        }
        out.push_str(text);
    }
    out
}
