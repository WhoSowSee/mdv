use super::*;

#[test]
fn builtin_themes_define_opaque_pager_status_bar() {
    for (name, source) in BUILTIN_THEME_FILES {
        let yaml: serde_yaml::Value = serde_yaml::from_str(source).unwrap();
        assert_eq!(
            yaml.get("pager_status_bar_transparent"),
            Some(&serde_yaml::Value::Bool(false)),
            "built-in theme '{name}' must explicitly use an opaque pager status bar"
        );
    }
}

#[test]
fn test_theme_manager() {
    let manager = ThemeManager::new();
    assert!(manager.get_theme("terminal").is_ok());
    assert!(manager.get_theme("monokai").is_ok());
    assert!(manager.get_theme("catppuccin").is_ok());
    assert!(manager.get_theme("Catppuccin").is_ok());
    assert!(manager.get_theme("Terminal").is_ok());
    assert!(manager.get_theme("MoNoKaI").is_ok());
    assert!(manager.get_theme("nonexistent").is_err());
}

#[test]
fn test_theme_luminosity() {
    let theme = Theme::default();
    let lum = calculate_theme_luminosity(&theme);
    assert!((0.0..=1.0).contains(&lum));
}

#[test]
fn test_create_style() {
    let theme = Theme::default();
    let style = create_style(&theme, ThemeElement::H1);
    // Should have bold attribute for H1
    assert!(style.bold);
}

#[test]
fn test_apply_custom_theme_overrides() {
    let mut theme = Theme::default();
    apply_custom_theme(
        &mut theme,
        "h1=#ffffff; link=187,154,247; background=none; strong=rgb(10,20,30); strong_emphasis=#070809; highlight=#0a0b0c; highlight_bg=#112233; emphasis_background=#0d0e0f; code_background=none; line_number=#010203; line_number_separator=#040506",
    )
    .expect("custom theme overrides should be applied");

    assert!(matches!(
        theme.h1,
        Color::Rgb {
            r: 255,
            g: 255,
            b: 255
        }
    ));
    assert!(matches!(
        theme.link,
        Color::Rgb {
            r: 187,
            g: 154,
            b: 247
        }
    ));
    assert!(matches!(
        theme.strong,
        Color::Rgb {
            r: 10,
            g: 20,
            b: 30
        }
    ));
    assert!(theme.background.is_none());
    assert_eq!(theme.strong_emphasis, Some(Color::Rgb { r: 7, g: 8, b: 9 }));
    assert_eq!(
        theme.highlight,
        Some(Color::Rgb {
            r: 10,
            g: 11,
            b: 12
        })
    );
    assert_eq!(
        theme.emphasis_background,
        Some(Color::Rgb {
            r: 13,
            g: 14,
            b: 15
        })
    );
    assert!(theme.code_background.is_none());
    assert!(matches!(
        theme.highlight_background,
        Color::Rgb {
            r: 0x11,
            g: 0x22,
            b: 0x33
        }
    ));
    assert_eq!(theme.line_number, Color::Rgb { r: 1, g: 2, b: 3 });
    assert_eq!(theme.line_number_separator, Color::Rgb { r: 4, g: 5, b: 6 });
}

#[test]
fn custom_theme_can_enable_transparent_pager_status_bar() {
    let mut theme = Theme::default();

    apply_custom_theme(&mut theme, "pager_status_bar_transparent=true")
        .expect("pager status bar transparency override should be accepted");

    let yaml = serde_yaml::to_value(theme).unwrap();
    assert_eq!(
        yaml.get("pager_status_bar_transparent"),
        Some(&serde_yaml::Value::Bool(true))
    );
}

#[test]
fn test_apply_custom_code_theme_overrides() {
    let mut theme = Theme::default();
    apply_custom_code_theme(&mut theme, "keyword=#123456;type=42,42,42")
        .expect("custom code theme overrides should be applied");

    assert!(matches!(
        theme.syntax.keyword,
        Color::Rgb {
            r: 18,
            g: 52,
            b: 86
        }
    ));
    assert!(matches!(
        theme.syntax.type_name,
        Color::Rgb {
            r: 42,
            g: 42,
            b: 42
        }
    ));
}

#[test]
fn removed_code_block_override_is_rejected() {
    let mut theme = Theme::default();
    let error = apply_custom_theme(&mut theme, "code_block=#ffffff")
        .expect_err("removed code_block override must be rejected");
    let error_chain = format!("{error:#}");
    assert!(
        error_chain.contains("Unknown key for custom theme: 'code_block'."),
        "unexpected error: {error_chain}"
    );
}

#[test]
fn test_apply_custom_theme_plain_ansi_value() {
    let mut theme = Theme::default();
    apply_custom_theme(&mut theme, "border=123").expect("plain ANSI value should be accepted");
    assert!(matches!(theme.border, Color::AnsiValue(123)));
}

#[test]
fn test_apply_custom_theme_ansi_function() {
    let mut theme = Theme::default();
    apply_custom_theme(&mut theme, "border=ansi(42)").expect("ansi() notation should be accepted");
    assert!(matches!(theme.border, Color::AnsiValue(42)));
}

#[test]
fn test_apply_custom_theme_rejects_ansi_without_parens() {
    let mut theme = Theme::default();
    let result = apply_custom_theme(&mut theme, "border=ansi42");
    assert!(result.is_err());
}
