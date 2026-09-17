use super::*;
use unicode_segmentation::UnicodeSegmentation;

pub(super) struct ScriptConversion {
    pub(super) output: String,
    pub(super) fully_converted: bool,
}

pub(super) fn convert_script(text: &str, kind: ScriptKind) -> ScriptConversion {
    let mut converted = String::new();
    let mut source = String::new();
    let mut fully_converted = true;
    let mut has_script_character = false;

    for character in text.chars() {
        if character.is_whitespace() {
            continue;
        }
        source.push(character);
        if let Some(mapped) = map_script_char(character, kind) {
            converted.push(mapped);
            has_script_character |= mapped != character;
        } else {
            fully_converted = false;
        }
    }

    fully_converted &= has_script_character || source.is_empty();
    if fully_converted {
        return ScriptConversion {
            output: converted,
            fully_converted,
        };
    }

    let marker = match kind {
        ScriptKind::Sup => '^',
        ScriptKind::Sub => '_',
    };

    let mut graphemes = source.graphemes(true);
    let single_grapheme = graphemes.next().is_some() && graphemes.next().is_none();
    let output = if single_grapheme {
        format!("{marker}{source}")
    } else {
        format!("{marker}({source})")
    };
    ScriptConversion {
        output,
        fully_converted,
    }
}

pub(crate) fn convert_html_script(text: &str, kind: ScriptKind) -> String {
    text.chars()
        .map(|character| map_script_char(character, kind).unwrap_or(character))
        .collect()
}

pub(super) fn map_script_char(ch: char, kind: ScriptKind) -> Option<char> {
    match kind {
        ScriptKind::Sup => match ch {
            '0' => Some('⁰'),
            '1' => Some('¹'),
            '2' => Some('²'),
            '3' => Some('³'),
            '4' => Some('⁴'),
            '5' => Some('⁵'),
            '6' => Some('⁶'),
            '7' => Some('⁷'),
            '8' => Some('⁸'),
            '9' => Some('⁹'),
            '+' => Some('⁺'),
            '-' | '−' => Some('⁻'),
            '=' => Some('⁼'),
            '(' => Some('⁽'),
            ')' => Some('⁾'),
            'a' => Some('ᵃ'),
            'b' => Some('ᵇ'),
            'c' => Some('ᶜ'),
            'd' => Some('ᵈ'),
            'e' => Some('ᵉ'),
            'f' => Some('ᶠ'),
            'g' => Some('ᵍ'),
            'h' => Some('ʰ'),
            'i' => Some('ⁱ'),
            'j' => Some('ʲ'),
            'k' => Some('ᵏ'),
            'l' => Some('ˡ'),
            'm' => Some('ᵐ'),
            'n' => Some('ⁿ'),
            'o' => Some('ᵒ'),
            'p' => Some('ᵖ'),
            'r' => Some('ʳ'),
            's' => Some('ˢ'),
            't' => Some('ᵗ'),
            'u' => Some('ᵘ'),
            'v' => Some('ᵛ'),
            'w' => Some('ʷ'),
            'x' => Some('ˣ'),
            'y' => Some('ʸ'),
            'z' => Some('ᶻ'),
            '⊤' => Some('ᵀ'),
            ',' | '.' | ';' | ':' => Some(ch),
            _ => None,
        },
        ScriptKind::Sub => match ch {
            '0' => Some('₀'),
            '1' => Some('₁'),
            '2' => Some('₂'),
            '3' => Some('₃'),
            '4' => Some('₄'),
            '5' => Some('₅'),
            '6' => Some('₆'),
            '7' => Some('₇'),
            '8' => Some('₈'),
            '9' => Some('₉'),
            '+' => Some('₊'),
            '-' | '−' => Some('₋'),
            '=' => Some('₌'),
            '(' => Some('₍'),
            ')' => Some('₎'),
            'a' => Some('ₐ'),
            'b' => Some('ᵦ'),
            'e' => Some('ₑ'),
            'h' => Some('ₕ'),
            'i' => Some('ᵢ'),
            'j' => Some('ⱼ'),
            'k' => Some('ₖ'),
            'l' => Some('ₗ'),
            'm' => Some('ₘ'),
            'n' => Some('ₙ'),
            'o' => Some('ₒ'),
            'p' => Some('ₚ'),
            'r' => Some('ᵣ'),
            's' => Some('ₛ'),
            't' => Some('ₜ'),
            'u' => Some('ᵤ'),
            'v' => Some('ᵥ'),
            'x' => Some('ₓ'),
            ',' | '.' | ';' | ':' => Some(ch),
            _ => None,
        },
    }
}
