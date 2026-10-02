use super::builder::{push_scope, register_color};
use super::scopes::{field_for_scope, syntax_scopes};
use super::*;
use crate::theme::{SyntaxField, SyntaxTheme};

pub(crate) fn apply_syntax_overrides(
    theme: &mut CodeHighlightTheme,
    syntax: &SyntaxTheme,
    fields: &[SyntaxField],
) {
    if fields.is_empty() {
        return;
    }
    let colors = fields
        .iter()
        .map(|&field| {
            (
                field,
                register_color(&mut theme.palette, field.color(syntax)),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut scopes = Vec::new();
    for item in std::mem::take(&mut theme.syntect.scopes) {
        for selector in item.scope.selectors {
            let mut style = item.style;
            let field = selector
                .path
                .as_slice()
                .iter()
                .rev()
                .find_map(|scope| field_for_scope(&scope.build_string()));
            if let Some(color) = field.and_then(|field| colors.get(&field)) {
                style.foreground = Some(*color);
            }
            scopes.push(ThemeItem {
                scope: ScopeSelectors {
                    selectors: vec![selector],
                },
                style,
            });
        }
    }
    for &field in fields {
        for &(selector, _) in syntax_scopes(field) {
            push_scope(
                &mut scopes,
                &mut theme.palette,
                selector,
                field.color(syntax),
                None,
            );
        }
    }
    theme.syntect.scopes = scopes;
}
