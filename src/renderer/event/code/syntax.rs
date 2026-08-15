use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn try_lookup<'s>(
        &'s self,
        tokens: &[String],
        seen: &mut Vec<String>,
    ) -> Option<&'s SyntaxReference> {
        for token in tokens {
            if token.is_empty() {
                continue;
            }

            if seen
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(token))
            {
                continue;
            }
            seen.push(token.clone());

            if Self::is_plain_language(token) {
                return Some(self.syntax_set.find_syntax_plain_text());
            }

            for candidate in Self::expand_language_aliases(token) {
                if let Some(syntax) = self.lookup_syntax(&candidate) {
                    return Some(syntax);
                }
            }
        }

        None
    }

    pub(super) fn lookup_syntax<'s>(&'s self, token: &str) -> Option<&'s SyntaxReference> {
        if token.is_empty() {
            return None;
        }

        self.syntax_set
            .find_syntax_by_token(token)
            .or_else(|| self.syntax_set.find_syntax_by_name(token))
            .or_else(|| self.syntax_set.find_syntax_by_extension(token))
    }
}
