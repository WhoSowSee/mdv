use std::collections::VecDeque;

pub mod commands;
pub mod ev_handler;
#[cfg(any(feature = "dynamic_output", feature = "static_output"))]
pub mod init;
#[cfg(any(feature = "dynamic_output", feature = "static_output"))]
mod panic_hook;
pub mod utils;

// Tests must reset this process-global state between pager runs.
pub static RUNMODE: parking_lot::Mutex<RunMode> = parking_lot::const_mutex(RunMode::Uninitialized);

use commands::Command;

pub struct CommandQueue(VecDeque<Command>);

impl CommandQueue {
    pub fn new() -> Self {
        Self(VecDeque::with_capacity(10))
    }
    pub const fn new_zero() -> Self {
        Self(VecDeque::new())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_back(&mut self, value: Command) {
        self.0.push_back(value);
    }

    pub fn pop_front(&mut self) -> Option<Command> {
        self.0.pop_front()
    }
}

/// Define the modes in which minus can run
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RunMode {
    #[cfg(feature = "static_output")]
    Static,
    #[cfg(feature = "dynamic_output")]
    Dynamic,
    Uninitialized,
}

impl RunMode {
    /// Returns whether no pager is running.
    #[must_use]
    pub fn is_uninitialized(self) -> bool {
        self == Self::Uninitialized
    }
}
