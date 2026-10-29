use super::consume_escape_sequence;
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
enum Attribute {
    IntensityReset,
    Bold,
    Faint,
    FontStyle,
    Underline,
    Blink,
    Inverse,
    Conceal,
    Strike,
    Font,
    Proportional,
    Frame,
    Overline,
    Ideogram,
    Script,
    Foreground,
    Background,
    UnderlineColor,
}

const ATTRIBUTE_COUNT: usize = Attribute::UnderlineColor as usize + 1;

#[derive(Clone, Copy, Debug)]
enum Parameter {
    Code(u16),
    Indexed(u16, u8),
    Rgb(u16, [u8; 3]),
    Underline(u8),
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SgrState {
    attributes: [Option<Parameter>; ATTRIBUTE_COUNT],
}

impl SgrState {
    pub(crate) fn update(&mut self, text: &str) {
        let mut chars = text.chars().peekable();
        while let Some(character) = chars.next() {
            if character == '\x1b' {
                self.apply_sequence(&consume_escape_sequence(&mut chars));
            }
        }
    }

    pub(super) fn apply_sequence(&mut self, sequence: &str) {
        let Some(parameters) = sequence
            .strip_prefix("\x1b[")
            .and_then(|sequence| sequence.strip_suffix('m'))
        else {
            return;
        };
        let mut next = *self;
        let mut parts = parameters.split(';');
        while let Some(part) = parts.next() {
            let Some(parameter) = parse_parameter(part, &mut parts) else {
                return;
            };
            if matches!(parameter, Parameter::Code(0)) {
                next = Self::default();
            } else {
                next.apply(parameter);
            }
        }
        *self = next;
    }

    fn apply(&mut self, parameter: Parameter) {
        use Attribute::*;
        let code = match parameter {
            Parameter::Code(code) | Parameter::Indexed(code, _) | Parameter::Rgb(code, _) => code,
            Parameter::Underline(_) => 4,
        };
        let attribute = match code {
            1 => Bold,
            2 => Faint,
            22 => {
                self.attributes[Bold as usize] = None;
                self.attributes[Faint as usize] = None;
                IntensityReset
            }
            3 | 20 | 23 => FontStyle,
            4 | 21 | 24 => Underline,
            5 | 6 | 25 => Blink,
            7 | 27 => Inverse,
            8 | 28 => Conceal,
            9 | 29 => Strike,
            10..=19 => Font,
            26 | 50 => Proportional,
            30..=39 | 90..=97 => Foreground,
            40..=49 | 100..=107 => Background,
            51 | 52 | 54 => Frame,
            53 | 55 => Overline,
            58 | 59 => UnderlineColor,
            60..=65 => Ideogram,
            73..=75 => Script,
            _ => return,
        };
        self.attributes[attribute as usize] = Some(parameter);
    }

    pub(crate) fn prefix(&self) -> String {
        let mut output = String::new();
        for parameter in self.attributes.iter().flatten() {
            match parameter {
                Parameter::Code(code) => {
                    let _ = write!(output, "\x1b[{code}m");
                }
                Parameter::Indexed(target, index) => {
                    let _ = write!(output, "\x1b[{target};5;{index}m");
                }
                Parameter::Rgb(target, [red, green, blue]) => {
                    let _ = write!(output, "\x1b[{target};2;{red};{green};{blue}m");
                }
                Parameter::Underline(style) => {
                    let _ = write!(output, "\x1b[4:{style}m");
                }
            }
        }
        output
    }
}

fn parse_parameter<'a>(
    part: &'a str,
    remaining: &mut impl Iterator<Item = &'a str>,
) -> Option<Parameter> {
    if part.contains(':') {
        return parse_colon_parameter(part);
    }
    let code = if part.is_empty() {
        0
    } else {
        part.parse().ok()?
    };
    if !matches!(code, 38 | 48 | 58) {
        return Some(Parameter::Code(code));
    }
    match remaining.next()? {
        "5" => Some(Parameter::Indexed(code, remaining.next()?.parse().ok()?)),
        "2" => Some(Parameter::Rgb(
            code,
            [
                remaining.next()?.parse().ok()?,
                remaining.next()?.parse().ok()?,
                remaining.next()?.parse().ok()?,
            ],
        )),
        _ => None,
    }
}

fn parse_colon_parameter(part: &str) -> Option<Parameter> {
    let parts = part.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["4", style] => Some(Parameter::Underline(style.parse().ok()?)),
        [target @ ("38" | "48" | "58"), "5", index] => Some(Parameter::Indexed(
            target.parse().ok()?,
            index.parse().ok()?,
        )),
        [target @ ("38" | "48" | "58"), "2", red, green, blue]
        | [
            target @ ("38" | "48" | "58"),
            "2",
            "" | "0",
            red,
            green,
            blue,
        ] => Some(Parameter::Rgb(
            target.parse().ok()?,
            [red.parse().ok()?, green.parse().ok()?, blue.parse().ok()?],
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
