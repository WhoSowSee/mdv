use super::structured::parse_with_structured_preset;
use super::*;
use crate::theme::{Color, ColorSource};

const BASE: &str = r#"
custom_callout:
  note: {icon: N, color: green}
  tip: {icon: T, color: yellow}
custom_code_block:
  default: {icon: D}
  rust: {icon: R, label: Rust, aliases: [rs, old]}
  python: {icon: P, label: Python}
checkbox_style: square
custom_checkbox:
  x: {icon: X, color: green}
  "!": {icon: I, color: yellow}
list_style: type:unicode;size:small
custom_list:
  1: {icon: A, color: green}
  2: {icon: B, color: yellow}
"#;

fn custom_counts(config: &Config) -> [usize; 4] {
    [
        config.custom_callouts.len(),
        config.custom_code_blocks.len(),
        config.checkbox_overrides.len(),
        config.list_marker.overrides.len(),
    ]
}

#[test]
fn custom_settings_merge_entries_fields_and_color_origins() {
    let _environment = env_lock();
    let config = parse_with_structured_preset(
        BASE,
        "custom_callout: {note: {color: yellow}, preset: {icon: P}}\ncustom_code_block: {rust: {label: Preset}}\ncustom_checkbox: {x: {color: yellow}}\ncustom_list: {1: {color: yellow}}\n",
        &[
            "--custom-callout=note:color=red;cli:icon=L",
            "--custom-code-block=rust:icon=Q",
            "--custom-checkbox=x:C",
            "--custom-list=1:red",
        ],
    );
    assert_eq!(custom_counts(&config), [4, 2, 2, 2]);
    for (name, icon, color) in [
        ("note", "N", Some(Color::Red)),
        ("tip", "T", Some(Color::Yellow)),
        ("preset", "P", None),
        ("cli", "L", None),
    ] {
        let entry = &config.custom_callouts[name];
        assert_eq!(entry.icon.as_deref(), Some(icon), "{name}");
        assert_eq!(entry.color, color, "{name}");
    }
    let rust = &config.custom_code_blocks["rust"];
    assert_eq!(rust.icon.as_deref(), Some("Q"));
    assert_eq!(rust.label.as_deref(), Some("Preset"));
    assert_eq!(rust.aliases, ["rs", "old"]);
    assert_eq!(
        config.custom_code_blocks["python"].icon.as_deref(),
        Some("P")
    );
    assert_eq!(config.custom_code_default_icon.as_deref(), Some("D"));
    for (state, icon, color) in [('x', "C", Color::Yellow), ('!', "I", Color::Yellow)] {
        let entry = &config.checkbox_overrides[&state];
        assert_eq!(entry.icon.as_deref(), Some(icon), "{state}");
        assert_eq!(entry.color, Some(color), "{state}");
    }
    for (level, icon, color) in [(1, "A", Color::Red), (2, "B", Color::Yellow)] {
        assert_eq!(
            config.list_marker.resolve(level),
            Some((icon.to_string(), Some(color)))
        );
    }
    for (setting, name, source) in [
        ("custom_callout", "note", ColorSource::Cli),
        ("custom_callout", "tip", ColorSource::Config),
        ("custom_checkbox", "x", ColorSource::Preset),
        ("custom_checkbox", "!", ColorSource::Config),
        ("custom_list", "1", ColorSource::Cli),
        ("custom_list", "2", ColorSource::Config),
    ] {
        assert_eq!(config.color_priorities.custom_color(setting, name), source);
    }
}

#[test]
fn aliases_replace_or_clear_only_when_present() {
    let _environment = env_lock();
    for (field, cli, expected) in [
        ("aliases: [new]", "rust:label=CLI", vec!["new"]),
        ("label: Preset", "rust:aliases=rs2|rs3", vec!["rs2", "rs3"]),
        ("aliases: []", "rust:label=CLI", vec![]),
        ("label: Preset", "rust:aliases=[]", vec![]),
    ] {
        let preset = format!("custom_code_block: {{rust: {{{field}}}}}\n");
        let config = parse_with_structured_preset(BASE, &preset, &["--custom-code-block", cli]);
        let rust = &config.custom_code_blocks["rust"];
        assert_eq!(rust.icon.as_deref(), Some("R"));
        assert_eq!(rust.aliases, expected, "{field}: {cli}");
        assert!(config.custom_code_blocks.contains_key("python"));
    }
}

#[test]
fn explicit_clears_remove_properties_entries_or_whole_settings() {
    let _environment = env_lock();
    let preset = "custom_callout: {note: {icon: null}, tip: null}\ncustom_code_block: {rust: {label: null, aliases: []}}\ncustom_checkbox: {x: {color: null}}\ncustom_list: {1: {icon: null}}\n";
    let config = parse_with_structured_preset(BASE, preset, &[]);
    assert_eq!(custom_counts(&config), [1, 2, 2, 2]);
    let note = &config.custom_callouts["note"];
    assert!(note.icon.is_none());
    assert_eq!(note.color, Some(Color::Green));
    assert!(!config.custom_callouts.contains_key("tip"));
    let rust = &config.custom_code_blocks["rust"];
    assert_eq!(rust.icon.as_deref(), Some("R"));
    assert!(rust.label.is_none());
    assert!(rust.aliases.is_empty());
    let checked = &config.checkbox_overrides[&'x'];
    assert_eq!(checked.icon.as_deref(), Some("X"));
    assert!(checked.color.is_none());
    assert_eq!(
        config.list_marker.resolve(1),
        Some(("⦁".to_string(), Some(Color::Green)))
    );
    let config = parse_with_structured_preset(
        BASE,
        preset,
        &[
            "--custom-callout=note:color=null",
            "--custom-code-block=default:null;rust:icon=null",
            "--custom-checkbox=x:icon=null",
            "--custom-list=1:color=null;2:null",
        ],
    );
    assert_eq!(custom_counts(&config), [0, 1, 1, 0]);
    assert!(config.custom_code_default_icon.is_none());
    assert!(!config.custom_code_blocks.contains_key("rust"));
    assert!(config.custom_code_blocks.contains_key("python"));
    assert!(!config.checkbox_overrides.contains_key(&'x'));
    assert!(config.checkbox_overrides.contains_key(&'!'));
    for value in ["null", "{}"] {
        let preset = [
            "custom_callout",
            "custom_code_block",
            "custom_checkbox",
            "custom_list",
        ]
        .map(|key| format!("{key}: {value}\n"))
        .join("");
        let config = parse_with_structured_preset(BASE, &preset, &[]);
        assert_eq!(custom_counts(&config), [0; 4], "{value}");
        assert!(config.custom_code_default_icon.is_none());
    }
    let config = parse_with_structured_preset(
        BASE,
        "",
        &[
            "--custom-callout=null",
            "--custom-code-block=null",
            "--custom-checkbox=null",
            "--custom-list=null",
        ],
    );
    assert_eq!(custom_counts(&config), [0; 4]);
    assert!(config.custom_code_default_icon.is_none());
}
