use std::borrow::Cow;

mod palette;
mod perceptual;
mod sgr;

pub use palette::ansi256_to_rgb;

/// The maximum color depth accepted by the output terminal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorDepth {
    /// Standard and bright ANSI colors, encoded without indexed color sequences.
    Ansi16,
    /// The 256-color ANSI palette.
    Ansi256,
    /// Preserve the original color sequences.
    #[default]
    TrueColor,
}

impl ColorDepth {
    /// Limit SGR colors while preserving text and non-color controls.
    /// ANSI 16 preserves hue families and separates distinct foreground/background
    /// colors that would otherwise collapse to one palette entry.
    #[must_use]
    pub fn adapt<'a>(self, text: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        let text = text.into();
        if self == Self::TrueColor
            || !text.as_bytes().contains(&b'\x1b')
            || !["38;", "48;", "58;", "38:", "48:", "58:"]
                .iter()
                .any(|pattern| text.contains(pattern))
        {
            return text;
        }
        match sgr::adapt(&text, self) {
            Cow::Borrowed(_) => text,
            Cow::Owned(converted) => Cow::Owned(converted),
        }
    }
}

#[cfg(test)]
mod tests;
