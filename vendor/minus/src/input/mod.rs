//! Keyboard and mouse input classification.
//!
//! [`HashedEventRegister`] maps binding descriptions to callbacks. Key descriptions use `c-`,
//! `m-`, and `s-` for Ctrl, Alt, and Shift; mouse descriptions use values such as `left:down`
//! and `scroll:up`. [`InputClassifier`] supports fully custom classification. Terminals may
//! normalize modifier combinations before `minus` receives them.

pub(crate) mod definitions;
pub(crate) mod hashed_event_register;

pub use crossterm::event as crossterm_event;

#[cfg(feature = "search")]
use crate::search::SearchMode;
use crate::{LineNumbers, PagerState};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

#[cfg_attr(
    docsrs,
    deprecated = "See [#163](https://github.com/AMythicDev/minus/pull/163)."
)]
pub use hashed_event_register::HashedEventRegister;

/// Events handled by the `minus` pager.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
#[non_exhaustive]
pub enum InputEvent {
    /// `Ctrl+C` or `Q`, exits the application.
    Exit,
    /// The terminal was resized. Contains the new number of rows.
    UpdateTermArea(usize, usize),
    /// Sent by movement keys like `Up` `Down`, `PageUp`, `PageDown`, 'g', `G` etc.
    /// Contains the new value for the upper mark.
    UpdateUpperMark(usize),
    /// `Ctrl+L`, inverts the line number display. Contains the new value.
    UpdateLineNumber(LineNumbers),
    /// A numeric prefix character.
    Number(char),
    /// Restore the original prompt
    RestorePrompt,
    /// Whether to allow Horizontal scrolling
    HorizontalScroll(bool),
    /// Sets the horizontal scroll offset.
    UpdateLeftMark(usize),
    /// Start a mouse selection at the given screen coordinates.
    StartSelection { x: u16, y: u16 },
    /// Update the current mouse selection to the given screen coordinates.
    UpdateSelection { x: u16, y: u16 },
    /// Clear the current mouse selection.
    ClearSelection,
    /// Copy the current selection.
    #[cfg(feature = "clipboard")]
    CopySelection,
    #[cfg(feature = "clipboard")]
    FinalizeSelection,
    /// Leaves pager state unchanged after the classifier callback runs.
    Ignore,
    /// `/`, Searching for certain pattern of text
    #[cfg(feature = "search")]
    Search(SearchMode),
    /// Moves to the next match.
    #[cfg(feature = "search")]
    #[deprecated = "Use [InputEvent::MoveToNextMatch(1)](InputEvent::MoveToNextMatch) for the same effect."]
    NextMatch,
    /// Moves to the previous match.
    #[deprecated = "Use [InputEvent::MoveToPrevMatch(1)](InputEvent::MoveToPrevMatch) for the same effect."]
    #[cfg(feature = "search")]
    PrevMatch,
    /// Move to the next nth match in the given direction
    #[cfg(feature = "search")]
    MoveToNextMatch(usize),
    /// Move to the previous nth match in the given direction
    #[cfg(feature = "search")]
    MoveToPrevMatch(usize),
    /// Enables or disables viewport anchoring to the end of output.
    FollowOutput(bool),
}

/// Maps terminal events to pager actions.
#[allow(clippy::module_name_repetitions)]
#[cfg_attr(
    docsrs,
    deprecated = "See [#163](https://github.com/AMythicDev/minus/pull/163)."
)]
pub trait InputClassifier {
    fn classify_input(&self, ev: Event, ps: &PagerState) -> Option<InputEvent>;
}

