use super::*;

pub(super) fn styled(
    text: &str,
    foreground: Option<Color>,
    background: Option<Color>,
    bold: bool,
    no_colors: bool,
) -> String {
    let mut style = AnsiStyle::new();
    if let Some(foreground) = foreground {
        style = style.fg(foreground);
    }
    if let Some(background) = background {
        style = style.bg(background);
    }
    if bold {
        style = style.bold();
    }
    style.apply(text, no_colors)
}

pub(super) fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

pub(super) fn sanitize_display(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

pub(super) fn truncate_plain(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if display_width(text) <= width {
        return text.to_string();
    }
    let content_width = width.saturating_sub(display_width(ELLIPSIS));
    let mut output = String::new();
    let mut used = 0;
    for character in text.chars() {
        let character_width = character.width().unwrap_or(0);
        if used + character_width > content_width {
            break;
        }
        output.push(character);
        used += character_width;
    }
    output.push_str(ELLIPSIS);
    output
}
