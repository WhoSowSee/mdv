use super::*;

#[test]
fn cli_cols_override_terminal_width() {
    let _env_lock = env_lock();
    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("-c"),
        OsString::from("42"),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load config");
    assert_eq!(config.cols, Some(42));
    assert!(config.cols_from_cli);
    assert_eq!(config.get_terminal_width(), 42);
}

#[test]
fn no_config_flag_skips_loading_files() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "no_colors: true\n").expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--no-config"),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load config");
    assert!(
        !config.no_colors,
        "config file should be ignored when --no-config is set"
    );
}

#[test]
fn config_file_settings_survive_cli_defaults() {
    let _env_lock = env_lock();
    let config = parse_with_config(
        r#"
no_colors: true
wrap: word
table_wrap: wrap
tab_length: 2
heading_layout: flat
table_smart_indent: true
link_style: inline
link_truncation: cut
"#,
    );

    assert!(config.no_colors);
    assert!(matches!(config.wrap, TextWrapMode::Word));
    assert!(matches!(config.table_wrap, TableWrapMode::Wrap));
    assert_eq!(config.tab_length, 2);
    assert!(matches!(config.heading_layout, HeadingLayout::Flat));
    assert!(config.table_smart_indent);
    assert!(matches!(config.link_style, LinkStyle::Inline));
    assert!(matches!(config.link_truncation, LinkTruncationStyle::Cut));
}

#[test]
fn config_file_parses_tablecut_link_truncation() {
    let _env_lock = env_lock();
    let config = parse_with_config(
        r#"
link_style: inline
link_truncation: tablecut
"#,
    );

    assert!(matches!(config.link_style, LinkStyle::Inline));
    assert!(matches!(
        config.link_truncation,
        LinkTruncationStyle::TableCut
    ));
}

#[test]
fn config_rejects_legacy_pretty_list_boolean() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "pretty_list: true\n").expect("write config file");

    assert!(Config::load_from_file(&config_path).is_err());
}

#[test]
fn config_accepts_pretty_list_style_and_uniform_marker() {
    let _env_lock = env_lock();
    let config = parse_with_config(
        "pretty_list: \"type:unicode;size:small\"\nuniform_list_marker: \"level:3\"\n",
    );

    assert_eq!(config.list_marker.resolve(1).unwrap().0, "⚬");
}

#[test]
fn config_cols_from_file_does_not_mark_cli_override() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "cols: 70\n").expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load config");
    assert_eq!(config.cols, Some(70));
    assert!(!config.cols_from_cli);
}

#[test]
fn cli_arguments_override_config_when_provided() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "wrap: word\nlink_style: inline\n").expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--wrap"),
        OsString::from("none"),
        OsString::from("--link-style"),
        OsString::from("hide"),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load config with overrides");
    assert!(matches!(config.wrap, TextWrapMode::None));
    assert!(matches!(config.link_style, LinkStyle::Hide));
}

#[test]
fn cli_block_spacing_merges_each_side_over_config() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(
        &config_path,
        "block_spacing: \"paragraph:top=2,bottom=3\"\n",
    )
    .expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--block-spacing"),
        OsString::from("paragraph:top=0"),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load block spacing overrides");
    let paragraph = config
        .block_spacing
        .spacing(crate::block_spacing::BlockElement::Paragraph);
    assert_eq!((paragraph.top, paragraph.bottom), (0, 3));
}

#[test]
fn preset_overrides_config_and_cli_overrides_preset() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    std::fs::write(
        temp_dir.path().join("config.yaml"),
        "cols: 100\ntheme: monokai\nsmart_indent: true\npretty_table: false\ncode_theme: monokai\n",
    )
    .expect("write config file");
    write_preset(
        temp_dir.path(),
        "reader.yaml",
        "name: reader\ncols: 45\ntheme: terminal\nsmart_indent: false\npretty_table: true\ncode_theme: null\n",
    );

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--preset"),
        OsString::from("reader"),
        OsString::from("--cols"),
        OsString::from("60"),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load layered config");
    assert_eq!(config.cols, Some(60));
    assert_eq!(config.theme, "terminal");
    assert!(!config.smart_indent);
    assert!(config.pretty_table);
    assert!(config.code_theme.is_none());
}

#[test]
fn no_config_does_not_disable_user_presets() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    write_preset(
        temp_dir.path(),
        "custom.yml",
        "name: custom\ncols: 45\ntheme: nord\n",
    );

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
        OsString::from("--no-config"),
        OsString::from("--preset"),
        OsString::from("custom"),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load preset without config");
    assert_eq!(config.theme, "nord");
}

#[test]
fn empty_theme_from_cli_falls_back_to_default() {
    let _env_lock = env_lock();
    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--theme"),
        OsString::from(""),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load config with empty theme");
    assert_eq!(config.theme, "terminal");
}

#[test]
fn empty_theme_in_config_file_falls_back_to_default() {
    let _env_lock = env_lock();
    let config = parse_with_config("theme: \"\"\n");

    assert_eq!(config.theme, "terminal");
}

#[test]
fn empty_code_theme_input_clears_override() {
    let _env_lock = env_lock();
    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--code-theme"),
        OsString::from(""),
    ]);

    let config = Config::from_cli(&cli, &matches).expect("load config with empty code theme");
    assert!(config.code_theme.is_none());
}

#[test]
fn empty_code_theme_in_config_file_is_ignored() {
    let _env_lock = env_lock();
    let config = parse_with_config("code_theme: \"\"\n");

    assert!(config.code_theme.is_none());
}
