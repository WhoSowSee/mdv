use super::*;

pub(super) fn blockquote_kind_info(kind: BlockQuoteKind) -> (CalloutKind, String) {
    match kind {
        BlockQuoteKind::Note => (CalloutKind::Note, "note".to_string()),
        BlockQuoteKind::Tip => (CalloutKind::Tip, "tip".to_string()),
        BlockQuoteKind::Important => (CalloutKind::Tip, "important".to_string()),
        BlockQuoteKind::Warning => (CalloutKind::Warning, "warning".to_string()),
        BlockQuoteKind::Caution => (CalloutKind::Warning, "caution".to_string()),
    }
}

pub(in crate::renderer::event) fn callout_palette_color(
    theme: &Theme,
    kind: CalloutKind,
) -> (&Color, &'static str) {
    let palette = &theme.callout.palette;
    match kind {
        CalloutKind::Note => (&palette.note, "note"),
        CalloutKind::Abstract => (&palette.abstract_color, "abstract"),
        CalloutKind::Info => (&palette.info, "info"),
        CalloutKind::Todo => (&palette.todo, "todo"),
        CalloutKind::Tip => (&palette.tip, "tip"),
        CalloutKind::Success => (&palette.success, "success"),
        CalloutKind::Question => (&palette.question, "question"),
        CalloutKind::Warning => (&palette.warning, "warning"),
        CalloutKind::Failure => (&palette.failure, "failure"),
        CalloutKind::Danger => (&palette.danger, "danger"),
        CalloutKind::Bug => (&palette.bug, "bug"),
        CalloutKind::Example => (&palette.example, "example"),
        CalloutKind::Quote => (&palette.quote, "quote"),
        CalloutKind::Properties => (&theme.front_matter.title, "properties"),
    }
}
