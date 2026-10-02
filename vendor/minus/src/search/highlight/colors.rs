use crate::{ColorDepth, HighlightColors, PromptColor};

const DEFAULT_FOREGROUND: Rgb = Rgb::new(192, 192, 192);
const DEFAULT_BACKGROUND: Rgb = Rgb::new(18, 20, 24);
const CURRENT_MATCH_BLEND: u16 = 55;
const OTHER_MATCH_BLEND: u16 = 22;
pub(super) const LINE_NAVIGATION_BACKGROUND: Rgb = Rgb::new(56, 58, 61);

#[derive(Clone, Copy)]
pub(super) struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn blend(self, tint: Self, percent: u16) -> Self {
        Self::new(
            blend_channel(self.r, tint.r, percent),
            blend_channel(self.g, tint.g, percent),
            blend_channel(self.b, tint.b, percent),
        )
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct SgrState {
    foreground: Option<Rgb>,
    background: Option<Rgb>,
}

impl SgrState {
    fn apply(&mut self, parameters: &[Option<u16>]) {
        let mut index = 0;
        while index < parameters.len() {
            let Some(code) = parameters[index] else {
                index += 1;
                continue;
            };
            match code {
                0 => *self = Self::default(),
                30..=37 => self.foreground = Some(ansi16_color(code - 30)),
                40..=47 => self.background = Some(ansi16_color(code - 40)),
                90..=97 => self.foreground = Some(ansi16_color(code - 90 + 8)),
                100..=107 => self.background = Some(ansi16_color(code - 100 + 8)),
                38 | 48 => {
                    if let Some((color, consumed)) = extended_color(parameters, index) {
                        if code == 38 {
                            self.foreground = Some(color);
                        } else {
                            self.background = Some(color);
                        }
                        index += consumed;
                    }
                }
                39 => self.foreground = None,
                49 => self.background = None,
                _ => {}
            }
            index += 1;
        }
    }
}

fn blend_channel(base: u8, tint: u8, percent: u16) -> u8 {
    let blended = (u16::from(base) * (100 - percent) + u16::from(tint) * percent + 50) / 100;
    u8::try_from(blended).unwrap_or(u8::MAX)
}

fn sgr_parameters(escape: &str) -> Option<Vec<Option<u16>>> {
    let parameters = escape
        .strip_prefix("\x1b[")
        .or_else(|| escape.strip_prefix("\u{9b}["))?
        .strip_suffix('m')?;
    if parameters.is_empty() {
        return Some(vec![Some(0)]);
    }
    Some(
        parameters
            .split(';')
            .map(|value| {
                if value.is_empty() {
                    Some(0)
                } else {
                    value.parse::<u16>().ok()
                }
            })
            .collect(),
    )
}

fn ansi16_color(index: u16) -> Rgb {
    const COLORS: [Rgb; 16] = [
        Rgb::new(0, 0, 0),
        Rgb::new(205, 0, 0),
        Rgb::new(0, 205, 0),
        Rgb::new(205, 205, 0),
        Rgb::new(0, 0, 238),
        Rgb::new(205, 0, 205),
        Rgb::new(0, 205, 205),
        Rgb::new(229, 229, 229),
        Rgb::new(127, 127, 127),
        Rgb::new(255, 0, 0),
        Rgb::new(0, 255, 0),
        Rgb::new(255, 255, 0),
        Rgb::new(92, 92, 255),
        Rgb::new(255, 0, 255),
        Rgb::new(0, 255, 255),
        Rgb::new(255, 255, 255),
    ];
    COLORS
        .get(usize::from(index))
        .copied()
        .unwrap_or(DEFAULT_FOREGROUND)
}

fn ansi256_color(index: u8) -> Rgb {
    if index < 16 {
        return ansi16_color(u16::from(index));
    }
    let (red, green, blue) = crate::ansi256_to_rgb(index);
    Rgb::new(red, green, blue)
}

fn extended_color(parameters: &[Option<u16>], index: usize) -> Option<(Rgb, usize)> {
    match parameters.get(index + 1).copied().flatten()? {
        5 => {
            let color = u8::try_from(parameters.get(index + 2).copied().flatten()?).ok()?;
            Some((ansi256_color(color), 2))
        }
        2 => {
            let red = u8::try_from(parameters.get(index + 2).copied().flatten()?).ok()?;
            let green = u8::try_from(parameters.get(index + 3).copied().flatten()?).ok()?;
            let blue = u8::try_from(parameters.get(index + 4).copied().flatten()?).ok()?;
            Some((Rgb::new(red, green, blue), 4))
        }
        _ => None,
    }
}

pub(super) fn track_sgr(history: &mut String, state: &mut SgrState, escape: &str) {
    let Some(parameters) = sgr_parameters(escape) else {
        return;
    };
    if parameters.first().copied().flatten() == Some(0) {
        history.clear();
    }
    history.push_str(escape);
    state.apply(&parameters);
}

fn highlight_color(state: SgrState, current: bool, fixed: Option<Rgb>) -> Rgb {
    fixed.unwrap_or_else(|| {
        let foreground = state.foreground.unwrap_or(DEFAULT_FOREGROUND);
        let background = state.background.unwrap_or(DEFAULT_BACKGROUND);
        let blend = if current {
            CURRENT_MATCH_BLEND
        } else {
            OTHER_MATCH_BLEND
        };
        background.blend(foreground, blend)
    })
}

pub(super) fn highlight_background(
    state: SgrState,
    current: bool,
    fixed: Option<Rgb>,
    depth: ColorDepth,
    colors: HighlightColors,
) -> String {
    let (foreground, background) = if depth == ColorDepth::Ansi16 {
        (
            Some(PromptColor::Black),
            Some(if fixed.is_some() {
                PromptColor::Grey
            } else if current {
                PromptColor::Yellow
            } else {
                PromptColor::Cyan
            }),
        )
    } else {
        let color = highlight_color(state, current, fixed);
        (
            None,
            Some(PromptColor::Rgb {
                r: color.r,
                g: color.g,
                b: color.b,
            }),
        )
    };
    colors.sequence(foreground, background)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_navigation_background_is_fixed() {
        let default = highlight_color(SgrState::default(), false, None);
        assert_eq!((default.r, default.g, default.b), (56, 58, 61));

        let styled = SgrState {
            foreground: Some(Rgb::new(206, 145, 120)),
            background: Some(Rgb::new(10, 20, 30)),
        };
        let fixed = highlight_color(styled, false, Some(LINE_NAVIGATION_BACKGROUND));
        assert_eq!((fixed.r, fixed.g, fixed.b), (56, 58, 61));
    }
}