/// Insert the default set of actions into the [`HashedEventRegister`]
#[allow(clippy::module_name_repetitions)]
#[cfg_attr(
    docsrs,
    deprecated = "See [#163](https://github.com/AMythicDev/minus/pull/163)."
)]
#[allow(clippy::too_many_lines)]
pub fn generate_default_bindings<S>(map: &mut HashedEventRegister<S>)
where
    S: std::hash::BuildHasher,
{
    map.add_key_events(&["q", "c-c", "esc"], |_, _| InputEvent::Exit);

    map.add_key_events(&["up", "k"], |_, ps| {
        let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(position))
    });
    map.add_key_events(&["down", "j"], |_, ps| {
        let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(position))
    });
    map.add_key_events(&["c-f"], |_, ps| {
        InputEvent::FollowOutput(!ps.follow_output)
    });
    map.add_key_events(&["enter"], |_, ps| {
        if ps.message.is_some() {
            InputEvent::RestorePrompt
        } else {
            let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
            InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(position))
        }
    });
    map.add_key_events(&["u", "c-u"], |_, ps| {
        let half_screen = ps.content_rows() / 2;
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(half_screen))
    });
    map.add_key_events(&["d", "c-d"], |_, ps| {
        let half_screen = ps.content_rows() / 2;
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(half_screen))
    });
    map.add_key_events(&["g", "home"], |_, _| InputEvent::UpdateUpperMark(0));

    map.add_key_events(&["s-g", "G"], |_, ps| {
        // Input line numbers are one-based; upper_mark is zero-based.
        let mut position = ps
            .prefix_num
            .parse::<usize>()
            .unwrap_or(usize::MAX)
            .saturating_sub(1);
        if position == 0 {
            position = usize::MAX;
        }
        // Missing line numbers resolve to the bottom after clamping.
        let row_to_go = *ps
            .lines_to_row_map
            .get(position)
            .unwrap_or(&(usize::MAX - 1));
        InputEvent::UpdateUpperMark(row_to_go)
    });
    map.add_key_events(&["pageup", "b"], |_, ps| {
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(ps.content_rows()))
    });
    map.add_key_events(&["pagedown", "f"], |_, ps| {
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(ps.content_rows()))
    });
    map.add_key_events(&["c-l"], |_, ps| {
        InputEvent::UpdateLineNumber(!ps.line_numbers)
    });
    map.add_key_events(&["end"], |_, _| InputEvent::UpdateUpperMark(usize::MAX - 1));
    #[cfg(feature = "search")]
    {
        map.add_key_events(&["/"], |_, _| InputEvent::Search(SearchMode::Forward));
        map.add_key_events(&["?"], |_, _| InputEvent::Search(SearchMode::Reverse));
        map.add_key_events(&["n"], |_, ps| {
            let position = ps.prefix_num.parse::<usize>().unwrap_or(1);

            if ps.search_state.search_mode == SearchMode::Forward {
                InputEvent::MoveToNextMatch(position)
            } else if ps.search_state.search_mode == SearchMode::Reverse {
                InputEvent::MoveToPrevMatch(position)
            } else {
                InputEvent::Ignore
            }
        });
        map.add_key_events(&["p", "s-n"], |_, ps| {
            let position = ps.prefix_num.parse::<usize>().unwrap_or(1);

            if ps.search_state.search_mode == SearchMode::Forward {
                InputEvent::MoveToPrevMatch(position)
            } else if ps.search_state.search_mode == SearchMode::Reverse {
                InputEvent::MoveToNextMatch(position)
            } else {
                InputEvent::Ignore
            }
        });
    }

    map.add_mouse_events(&["scroll:up"], |_, ps| {
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(5))
    });
    map.add_mouse_events(&["scroll:down"], |_, ps| {
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(5))
    });
    map.add_mouse_events(&["left:down"], |ev, _| {
        let Event::Mouse(MouseEvent { column, row, .. }) = ev else {
            unreachable!();
        };
        InputEvent::StartSelection { x: column, y: row }
    });
    map.add_mouse_events(&["left:drag"], |ev, _| {
        let Event::Mouse(MouseEvent { column, row, .. }) = ev else {
            unreachable!();
        };
        InputEvent::UpdateSelection { x: column, y: row }
    });

    #[cfg(feature = "clipboard")]
    {
        map.add_mouse_events(&["left:up"], |_, _| InputEvent::FinalizeSelection);
        map.add_key_events(&["y"], |_, _| InputEvent::CopySelection);
    }

    map.add_key_events(&["c-s-h", "c-h"], |_, ps| {
        InputEvent::HorizontalScroll(!ps.screen.line_wrapping)
    });
    map.add_key_events(&["h", "left"], |_, ps| {
        let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
        InputEvent::UpdateLeftMark(ps.left_mark.saturating_sub(position))
    });
    map.add_key_events(&["l", "right"], |_, ps| {
        let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
        InputEvent::UpdateLeftMark(ps.left_mark.saturating_add(position))
    });
    map.add_resize_event(|ev, _| {
        let Event::Resize(cols, rows) = ev else {
            unreachable!();
        };
        InputEvent::UpdateTermArea(cols as usize, rows as usize)
    });

    map.insert_wild_event_matcher(|ev, _| {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }) = ev
        {
            if c.is_ascii_digit() {
                InputEvent::Number(c)
            } else {
                InputEvent::Ignore
            }
        } else {
            InputEvent::Ignore
        }
    });
}

/// The default set of input definitions
///
/// **This is kept only for legacy purposes and may not be well updated with all the latest changes**
#[cfg_attr(
    docsrs,
    deprecated = "See [#163](https://github.com/AMythicDev/minus/pull/163)."
)]
pub struct DefaultInputClassifier;

