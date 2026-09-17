use crate::utils::{WrapMode, display_width};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn wrap_flat_math(text: &str, width: usize, mode: WrapMode) -> String {
    if width == 0 || mode == WrapMode::None || display_width(text) <= width {
        return text.to_string();
    }
    let units = match mode {
        WrapMode::Word => text.split_word_bounds().collect::<Vec<_>>(),
        WrapMode::Character => text.graphemes(true).collect::<Vec<_>>(),
        WrapMode::None => return text.to_string(),
    };
    let mut lines = Vec::new();
    let mut current = String::new();
    for unit in units {
        if unit.trim().is_empty() && current.is_empty() {
            continue;
        }
        if display_width(&current) + display_width(unit) <= width {
            current.push_str(unit);
            continue;
        }
        if !current.trim().is_empty() {
            lines.push(current.trim_end().to_string());
            current.clear();
        }
        if display_width(unit) <= width {
            current.push_str(unit.trim_start());
        } else {
            current.push_str(unit);
        }
    }
    if !current.trim().is_empty() {
        lines.push(current.trim_end().to_string());
    }
    lines.join("\n")
}
