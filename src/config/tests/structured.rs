use super::*;
use crate::block_spacing::BlockElement;
use crate::cli::CalloutStyle;
use crate::theme::{Color, Theme, apply_custom_code_theme, apply_custom_theme};

fn parse_with_structured_preset(
    config_contents: &str,
    preset_settings: &str,
    extra_args: &[&str],
) -> Config {
    let temp_dir = TempDir::new().expect("create temp dir");
    std::fs::write(temp_dir.path().join("config.yaml"), config_contents).expect("write config");
    write_preset(
        temp_dir.path(),
        "structured.yaml",
        &format!("name: structured\n{preset_settings}"),
    );

    let mut args = vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--preset"),
        OsString::from("structured"),
    ];
    args.extend(extra_args.iter().map(|arg| OsString::from(*arg)));
    let (cli, matches) = parse_cli_from(args);
    Config::from_cli(&cli, &matches).expect("load structured preset")
}

#[test]
fn structured_theme_overrides_load_from_config() {
    let config = parse_with_config(
        r##"
custom_theme:
  text: "#010203"
  background: null
  pager_status_bar_transparent: true
custom_code_theme:
  keyword: "#040506"
  number: 42
"##,
    );

    let mut theme = Theme::default();
    apply_custom_theme(
        &mut theme,
        config.custom_theme.as_deref().expect("custom theme"),
    )
    .expect("apply custom theme");
    apply_custom_code_theme(
        &mut theme,
        config
            .custom_code_theme
            .as_deref()
            .expect("custom code theme"),
    )
    .expect("apply custom code theme");

    assert!(matches!(theme.text, Color::Rgb { r: 1, g: 2, b: 3 }));
    assert!(theme.background.is_none());
    assert!(theme.pager_status_bar_transparent);
    assert!(matches!(
        theme.syntax.keyword,
        Color::Rgb { r: 4, g: 5, b: 6 }
    ));
    assert!(matches!(theme.syntax.number, Color::AnsiValue(42)));
}

#[test]
fn structured_block_spacing_and_callout_style_load_from_config() {
    let config = parse_with_config(
        r#"
block_spacing:
  paragraph:
    top: 0
    bottom: 2
  callout:
    top: 3
callout_style:
  style: pretty
  show_icons: true
  fold_icons: true
  label_inside: true
  uppercase: true
"#,
    );

    let paragraph = config.block_spacing.spacing(BlockElement::Paragraph);
    let callout = config.block_spacing.spacing(BlockElement::Callout);
    assert_eq!((paragraph.top, paragraph.bottom), (0, 2));
    assert_eq!((callout.top, callout.bottom), (3, 1));
    assert_eq!(config.callout_style.style, CalloutStyle::Pretty);
    assert!(config.callout_style.show_icons);
    assert!(config.callout_style.show_fold_icons);
    assert!(config.callout_style.label_inside);
    assert!(config.callout_style.uppercase);
}

#[test]
fn structured_callout_and_code_block_overrides_load_from_config() {
    let config = parse_with_config(
        r##"
custom_callout:
  important:
    icon: "!"
    color: "#ff0000"
  note:
    color: 42
custom_code_block:
  default:
    icon: "?"
  rust:
    icon: "R"
    label: Rust
    aliases:
      - rs
      - rustlang
"##,
    );

    let important = config
        .custom_callouts
        .get("important")
        .expect("important callout");
    assert_eq!(important.icon.as_deref(), Some("!"));
    assert!(matches!(
        important.color,
        Some(Color::Rgb { r: 255, g: 0, b: 0 })
    ));
    assert!(matches!(
        config
            .custom_callouts
            .get("note")
            .and_then(|entry| entry.color.as_ref()),
        Some(Color::AnsiValue(42))
    ));

    let rust = config
        .custom_code_blocks
        .get("rust")
        .expect("rust code block");
    assert_eq!(rust.icon.as_deref(), Some("R"));
    assert_eq!(rust.label.as_deref(), Some("Rust"));
    assert_eq!(rust.aliases, ["rs", "rustlang"]);
    assert_eq!(config.custom_code_default_icon.as_deref(), Some("?"));
}

