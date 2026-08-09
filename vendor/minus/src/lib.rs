#![cfg_attr(docsrs, feature(doc_cfg))]
// A featureless build exposes no usable API, so its dead-code warnings are intentional.
#![cfg_attr(
    not(any(feature = "dynamic_output", feature = "static_output")),
    allow(unused_imports),
    allow(dead_code)
)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Terminal paging for static and streaming output.
//!
//! [`Pager`] sends content and configuration to the pager. Feature-gated runner functions start
//! either a streaming or preloaded session, while [`input`] exposes input customization.
#[cfg(feature = "dynamic_output")]
mod dynamic_pager;
pub mod error;
pub mod hooks;
pub mod input;
#[path = "core/mod.rs"]
mod minus_core;
mod pager;
mod prompt;
pub mod screen;
#[cfg(feature = "search")]
#[cfg_attr(docsrs, doc(cfg(feature = "search")))]
pub mod search;
mod selection;
pub mod state;
#[cfg(feature = "static_output")]
mod static_pager;

#[cfg(feature = "dynamic_output")]
pub use dynamic_pager::dynamic_paging;
#[cfg(feature = "static_output")]
pub use static_pager::page_all;

pub use minus_core::RunMode;
#[cfg(feature = "search")]
pub use search::SearchMode;

pub use error::MinusError;
pub use pager::Pager;
pub use prompt::{
    PromptAttribute, PromptColor, PromptContext, PromptError, PromptLine, PromptRenderer,
    PromptSpan, PromptStyle,
};
pub use state::PagerState;

/// Exit callbacks invoked in registration order.
pub type ExitCallbacks = Vec<Box<dyn FnMut() + Send + Sync + 'static>>;

type Result<T = (), E = MinusError> = std::result::Result<T, E>;

/// Determines whether exiting the pager also terminates the process.
#[derive(PartialEq, Clone, Debug, Eq)]
pub enum ExitStrategy {
    /// Terminates the process when the pager exits. This is the default.
    ProcessQuit,
    /// Returns control to the caller when the pager exits.
    PagerQuit,
}

/// Controls line-number visibility and whether the user may toggle it.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LineNumbers {
    /// Enables line numbers and prevents user toggling.
    AlwaysOn,
    /// Enables line numbers while allowing user toggling.
    Enabled,
    /// Disables line numbers while allowing user toggling.
    Disabled,
    /// Disables line numbers and prevents user toggling.
    AlwaysOff,
}

impl LineNumbers {
    const EXTRA_PADDING: usize = 5;

    #[allow(dead_code)]
    const fn is_invertible(self) -> bool {
        matches!(self, Self::Enabled | Self::Disabled)
    }

    const fn is_on(self) -> bool {
        matches!(self, Self::Enabled | Self::AlwaysOn)
    }
}

impl std::ops::Not for LineNumbers {
    type Output = Self;

    fn not(self) -> Self::Output {
        use LineNumbers::{Disabled, Enabled};

        match self {
            Enabled => Disabled,
            Disabled => Enabled,
            ln => ln,
        }
    }
}

#[cfg(test)]
mod tests;
