use super::core::{CalloutFold, CalloutInfo, CalloutKind, CalloutState};
use super::{CalloutStyle, CowStr, EventRenderer, LinkStyle, Result, ThemeElement, create_style};

#[derive(Debug, Clone)]
struct HighlightSegment {
    text: String,
    highlighted: bool,
}

const CALLOUT_BUFFER_LIMIT: usize = 64;

enum CalloutBufferEval {
    Pending,
    Callout(CalloutMarker),
    NotCallout,
}

struct CalloutMarker {
    kind: CalloutKind,
    label: String,
    label_override: Option<String>,
    fold: Option<CalloutFold>,
    trailing: Option<String>,
    allow_label_override: bool,
    suppress_paragraph_break: bool,
}

enum CalloutDecision {
    RenderHeader {
        kind: CalloutKind,
        label: String,
        label_override: Option<String>,
        fold: Option<CalloutFold>,
        trailing: Option<String>,
        suppress_paragraph_break: bool,
    },
    AwaitLabelOverride,
    FlushBuffer(String),
    Pending,
}

mod callouts;
mod handling;
mod segments;
mod styled;
mod wrapping;
