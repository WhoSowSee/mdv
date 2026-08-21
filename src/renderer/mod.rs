mod event;
mod front_matter;
mod line_numbers;
mod syntax_set;
mod syntax_theme;
pub(super) mod terminal;

pub use terminal::TerminalRenderer;

#[cfg(test)]
mod tests;