impl InputClassifier for DefaultInputClassifier {
    #[allow(clippy::too_many_lines)]
    fn classify_input(&self, ev: Event, ps: &PagerState) -> Option<InputEvent> {
        #[allow(clippy::unnested_or_patterns)]
        match ev {
            Event::Key(KeyEvent {
                code,
                modifiers: KeyModifiers::NONE,
                ..
            }) if code == KeyCode::Up || code == KeyCode::Char('k') => {
                let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
                Some(InputEvent::UpdateUpperMark(
                    ps.upper_mark.saturating_sub(position),
                ))
            }

            Event::Key(KeyEvent {
                code,
                modifiers: KeyModifiers::NONE,
                ..
            }) if code == KeyCode::Down || code == KeyCode::Char('j') => {
                let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
                Some(InputEvent::UpdateUpperMark(
                    ps.upper_mark.saturating_add(position),
                ))
            }

            Event::Key(KeyEvent {
                code,
                modifiers: KeyModifiers::CONTROL,
                ..
            }) if code == KeyCode::Char('f') => Some(InputEvent::FollowOutput(!ps.follow_output)),

            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            }) if c.is_ascii_digit() => Some(InputEvent::Number(c)),

            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if ps.message.is_some() {
                    Some(InputEvent::RestorePrompt)
                } else {
                    let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
                    Some(InputEvent::UpdateUpperMark(
                        ps.upper_mark.saturating_add(position),
                    ))
                }
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers,
                ..
            }) if modifiers == KeyModifiers::CONTROL || modifiers == KeyModifiers::NONE => {
                let half_screen = ps.content_rows() / 2;
                Some(InputEvent::UpdateUpperMark(
                    ps.upper_mark.saturating_sub(half_screen),
                ))
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers,
                ..
            }) if modifiers == KeyModifiers::CONTROL || modifiers == KeyModifiers::NONE => {
                let half_screen = ps.content_rows() / 2;
                Some(InputEvent::UpdateUpperMark(
                    ps.upper_mark.saturating_add(half_screen),
                ))
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                ..
            }) => Some(InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(5))),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                ..
            }) => Some(InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(5))),
            Event::Key(KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::UpdateUpperMark(0)),
            Event::Key(KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::SHIFT,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('G'),
                modifiers: KeyModifiers::SHIFT,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('G'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                // Input line numbers are one-based; upper_mark is zero-based.
                let mut position = ps
                    .prefix_num
                    .parse::<usize>()
                    .unwrap_or(usize::MAX)
                    .saturating_sub(1);
                if position == 0 {
                    position = usize::MAX;
                }
                Some(InputEvent::UpdateUpperMark(position))
            }

            Event::Key(KeyEvent {
                code,
                modifiers: KeyModifiers::NONE,
                ..
            }) if code == KeyCode::PageUp || code == KeyCode::Char('b') => Some(
                InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(ps.content_rows())),
            ),
            Event::Key(KeyEvent {
                code: c,
                modifiers: KeyModifiers::NONE,
                ..
            }) if c == KeyCode::PageDown || c == KeyCode::Char('f') => Some(
                InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(ps.content_rows())),
            ),

            Event::Resize(cols, rows) => {
                Some(InputEvent::UpdateTermArea(cols as usize, rows as usize))
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Some(InputEvent::UpdateLineNumber(!ps.line_numbers)),

            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Some(InputEvent::Exit),

            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers,
                ..
            }) if modifiers == KeyModifiers::CONTROL.intersection(KeyModifiers::SHIFT) => {
                Some(InputEvent::HorizontalScroll(!ps.screen.line_wrapping))
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::UpdateLeftMark(ps.left_mark.saturating_sub(1))),
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::UpdateLeftMark(ps.left_mark.saturating_add(1))),

            #[cfg(feature = "search")]
            Event::Key(KeyEvent {
                code: KeyCode::Char('/'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::Search(SearchMode::Forward)),
            #[cfg(feature = "search")]
            Event::Key(KeyEvent {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::Search(SearchMode::Reverse)),
            #[cfg(feature = "search")]
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
                if ps.search_state.search_mode == SearchMode::Reverse {
                    Some(InputEvent::MoveToPrevMatch(position))
                } else {
                    Some(InputEvent::MoveToNextMatch(position))
                }
            }
            #[cfg(feature = "search")]
            Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
                if ps.search_state.search_mode == SearchMode::Reverse {
                    Some(InputEvent::MoveToNextMatch(position))
                } else {
                    Some(InputEvent::MoveToPrevMatch(position))
                }
            }
            _ => None,
        }
    }
}
#[cfg(test)]
mod tests;
