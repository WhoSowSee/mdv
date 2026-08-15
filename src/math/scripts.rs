use super::*;

pub(crate) fn convert_script(text: &str, kind: ScriptKind) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if let Some(mapped) = map_script_char(ch, kind) {
            out.push(mapped);
        } else {
            let marker = match kind {
                ScriptKind::Sup => "^",
                ScriptKind::Sub => "_",
            };
            return format!("{}({})", marker, text);
        }
    }
    out
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
            _ => None,
        },
    }
}

pub(super) fn literal_command(name: &str) -> Option<&'static str> {
    match name {
        "%" => Some("%"),
        "$" => Some("$"),
        "#" => Some("#"),
        "_" => Some("_"),
        "{" => Some("{"),
        "}" => Some("}"),
        "&" => Some("&"),
        "^" => Some("^"),
        _ => None,
    }
}

pub(super) fn spacing_command(name: &str) -> Option<&'static str> {
    match name {
        "," | ";" | ":" | "!" | " " => Some(" "),
        "quad" => Some("  "),
        "qquad" => Some("    "),
        _ => None,
    }
}

pub(super) fn delimiter_symbol(name: &str) -> Option<&'static str> {
    match name {
        "(" => Some("("),
        ")" => Some(")"),
        "[" => Some("["),
        "]" => Some("]"),
        "{" => Some("{"),
        "}" => Some("}"),
        "|" => Some("|"),
        "langle" => Some("⟨"),
        "rangle" => Some("⟩"),
        "lfloor" => Some("⌊"),
        "rfloor" => Some("⌋"),
        "lceil" => Some("⌈"),
        "rceil" => Some("⌉"),
        "vert" => Some("|"),
        "Vert" => Some("‖"),
        _ => None,
    }
}

pub(super) fn mathbb_symbol(letter: &str) -> Option<&'static str> {
    match letter {
        "R" => Some("ℝ"),
        "Z" => Some("ℤ"),
        "Q" => Some("ℚ"),
        "C" => Some("ℂ"),
        "N" => Some("ℕ"),
        _ => None,
    }
}