#[test]
fn structured_checkbox_and_list_overrides_load_from_config() {
    let config = parse_with_config(
        r#"
pretty_checkbox: square
custom_checkbox:
  " ":
    icon: "U"
    color: yellow
  x:
    color: green
pretty_list: "type:unicode;size:large"
custom_list:
  1:
    icon: "*"
    color: yellow
  2:
    color: red
"#,
    );

    let unchecked = config
        .checkbox_overrides
        .get(&' ')
        .expect("unchecked override");
    assert_eq!(unchecked.icon.as_deref(), Some("U"));
    assert!(matches!(unchecked.color, Some(Color::Yellow)));
    assert!(matches!(
        config
            .checkbox_overrides
            .get(&'x')
            .and_then(|entry| entry.color.as_ref()),
        Some(Color::Green)
    ));

    let (level_one_icon, level_one_color) = config.list_marker.resolve(1).expect("level one");
    assert_eq!(level_one_icon, "*");
    assert!(matches!(level_one_color, Some(Color::Yellow)));
    let (level_two_icon, level_two_color) = config.list_marker.resolve(2).expect("level two");
    assert_eq!(level_two_icon, "▪");
    assert!(matches!(level_two_color, Some(Color::Red)));
}

#[test]
fn structured_preset_and_cli_keep_field_level_priority() {
    let config_contents = "custom_callout:\n  config:\n    icon: C\n  retained:\n    icon: R\n";
    let preset_settings = "custom_callout:\n  preset:\n    icon: P\n";

    let preset_config = parse_with_structured_preset(config_contents, preset_settings, &[]);
    assert_eq!(preset_config.custom_callouts.len(), 1);
    assert!(preset_config.custom_callouts.contains_key("preset"));

    let config = parse_with_structured_preset(
        config_contents,
        preset_settings,
        &["--custom-callout", "cli:icon=L"],
    );
    assert_eq!(config.custom_callouts.len(), 1);
    assert!(config.custom_callouts.contains_key("cli"));
}

#[test]
fn structured_block_spacing_keeps_preset_and_cli_priority() {
    let config = parse_with_structured_preset(
        "block_spacing:\n  paragraph:\n    top: 2\n    bottom: 3\n  callout:\n    top: 4\n",
        "block_spacing:\n  paragraph:\n    top: 1\n",
        &["--block-spacing", "paragraph:bottom=0"],
    );

    let paragraph = config.block_spacing.spacing(BlockElement::Paragraph);
    let callout = config.block_spacing.spacing(BlockElement::Callout);
    assert_eq!((paragraph.top, paragraph.bottom), (1, 0));
    assert_eq!((callout.top, callout.bottom), (1, 1));
}

#[test]
fn structured_settings_keep_legacy_scalar_forms() {
    let config = parse_with_config(
        r##"
custom_theme: "text=#010203"
custom_code_theme: "keyword=#040506"
block_spacing: "paragraph:top=0,bottom=2"
callout_style: "pretty:show-icons;fold-icons"
pretty_checkbox: square
custom_checkbox: "x:X:green"
pretty_list: "type:unicode;size:large"
custom_list: "1:*:yellow"
custom_callout: "important:icon=!,color=#ff0000"
custom_code_block: "rust:icon=R,label=Rust,aliases=rs|rustlang"
"##,
    );

    assert!(config.custom_theme.is_some());
    assert!(config.custom_code_theme.is_some());
    assert_eq!(
        config.block_spacing.spacing(BlockElement::Paragraph).bottom,
        2
    );
    assert!(config.callout_style.show_icons);
    assert!(config.callout_style.show_fold_icons);
    assert!(config.checkbox_overrides.contains_key(&'x'));
    assert_eq!(config.list_marker.resolve(1).expect("list marker").0, "*");
    assert!(config.custom_callouts.contains_key("important"));
    assert!(config.custom_code_blocks.contains_key("rust"));
}

#[test]
fn empty_structured_preset_values_clear_lower_priority_settings() {
    let config = parse_with_structured_preset(
        "custom_callout:\n  config:\n    icon: C\nblock_spacing:\n  paragraph:\n    top: 2\n",
        "custom_callout: {}\nblock_spacing: {}\n",
        &[],
    );

    assert!(config.custom_callouts.is_empty());
    let paragraph = config.block_spacing.spacing(BlockElement::Paragraph);
    assert_eq!((paragraph.top, paragraph.bottom), (0, 1));
}
