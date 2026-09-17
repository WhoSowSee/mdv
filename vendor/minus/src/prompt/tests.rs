use super::{PromptAttribute, PromptColor, PromptError, PromptLine, PromptSpan, PromptStyle};

#[test]
fn prompt_span_rejects_multiline_and_control_text() {
    let style = PromptStyle::default();

    assert_eq!(
        PromptSpan::new("first\nsecond", style),
        Err(PromptError::MultilineText)
    );
    assert_eq!(
        PromptSpan::new("unsafe\x1b[31m", style),
        Err(PromptError::ControlCharacter('\x1b'))
    );
}

#[test]
fn prompt_line_handles_unicode_width_and_right_alignment() {
    let line = PromptLine::new()
        .left(PromptSpan::new("界界界", PromptStyle::default()).unwrap())
        .right(PromptSpan::new("XY", PromptStyle::default()).unwrap())
        .truncation_indicator(PromptSpan::new("…", PromptStyle::default()).unwrap());

    assert_eq!(line.render_plain(5), "界…XY");

    let narrow = PromptLine::new().right(PromptSpan::new("界", PromptStyle::default()).unwrap());
    assert_eq!(narrow.render_plain(1), " ");
}

#[test]
fn prompt_line_pads_to_width_and_resets_styles() {
    let fill = PromptStyle::default().background(PromptColor::Rgb {
        r: 36,
        g: 36,
        b: 36,
    });
    let brand = PromptStyle::default()
        .foreground(PromptColor::AnsiValue(154))
        .attribute(PromptAttribute::Bold);
    let line = PromptLine::new()
        .left(PromptSpan::new("MDV", brand).unwrap())
        .right(PromptSpan::new("HELP", fill).unwrap())
        .fill_style(fill);

    assert_eq!(line.render_plain(10), "MDV   HELP");
    let rendered = line.render(10);
    assert!(rendered.contains("\x1b["));
    assert!(rendered.ends_with("\x1b[0m"));
}
