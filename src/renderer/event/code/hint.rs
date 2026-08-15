use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn split_language_hint(hint: &str) -> Vec<String> {
        let mut parts = Vec::new();

        let trimmed = hint.trim();
        if trimmed.is_empty() {
            return parts;
        }

        for fragment in trimmed.split(LANGUAGE_SEPARATORS) {
            let mut piece = fragment.trim();
            if piece.is_empty() {
                continue;
            }

            if let Some((_, value)) = piece.split_once('=') {
                piece = value.trim();
            }

            if piece.starts_with('{') && piece.ends_with('}') && piece.len() > 2 {
                piece = &piece[1..piece.len() - 1];
            }

            let piece = piece
                .trim()
                .trim_matches(|c: char| matches!(c, '{' | '}' | '"' | '\'' | '`' | '.' | '!'));

            if piece.is_empty() {
                continue;
            }

            let piece = piece.strip_prefix("language-").unwrap_or(piece);
            let piece = piece.strip_prefix('.').unwrap_or(piece);

            let normalized = piece.trim();
            if normalized.is_empty() {
                continue;
            }

            let normalized = normalized.to_lowercase();
            if !parts
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&normalized))
            {
                parts.push(normalized);
            }
        }

        parts
    }
}
