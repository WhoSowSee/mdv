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

pub(super) fn build_callout_palette(theme: &Theme) -> HashMap<CalloutKind, Color> {
    let base = build_base_callout_palette(theme);
    remap_callout_palette(theme, &base)
}

pub(super) fn build_base_callout_palette(theme: &Theme) -> HashMap<CalloutKind, Color> {
    let mut palette = HashMap::new();
    let mut used: Vec<Color> = Vec::new();
    let fallback = collect_callout_fallback_colors(theme);

    let assignments = [
        (CalloutKind::Note, &theme.link),
        (CalloutKind::Abstract, &theme.table_header),
        (CalloutKind::Info, &theme.h4),
        (CalloutKind::Todo, &theme.emphasis),
        (CalloutKind::Tip, &theme.list_marker),
        (CalloutKind::Success, &theme.h2),
        (CalloutKind::Question, &theme.h5),
        (CalloutKind::Warning, &theme.warning),
        (CalloutKind::Failure, &theme.h1),
        (CalloutKind::Danger, &theme.error),
        (CalloutKind::Bug, &theme.h6),
        (CalloutKind::Example, &theme.code),
        (CalloutKind::Quote, &theme.quote),
    ];

    for (kind, primary) in assignments {
        let color = select_unique_callout_color(primary, &mut used, &fallback);
        palette.insert(kind, color);
    }

    palette
}

pub(super) fn remap_callout_palette(
    theme: &Theme,
    base: &HashMap<CalloutKind, Color>,
) -> HashMap<CalloutKind, Color> {
    let note = base_callout_color(theme, base, CalloutKind::Note);
    let abstract_color = base_callout_color(theme, base, CalloutKind::Abstract);
    let info = base_callout_color(theme, base, CalloutKind::Info);
    let todo = base_callout_color(theme, base, CalloutKind::Todo);
    let tip = base_callout_color(theme, base, CalloutKind::Tip);
    let success = base_callout_color(theme, base, CalloutKind::Success);
    let question = base_callout_color(theme, base, CalloutKind::Question);
    let danger = base_callout_color(theme, base, CalloutKind::Danger);
    let quote = base_callout_color(theme, base, CalloutKind::Quote);

    let mut palette = HashMap::new();
    palette.insert(CalloutKind::Note, note.clone());
    palette.insert(CalloutKind::Info, note.clone());

    palette.insert(CalloutKind::Abstract, abstract_color.clone());
    palette.insert(CalloutKind::Example, abstract_color.clone());

    palette.insert(CalloutKind::Todo, todo);
    palette.insert(CalloutKind::Tip, tip);

    palette.insert(CalloutKind::Success, quote);
    palette.insert(CalloutKind::Warning, success);

    palette.insert(CalloutKind::Question, info);

    palette.insert(CalloutKind::Failure, question.clone());
    palette.insert(CalloutKind::Danger, question.clone());
    palette.insert(CalloutKind::Bug, question);

    palette.insert(CalloutKind::Quote, danger);

    palette
}

pub(super) fn base_callout_color(
    theme: &Theme,
    base: &HashMap<CalloutKind, Color>,
    kind: CalloutKind,
) -> Color {
    base.get(&kind)
        .cloned()
        .unwrap_or_else(|| theme.text.clone())
}

pub(super) fn select_unique_callout_color(
    primary: &Color,
    used: &mut Vec<Color>,
    fallback: &[Color],
) -> Color {
    if !used.contains(primary) {
        used.push(primary.clone());
        return primary.clone();
    }

    if let Some(color) = fallback.iter().find(|candidate| !used.contains(candidate)) {
        used.push(color.clone());
        return color.clone();
    }

    primary.clone()
}

pub(super) fn collect_callout_fallback_colors(theme: &Theme) -> Vec<Color> {
    let candidates = [
        theme.h1.clone(),
        theme.h2.clone(),
        theme.h3.clone(),
        theme.h4.clone(),
        theme.h5.clone(),
        theme.h6.clone(),
        theme.link.clone(),
        theme.emphasis.clone(),
        theme.strong.clone(),
        theme.list_marker.clone(),
        theme.table_header.clone(),
        theme.table_border.clone(),
        theme.error.clone(),
        theme.warning.clone(),
        theme.quote.clone(),
        theme.code.clone(),
        theme.text.clone(),
        theme.text_light.clone(),
        theme.border.clone(),
        theme.syntax.keyword.clone(),
        theme.syntax.string.clone(),
        theme.syntax.comment.clone(),
        theme.syntax.number.clone(),
        theme.syntax.operator.clone(),
        theme.syntax.function.clone(),
        theme.syntax.variable.clone(),
        theme.syntax.type_name.clone(),
        Color::AnsiValue(33),
        Color::AnsiValue(39),
        Color::AnsiValue(45),
        Color::AnsiValue(51),
        Color::AnsiValue(75),
        Color::AnsiValue(81),
        Color::AnsiValue(99),
        Color::AnsiValue(111),
        Color::AnsiValue(135),
        Color::AnsiValue(141),
        Color::AnsiValue(171),
        Color::AnsiValue(203),
        Color::AnsiValue(215),
        Color::AnsiValue(221),
        Color::AnsiValue(227),
    ];

    let mut unique = Vec::new();
    for color in candidates {
        if !unique.contains(&color) {
            unique.push(color);
        }
    }

    unique
}
