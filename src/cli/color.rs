//! User-selected color modes and resolved terminal styling.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lower")]
pub enum ColorMode {
    /// Enable styling when the output destination is a terminal.
    #[default]
    Auto,
    /// Enable styling for every output destination.
    Always,
    /// Disable generated SGR styling and OSC 8 hyperlinks.
    Never,
}

impl ColorMode {
    /// Resolve the requested mode for the caller's output destination.
    pub const fn resolve(self, output_is_terminal: bool) -> OutputStyle {
        match self {
            Self::Auto if output_is_terminal => OutputStyle::Enabled,
            Self::Always => OutputStyle::Enabled,
            Self::Auto | Self::Never => OutputStyle::Disabled,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputStyle {
    /// Generate terminal styling and hyperlinks where requested by the theme.
    Enabled,
    /// Limit styling to the 16 standard and bright ANSI colors.
    Ansi16,
    /// Limit styling to the 256-color ANSI palette.
    Ansi256,
    /// Preserve rendered text without generated terminal styling or hyperlinks.
    Disabled,
}

impl OutputStyle {
    /// Return whether terminal styling is enabled.
    pub const fn is_enabled(self) -> bool {
        !self.is_disabled()
    }

    /// Return whether terminal styling is disabled.
    pub const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub(crate) const fn color_depth(self) -> minus::ColorDepth {
        match self {
            Self::Ansi16 => minus::ColorDepth::Ansi16,
            Self::Ansi256 => minus::ColorDepth::Ansi256,
            Self::Enabled | Self::Disabled => minus::ColorDepth::TrueColor,
        }
    }

    pub(crate) fn adapt_owned(self, text: String) -> String {
        self.color_depth().adapt(text).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_every_mode_for_terminal_and_pipe() {
        assert_eq!(ColorMode::Auto.resolve(true), OutputStyle::Enabled);
        assert_eq!(ColorMode::Auto.resolve(false), OutputStyle::Disabled);
        assert_eq!(ColorMode::Always.resolve(true), OutputStyle::Enabled);
        assert_eq!(ColorMode::Always.resolve(false), OutputStyle::Enabled);
        assert_eq!(ColorMode::Never.resolve(true), OutputStyle::Disabled);
        assert_eq!(ColorMode::Never.resolve(false), OutputStyle::Disabled);
    }
}
