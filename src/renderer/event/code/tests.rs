use super::*;
use crate::config::Config;
use crate::renderer::syntax_theme::CodeHighlightTheme;
use crate::theme::Theme;
use syntect::highlighting::Theme as SyntectTheme;
use syntect::parsing::SyntaxSet;

fn test_code_theme() -> CodeHighlightTheme {
    CodeHighlightTheme::syntect_only(SyntectTheme::default())
}

#[test]
fn resolve_syntax_returns_plain_text_when_guessing_disabled() {
    let config = Config {
        code_guessing: false,
        ..Config::default()
    };

    let theme = Theme::default();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let code_theme = test_code_theme();

    let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);

    let syntax_with_hint = renderer.resolve_syntax(Some("unknownlang"), "fn main() {}");
    assert_eq!(syntax_with_hint.name, "Plain Text");

    let syntax_without_hint = renderer.resolve_syntax(None, "fn main() {}");
    assert_eq!(syntax_without_hint.name, "Plain Text");
}

#[test]
fn code_block_icon_mapping_recognises_common_languages() {
    let config = Config::default();
    let theme = Theme::default();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let code_theme = test_code_theme();
    let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);

    assert_eq!(
        renderer.code_block_icon_for_hint("rust", "Rust"),
        "".to_string()
    );
    assert_eq!(
        renderer.code_block_icon_for_hint("python", "Python"),
        "".to_string()
    );
    assert_eq!(
        renderer.code_block_icon_for_hint("javascript", "JavaScript"),
        "".to_string()
    );
    assert_eq!(
        renderer.code_block_icon_for_hint("text", "Text"),
        "󰦪".to_string()
    );
    assert_eq!(
        renderer.code_block_icon_for_hint("shell", "Shell"),
        "".to_string()
    );
}

#[test]
fn code_block_icon_mapping_uses_default_icon_for_unknown_language() {
    let config = Config::default();
    let theme = Theme::default();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let code_theme = test_code_theme();
    let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);

    assert_eq!(
        renderer.code_block_icon_for_hint("unknownlang", "Unknownlang"),
        " ".to_string()
    );
}
#[test]
fn custom_code_block_icon_overrides_default() {
    let config = Config {
        custom_code_blocks: std::collections::HashMap::from([(
            "rust".to_string(),
            crate::custom_code_block::CustomCodeBlock {
                icon: Some("🦀".to_string()),
                label: None,
                aliases: Vec::new(),
            },
        )]),
        ..Config::default()
    };
    let theme = Theme::default();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let code_theme = test_code_theme();
    let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);

    assert_eq!(
        renderer.code_block_icon_for_hint("rust", "Rust"),
        "🦀".to_string()
    );
    assert_eq!(
        renderer.code_block_icon_for_hint("python", "Python"),
        "".to_string()
    );
}

#[test]
fn custom_default_icon_overrides_builtin_default() {
    let config = Config {
        custom_code_default_icon: Some("🚀".to_string()),
        ..Config::default()
    };
    let theme = Theme::default();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let code_theme = test_code_theme();
    let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);

    assert_eq!(
        renderer.code_block_icon_for_hint("unknownlang", "Unknownlang"),
        "🚀".to_string()
    );
    assert_eq!(
        renderer.code_block_icon_for_hint("rust", "Rust"),
        "".to_string()
    );
}
#[test]
fn highlight_code_reset_closes_last_line_without_phantom_row() {
    let config = Config::default();
    let theme = Theme::default();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let code_theme = test_code_theme();
    let renderer = EventRenderer::new(&config, &theme, &syntax_set, &code_theme);
    let highlighted = renderer
        .highlight_code("print(\"hi\")\n", Some("python"))
        .unwrap();

    // Reset before the trailing newline; after it, the split yields a phantom empty body row.
    assert!(
        highlighted.ends_with("\x1b[0m\n"),
        "reset must close the last visible line, got: {highlighted:?}"
    );
    assert_eq!(
        highlighted.lines().count(),
        1,
        "a single source line must render as exactly one row, got: {highlighted:?}"
    );
}
