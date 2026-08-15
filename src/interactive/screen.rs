use super::app::App;
use super::browser::{BrowserSection, BrowserState, FilterState};
use crate::terminal::AnsiStyle;
use crate::utils::display_width;
use anyhow::{Context, Result};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::style::{Color, Print, ResetColor};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{execute, queue};
use std::io::{Stdout, Write, stdout};
use std::time::{Duration, SystemTime};
use unicode_width::UnicodeWidthChar;

const ELLIPSIS: &str = "…";
const BROWSER_ACCENT: Color = Color::Rgb {
    r: 126,
    g: 156,
    b: 216,
};
const BROWSER_SELECTED_DATE: Color = Color::Rgb {
    r: 92,
    g: 107,
    b: 135,
};
const BROWSER_FILTER_INPUT: Color = Color::Rgb {
    r: 184,
    g: 197,
    b: 223,
};
const BROWSER_LOGO_FOREGROUND: Color = Color::Rgb {
    r: 31,
    g: 35,
    b: 53,
};
const BROWSER_HELP_KEY: Color = Color::Rgb {
    r: 97,
    g: 97,
    b: 97,
};
const BROWSER_HELP_LABEL: Color = Color::Rgb {
    r: 73,
    g: 73,
    b: 73,
};
const BROWSER_HELP_SEPARATOR: Color = Color::Rgb {
    r: 60,
    g: 60,
    b: 60,
};
const BROWSER_MINI_HELP: &[(&str, &str)] = &[
    ("h/l ←/→", "page"),
    ("/", "find"),
    ("r", "refresh"),
    ("e", "edit"),
    ("q", "quit"),
    ("?", "more"),
];
const BROWSER_FILTERED_MINI_HELP: &[(&str, &str)] = &[
    ("tab", "section"),
    ("/", "edit search"),
    ("esc", "clear filter"),
    ("r", "refresh"),
    ("e", "edit"),
    ("q", "quit"),
    ("?", "more"),
];
const BROWSER_FULL_HELP_ROWS: [[Option<(&str, &str)>; 4]; 4] = [
    [
        Some(("enter", "open")),
        Some(("/", "find")),
        Some(("e", "edit")),
        Some(("r", "refresh")),
    ],
    [
        Some(("j/k ↑/↓", "choose")),
        Some(("esc", "clear")),
        Some(("!", "errors")),
        Some(("q", "quit")),
    ],
    [
        Some(("h/l ←/→", "page")),
        Some(("tab", "section")),
        Some(("?", "close help")),
        None,
    ],
    [
        Some(("g/home", "first")),
        Some(("G/end", "last")),
        None,
        None,
    ],
];

mod draw;
mod header;
mod help;
mod session;
mod style;
mod time;

pub(super) use session::TerminalSession;

use draw::*;
use header::*;
use help::*;
#[cfg(test)]
use session::{write_pager_pause, write_pager_resume};
use style::*;
use time::*;
#[cfg(test)]
mod tests;
