use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// Remove SGR and OSC 8 sequences, preserving all other escape sequences.
pub fn strip_ansi(text: &str) -> String {
    strip_ansi_cow(text).into_owned()
}

/// Measure visible text with the same Unicode rules as `display_width`.
pub fn display_width_ansi(text: &str) -> usize {
    UnicodeWidthStr::width(strip_ansi_cow(text).as_ref())
}

fn strip_ansi_cow(text: &str) -> Cow<'_, str> {
    // SGR removal precedes OSC 8 matching, including SGR inside a hyperlink target.
    match strip_sequences(text, sgr_end) {
        Cow::Borrowed(text) => strip_sequences(text, osc_end),
        Cow::Owned(text) => match strip_sequences(&text, osc_end) {
            Cow::Borrowed(_) => Cow::Owned(text),
            Cow::Owned(clean) => Cow::Owned(clean),
        },
    }
}

fn strip_sequences(text: &str, end: fn(&[u8]) -> Option<usize>) -> Cow<'_, str> {
    let mut result: Option<String> = None;
    let mut copied = 0;
    let mut search = 0;
    while let Some(offset) = text[search..].find('\x1b') {
        let start = search + offset;
        search = start + 1;
        if let Some(length) = end(&text.as_bytes()[start..]) {
            result
                .get_or_insert_with(|| String::with_capacity(text.len()))
                .push_str(&text[copied..start]);
            search = start + length;
            copied = search;
        }
    }
    match result {
        Some(mut result) => {
            result.push_str(&text[copied..]);
            Cow::Owned(result)
        }
        None => Cow::Borrowed(text),
    }
}

fn sgr_end(bytes: &[u8]) -> Option<usize> {
    let rest = bytes.strip_prefix(b"\x1b[")?;
    let index = rest
        .iter()
        .position(|byte| !byte.is_ascii_digit() && *byte != b';')?;
    (rest[index] == b'm').then_some(index + 3)
}

fn osc_end(bytes: &[u8]) -> Option<usize> {
    let rest = bytes.strip_prefix(b"\x1b]8;;")?;
    let index = rest.iter().position(|byte| *byte == 0x1b)?;
    (rest.get(index + 1) == Some(&b'\\')).then_some(index + 7)
}
