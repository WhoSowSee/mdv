use super::*;
use crate::theme::SyntaxField;

pub(super) fn syntax_scopes(field: SyntaxField) -> &'static [(&'static str, Option<FontStyle>)] {
    match field {
        SyntaxField::Keyword => &[
            ("keyword", Some(FontStyle::BOLD)),
            ("storage", Some(FontStyle::BOLD)),
            ("storage.type.function", Some(FontStyle::BOLD)),
            ("storage.type.struct", Some(FontStyle::BOLD)),
            ("storage.type.impl", Some(FontStyle::BOLD)),
            ("storage.type.trait", Some(FontStyle::BOLD)),
            ("storage.type.enum", Some(FontStyle::BOLD)),
            ("storage.type.union", Some(FontStyle::BOLD)),
            ("storage.type.mod", Some(FontStyle::BOLD)),
            ("meta.directive", Some(FontStyle::BOLD)),
        ],
        SyntaxField::String => &[("string", None), ("constant.character.escape", None)],
        SyntaxField::Comment => &[
            ("comment", Some(FontStyle::ITALIC)),
            ("punctuation.definition.comment", Some(FontStyle::ITALIC)),
        ],
        SyntaxField::Number => &[("constant.numeric", None), ("constant.language", None)],
        SyntaxField::Operator => &[
            ("keyword.operator", None),
            ("punctuation", None),
            ("meta.brace", None),
        ],
        SyntaxField::Function => &[("entity.name.function", None), ("support.function", None)],
        SyntaxField::Variable => &[
            ("variable", None),
            ("variable.parameter", Some(FontStyle::ITALIC)),
            ("entity.other.attribute-name", None),
        ],
        SyntaxField::TypeName => &[
            ("entity.name.type", Some(FontStyle::BOLD)),
            ("support.type", Some(FontStyle::BOLD)),
            ("storage.type", Some(FontStyle::BOLD)),
        ],
    }
}

pub(super) fn field_for_scope(scope: &str) -> Option<SyntaxField> {
    SyntaxField::ALL
        .into_iter()
        .flat_map(|field| {
            syntax_scopes(field)
                .iter()
                .map(move |&(prefix, _)| (field, prefix))
        })
        .filter(|&(_, prefix)| {
            scope
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('.'))
        })
        .max_by_key(|&(_, prefix)| prefix.len())
        .map(|(field, _)| field)
}
