use std::fmt::Debug;

use crate::{
    ExitStrategy, LineNumbers, PromptLine, PromptRenderer,
    hooks::{Hook, HookCallback},
    input::{InputClassifier, InputEvent},
    minus_core::utils::display::AppendStyle,
};

#[cfg(feature = "search")]
use crate::search::SearchOpts;

#[derive(Debug, PartialEq, Eq)]
pub enum IoCommand {
    RedrawPrompt,
    RedrawDisplay,
    DrawAppendedText(usize, usize, AppendStyle),
    SetUpperMark(usize),
    RedrawSelection(usize, usize),
    #[cfg(feature = "search")]
    FetchSearchQuery,
}

#[non_exhaustive]
#[allow(private_interfaces)]
pub enum Command {
    UserInput(InputEvent),
    AppendData(String),
    SetData(String),
    SendMessage(String),
    SetTimedMessage {
        text: String,
        id: usize,
    },
    ClearMessage(usize),
    ShowPrompt(bool),
    SetPrompt(String),
    SetPromptRenderer(Option<PromptRenderer>),
    SetPromptPanel(Vec<PromptLine>),
    #[cfg(feature = "search")]
    SetSearchPrompt(Option<String>),

    LineWrapping(bool),
    SetLineNumbers(LineNumbers),
    FollowOutput(bool),

    SetExitStrategy(ExitStrategy),
    SetInputClassifier(Box<dyn InputClassifier + Send + Sync + 'static>),
    AddExitCallback(Box<dyn FnMut() + Send + Sync + 'static>),
    AddHook(Hook, u64, HookCallback),
    RemoveHook(Hook, u64),
    #[cfg(feature = "static_output")]
    SetRunNoOverflow(bool),
    #[cfg(feature = "search")]
    IncrementalSearchCondition(Box<dyn Fn(&SearchOpts) -> bool + Send + Sync + 'static>),

    Io(IoCommand),
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SetData(d1), Self::SetData(d2))
            | (Self::AppendData(d1), Self::AppendData(d2))
            | (Self::SetPrompt(d1), Self::SetPrompt(d2))
            | (Self::SendMessage(d1), Self::SendMessage(d2)) => d1 == d2,
            (
                Self::SetTimedMessage {
                    text: left_text,
                    id: left_id,
                },
                Self::SetTimedMessage {
                    text: right_text,
                    id: right_id,
                },
            ) => left_text == right_text && left_id == right_id,
            (Self::ClearMessage(left), Self::ClearMessage(right)) => left == right,
            #[cfg(feature = "search")]
            (Self::SetSearchPrompt(left), Self::SetSearchPrompt(right)) => left == right,
            (Self::LineWrapping(d1), Self::LineWrapping(d2)) => d1 == d2,
            (Self::SetLineNumbers(d1), Self::SetLineNumbers(d2)) => d1 == d2,
            (Self::ShowPrompt(d1), Self::ShowPrompt(d2)) => d1 == d2,
            (Self::SetExitStrategy(d1), Self::SetExitStrategy(d2)) => d1 == d2,
            #[cfg(feature = "static_output")]
            (Self::SetRunNoOverflow(d1), Self::SetRunNoOverflow(d2)) => d1 == d2,
            (Self::SetInputClassifier(_), Self::SetInputClassifier(_))
            | (Self::AddExitCallback(_), Self::AddExitCallback(_))
            | (Self::AddHook(..), Self::AddHook(..)) => true,
            (Self::SetPromptRenderer(left), Self::SetPromptRenderer(right)) => {
                left.is_some() == right.is_some()
            }
            (Self::SetPromptPanel(left), Self::SetPromptPanel(right)) => left == right,
            (Self::RemoveHook(h1, id1), Self::RemoveHook(h2, id2)) => h1 == h2 && id1 == id2,
            #[cfg(feature = "search")]
            (Self::IncrementalSearchCondition(_), Self::IncrementalSearchCondition(_)) => true,
            (Self::Io(a), Self::Io(b)) => a == b,
            _ => false,
        }
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetData(text) => write!(f, "SetData({text:?})"),
            Self::AppendData(text) => write!(f, "AppendData({text:?})"),
            Self::SetPrompt(text) => write!(f, "SetPrompt({text:?})"),
            Self::SetPromptRenderer(renderer) => {
                write!(f, "SetPromptRenderer({})", renderer.is_some())
            }
            Self::SetPromptPanel(lines) => write!(f, "SetPromptPanel({})", lines.len()),
            #[cfg(feature = "search")]
            Self::SetSearchPrompt(prompt) => write!(f, "SetSearchPrompt({prompt:?})"),
            Self::SendMessage(text) => write!(f, "SendMessage({text:?})"),
            Self::SetTimedMessage { text, id } => {
                write!(f, "SetTimedMessage({text:?}, {id})")
            }
            Self::ClearMessage(id) => write!(f, "ClearMessage({id})"),
            Self::SetLineNumbers(ln) => write!(f, "SetLineNumbers({ln:?})"),
            Self::LineWrapping(lw) => write!(f, "LineWrapping({lw:?})"),
            Self::SetExitStrategy(es) => write!(f, "SetExitStrategy({es:?})"),
            Self::SetInputClassifier(_) => write!(f, "SetInputClassifier"),
            Self::ShowPrompt(show) => write!(f, "ShowPrompt({show:?})"),
            #[cfg(feature = "search")]
            Self::IncrementalSearchCondition(_) => write!(f, "IncrementalSearchCondition"),
            Self::AddExitCallback(_) => write!(f, "AddExitCallback"),
            Self::AddHook(h, id, _) => write!(f, "AddHook({h:?}, {id})"),
            Self::RemoveHook(h, id) => write!(f, "RemoveHook({h:?}, {id})"),
            #[cfg(feature = "static_output")]
            Self::SetRunNoOverflow(val) => write!(f, "SetRunNoOverflow({val:?})"),
            Self::UserInput(input) => write!(f, "UserInput({input:?})"),
            Self::FollowOutput(follow_output) => write!(f, "FollowOutput({follow_output:?})"),
            Self::Io(c) => write!(f, "Io({c:?})"),
        }
    }
}
