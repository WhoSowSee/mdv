use super::{
    ColorDepth,
    palette::{Color, contrasting_foreground},
};
use crate::selection::ansi_sequence_end;
use std::borrow::Cow;
use std::fmt::Write;

#[derive(Clone, Copy, Default)]
struct Colors {
    foreground: Option<Color>,
    background: Option<Color>,
    corrected_foreground: Option<u8>,
}

pub(super) fn adapt(text: &str, depth: ColorDepth) -> Cow<'_, str> {
    let mut output = String::with_capacity(text.len());
    let mut colors = Colors::default();
    let mut position = 0;
    while let Some(offset) = text[position..].find('\x1b') {
        let start = position + offset;
        output.push_str(&text[position..start]);
        let (end, sgr) = ansi_sequence_end(text.as_bytes(), start).unwrap();
        let sequence = &text[start..end];
        if sgr {
            let mut next = colors;
            if let Some(converted) = next.convert(&sequence[2..sequence.len() - 1], depth) {
                output.push_str(&converted);
                colors = next;
            } else {
                output.push_str(sequence);
            }
        } else {
            output.push_str(sequence);
        }
        position = end;
    }
    output.push_str(&text[position..]);
    if output == text {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(output)
    }
}

impl Colors {
    fn convert(&mut self, parameters: &str, depth: ColorDepth) -> Option<String> {
        let parts = parameters.split(';').collect::<Vec<_>>();
        let mut converted = Vec::with_capacity(parts.len());
        let mut index = 0;
        while index < parts.len() {
            let part = parts[index];
            if part.contains(':') {
                let components = part.split(':').collect::<Vec<_>>();
                let target = number(components[0])?;
                if matches!(target, 38 | 48 | 58) {
                    let color = colon_color(&components)?;
                    self.color(target, color, depth, &mut converted);
                } else {
                    converted.push(part.to_owned());
                }
                index += 1;
                continue;
            }
            let value = if part.is_empty() { 0 } else { number(part)? };
            if matches!(value, 38 | 48 | 58) {
                let (color, consumed) = match *parts.get(index + 1)? {
                    "5" => (Color::Indexed(number(parts.get(index + 2)?)?), 3),
                    "2" => (
                        Color::Rgb([
                            number(parts.get(index + 2)?)?,
                            number(parts.get(index + 3)?)?,
                            number(parts.get(index + 4)?)?,
                        ]),
                        5,
                    ),
                    _ => return None,
                };
                self.color(value, color, depth, &mut converted);
                index += consumed;
                continue;
            }
            match value {
                0 => *self = Self::default(),
                30..=37 | 90..=97 => {
                    self.foreground = Some(Color::Indexed(if value < 90 {
                        value - 30
                    } else {
                        value - 82
                    }));
                    self.corrected_foreground = None;
                }
                40..=47 | 100..=107 => {
                    self.background = Some(Color::Indexed(if value < 100 {
                        value - 40
                    } else {
                        value - 92
                    }));
                }
                39 => {
                    self.foreground = None;
                    self.corrected_foreground = None;
                }
                49 => self.background = None,
                _ => {}
            }
            converted.push(part.to_owned());
            index += 1;
        }
        let mut output = String::new();
        if !converted.is_empty() {
            let _ = write!(output, "\x1b[{}m", converted.join(";"));
        }
        if depth == ColorDepth::Ansi16 {
            self.ensure_contrast(&mut output);
        }
        Some(output)
    }

    fn color(&mut self, target: u8, color: Color, depth: ColorDepth, output: &mut Vec<String>) {
        // Basic 16-color SGR has no separate underline color.
        if target == 58 && depth == ColorDepth::Ansi16 {
            return;
        }
        match target {
            38 => {
                self.foreground = Some(color);
                self.corrected_foreground = None;
            }
            48 => self.background = Some(color),
            _ => {}
        }
        let index = color.index(depth);
        if depth == ColorDepth::Ansi16 {
            output.push(ansi_code(index, target == 48).to_string());
        } else {
            output.push(format!("{target};5;{index}"));
        }
    }

    fn ensure_contrast(&mut self, output: &mut String) {
        let correction = self.foreground.zip(self.background).and_then(|(fg, bg)| {
            let foreground = fg.index(ColorDepth::Ansi16);
            let background = bg.index(ColorDepth::Ansi16);
            (foreground == background && fg.rgb() != bg.rgb())
                .then(|| contrasting_foreground(background))
        });
        if correction != self.corrected_foreground {
            if let Some(index) =
                correction.or_else(|| self.foreground.map(|c| c.index(ColorDepth::Ansi16)))
            {
                let _ = write!(output, "\x1b[{}m", ansi_code(index, false));
            }
            self.corrected_foreground = correction;
        }
    }
}

fn number(value: &str) -> Option<u8> {
    value.parse().ok()
}

fn colon_color(parts: &[&str]) -> Option<Color> {
    match parts {
        [_, "5", index] => Some(Color::Indexed(number(index)?)),
        [_, "2", r, g, b] | [_, "2", "" | "0", r, g, b] => {
            Some(Color::Rgb([number(r)?, number(g)?, number(b)?]))
        }
        _ => None,
    }
}

const fn ansi_code(index: u8, background: bool) -> u8 {
    (if index < 8 {
        30 + index
    } else {
        90 + index - 8
    }) + if background { 10 } else { 0 }
}
