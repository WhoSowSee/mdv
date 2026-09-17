use super::ast::MathFont;

pub(super) fn apply(text: &str, fonts: &[MathFont]) -> String {
    text.chars()
        .map(|character| {
            fonts
                .iter()
                .rev()
                .fold(character, |character, font| match font {
                    MathFont::Script => script(character),
                    MathFont::Fraktur => fraktur(character),
                    MathFont::Bold => bold(character),
                })
        })
        .collect()
}

fn script(character: char) -> char {
    match character {
        'A' => '𝒜',
        'B' => 'ℬ',
        'C' => '𝒞',
        'D' => '𝒟',
        'E' => 'ℰ',
        'F' => 'ℱ',
        'G' => '𝒢',
        'H' => 'ℋ',
        'I' => 'ℐ',
        'J' => '𝒥',
        'K' => '𝒦',
        'L' => 'ℒ',
        'M' => 'ℳ',
        'N' => '𝒩',
        'O' => '𝒪',
        'P' => '𝒫',
        'Q' => '𝒬',
        'R' => 'ℛ',
        'S' => '𝒮',
        'T' => '𝒯',
        'U' => '𝒰',
        'V' => '𝒱',
        'W' => '𝒲',
        'X' => '𝒳',
        'Y' => '𝒴',
        'Z' => '𝒵',
        'a' => '𝒶',
        'b' => '𝒷',
        'c' => '𝒸',
        'd' => '𝒹',
        'e' => 'ℯ',
        'f' => '𝒻',
        'g' => 'ℊ',
        'h' => '𝒽',
        'i' => '𝒾',
        'j' => '𝒿',
        'k' => '𝓀',
        'l' => '𝓁',
        'm' => '𝓂',
        'n' => '𝓃',
        'o' => 'ℴ',
        'p' => '𝓅',
        'q' => '𝓆',
        'r' => '𝓇',
        's' => '𝓈',
        't' => '𝓉',
        'u' => '𝓊',
        'v' => '𝓋',
        'w' => '𝓌',
        'x' => '𝓍',
        'y' => '𝓎',
        'z' => '𝓏',
        _ => character,
    }
}

fn fraktur(character: char) -> char {
    match character {
        'C' => 'ℭ',
        'H' => 'ℌ',
        'I' => 'ℑ',
        'R' => 'ℜ',
        'Z' => 'ℨ',
        'A'..='Z' => offset(character, 'A', 0x1D504),
        'a'..='z' => offset(character, 'a', 0x1D51E),
        _ => character,
    }
}

fn bold(character: char) -> char {
    match character {
        'A'..='Z' => offset(character, 'A', 0x1D400),
        'a'..='z' => offset(character, 'a', 0x1D41A),
        '0'..='9' => offset(character, '0', 0x1D7CE),
        'Α'..='Ρ' => offset(character, 'Α', 0x1D6A8),
        'Σ'..='Ω' => offset(character, 'Σ', 0x1D6BA),
        'α'..='ω' => offset(character, 'α', 0x1D6C2),
        'ϵ' => '𝛜',
        'ϑ' => '𝛝',
        'ϖ' => '𝛡',
        'ϱ' => '𝛠',
        'ϕ' => '𝛟',
        _ => character,
    }
}

fn offset(character: char, start: char, target: u32) -> char {
    char::from_u32(target + character as u32 - start as u32).unwrap_or(character)
}
