use super::*;
use crate::block_spacing::BlockElement;
use crate::cli::{CalloutStyle, MathBlockStyle};
use crate::theme::{Color, Theme, apply_custom_code_theme, apply_custom_theme};

pub(super) fn parse_with_structured_preset(
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
    let _environment = env_lock();
    let config = parse_with_structured_preset(
        r##"
custom_theme:
  text: "#010203"
  horizontal_rule: "#070809"
  details_border: darkgrey
  emphasis: {text: cyan, background: null}
  code: {text: red, background: blue}
  line_number:
    number: red
    separator: grey
  callout:
    palette:
      success: cyan
  syntax:
    number: magenta
  background: null
  pager:
    transparent: true
    help: cyan
    search: {background: red}
    search_current: {text: reset, background: null}
custom_code_theme:
  keyword: "#040506"
  number: 42
"##,
        "custom_theme: {pager: {file_name: yellow, title: magenta}}\n",
        &["--custom-theme", "pager:title=red"],
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
    assert_eq!(theme.horizontal_rule, Some(Color::Rgb { r: 7, g: 8, b: 9 }));
    assert_eq!(theme.line_number.number, Color::Red);
    assert_eq!(theme.details_border, Some(Color::DarkGrey));
    assert_eq!(theme.emphasis.text, Color::Cyan);
    assert!(theme.emphasis.background.is_none());
    assert_eq!(theme.code.text, Color::Red);
    assert_eq!(theme.code.background, Some(Color::Blue));
    assert_eq!(theme.callout.palette.success, Color::Cyan);
    assert!(theme.pager.transparent);
    assert_eq!(theme.pager.help, Some(Color::Cyan));
    assert_eq!(theme.pager.title, Some(Color::Red));
    assert_eq!(theme.pager.file_name, Some(Color::Yellow));
    assert_eq!(theme.pager.search.background, Some(Color::Red));
    assert_eq!(theme.pager.search_current.text, Some(Color::Reset));
    assert!(theme.pager.search_current.background.is_none());
    assert!(matches!(
        theme.syntax.keyword,
        Color::Rgb { r: 4, g: 5, b: 6 }
    ));
    assert!(matches!(theme.syntax.number, Color::AnsiValue(42)));
}

#[test]
fn structured_block_spacing_and_callout_style_load_from_config() {
    let _environment = env_lock();
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
fn preset_and_cli_override_math_block_style() {
    let _environment = env_lock();
    let preset = parse_with_structured_preset(
        "math_block_style: pretty\n",
        "math_block_style: basic\n",
        &[],
    );
    assert_eq!(preset.math_block_style, MathBlockStyle::Basic);

    let config = parse_with_structured_preset(
        "math_block_style: pretty\n",
        "math_block_style: basic\n",
        &["--math-block-style", "simple"],
    );
    assert_eq!(config.math_block_style, MathBlockStyle::Simple);
}

#[test]
fn structured_callout_and_code_block_overrides_load_from_config() {
    let _environment = env_lock();
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
    let _environment = env_lock();
    let config = parse_with_config(
        r#"
checkbox_style: square
custom_checkbox:
  " ":
    icon: "U"
    color: yellow
  x:
    color: green
list_style: "type:unicode;size:large"
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
fn structured_block_spacing_keeps_preset_and_cli_priority() {
    let _environment = env_lock();
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
    let _environment = env_lock();
    let config = parse_with_config(
        r##"
custom_theme: "text=#010203"
custom_code_theme: "keyword=#040506"
block_spacing: "paragraph:top=0,bottom=2"
callout_style: "pretty:show-icons;fold-icons"
checkbox_style: square
custom_checkbox: "x:X:green"
list_style: "type:unicode;size:large"
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
fn empty_structured_preset_resets_block_spacing() {
    let _environment = env_lock();
    let config = parse_with_structured_preset(
        "block_spacing:\n  paragraph:\n    top: 2\n",
        "block_spacing: {}\n",
        &[],
    );

    let paragraph = config.block_spacing.spacing(BlockElement::Paragraph);
    assert_eq!((paragraph.top, paragraph.bottom), (0, 1));
}
