use super::*;

#[test]
fn code_block_style_parses_name_and_icon_options() {
    let cli = Cli::parse_from(["mdv", "--code-block-style", "pretty:show-name;show-icon"]);
    let style = cli.code_block_style.expect("code block style parsed");
    assert!(matches!(style.style, CodeBlockStyle::Pretty));
    assert!(style.show_name);
    assert!(style.show_icon);
}

#[test]
fn code_block_style_defaults_to_basic_without_label() {
    let cli = Cli::parse_from(["mdv"]);
    let style = cli.code_block_style.expect("code block style parsed");
    assert!(matches!(style.style, CodeBlockStyle::Basic));
    assert!(!style.show_name);
    assert!(!style.show_icon);
}

#[test]
fn code_block_style_simple_without_label_parses() {
    let cli = Cli::parse_from(["mdv", "--code-block-style", "simple"]);
    let style = cli.code_block_style.expect("code block style parsed");
    assert!(matches!(style.style, CodeBlockStyle::Simple));
    assert!(!style.show_name);
    assert!(!style.show_icon);
}

#[test]
fn code_block_style_rejects_unknown_option() {
    let result = Cli::try_parse_from(["mdv", "--code-block-style", "pretty:bad-option"]);
    assert!(result.is_err());
}

#[test]
fn code_block_style_options_are_independent() {
    let cli = Cli::parse_from(["mdv", "--code-block-style", "basic:show-icon"]);
    let style = cli.code_block_style.expect("code block style parsed");
    assert!(matches!(style.style, CodeBlockStyle::Basic));
    assert!(!style.show_name);
    assert!(style.show_icon);

    let cli = Cli::parse_from(["mdv", "--code-block-style", "basic:show-name"]);
    let style = cli.code_block_style.expect("code block style parsed");
    assert!(style.show_name);
    assert!(!style.show_icon);
}

#[test]
fn code_block_style_rejects_removed_options() {
    for option in ["show-icons", "icon-only"] {
        let value = format!("simple:{}", option);
        let result = Cli::try_parse_from(["mdv", "--code-block-style", &value]);
        assert!(result.is_err(), "option should be rejected: {}", option);
    }
}

#[test]
fn callout_style_parses_simple_icons() {
    let cli = Cli::parse_from([
        "mdv",
        "--callout-style",
        "simple:show-simple-icons;uppercase",
    ]);
    let style = cli.style_callout.expect("callout style parsed");

    assert!(matches!(style.style, CalloutStyle::Simple));
    assert!(!style.show_icons);
    assert!(style.show_simple_icons);
    assert!(style.uppercase);
    assert_eq!(style.to_string(), "simple:show-simple-icons;uppercase");
}

#[test]
fn callout_style_rejects_multiple_icon_sets() {
    let result = Cli::try_parse_from([
        "mdv",
        "--callout-style",
        "simple:show-icons;show-simple-icons",
    ]);

    assert!(result.is_err());
}

#[test]
fn custom_code_block_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--custom-code-block", "rust:icon=;python:icon="]);
    assert_eq!(
        cli.custom_code_block.expect("custom code block parsed"),
        "rust:icon=;python:icon="
    );
}

#[test]
fn pretty_list_rejects_legacy_bare_flag() {
    assert!(Cli::try_parse_from(["mdv", "--pretty-list"]).is_err());
    assert!(Cli::try_parse_from(["mdv", "-L"]).is_err());
    assert!(Cli::try_parse_from(["mdv", "--pretty-list", "README.md"]).is_err());
}

#[test]
fn pretty_list_accepts_spaced_style_value() {
    let cli = Cli::parse_from([
        "mdv",
        "--pretty-list",
        "type:nerd-font;size:small",
        "README.md",
    ]);
    let style = cli.pretty_list.expect("pretty list style parsed");

    assert_eq!(style.to_string(), "type:nerd-font;size:small");
    assert_eq!(cli.filename.as_deref(), Some("README.md"));
}

#[test]
fn block_spacing_rejects_invalid_entries() {
    for value in [
        "unknown:top=1",
        "paragraph:left=1",
        "paragraph:top=-1",
        "paragraph:",
        "paragraph:top=1;paragraph:bottom=2",
        "paragraph:top=1,top=2",
    ] {
        assert!(
            Cli::try_parse_from(["mdv", "--block-spacing", value]).is_err(),
            "accepted invalid block spacing: {value}"
        );
    }
}
