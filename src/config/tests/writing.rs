use super::*;

#[test]
fn default_config_template_matches_default_settings() {
    assert_eq!(
        DEFAULT_CONFIG_TEMPLATE,
        include_str!("../../../docs/examples/config.yaml")
    );
    let config: Config =
        serde_yaml::from_str(DEFAULT_CONFIG_TEMPLATE).expect("default config template parses");

    assert_eq!(config.theme, "terminal");
    assert!(config.code_theme.is_none());
    assert!(config.cols.is_none());
    assert_eq!(config.margin, HorizontalMargins::default());
    assert_eq!(
        serde_yaml::to_value(config.margin).expect("serialize default margin"),
        serde_yaml::Value::Null
    );
    assert!(!config.smart_indent);
    assert!(!config.render_html);
    assert!(config.line_numbers.is_none());
    assert!(matches!(config.link_style, LinkStyle::Clickable));
}

#[test]
fn write_default_config_uses_init_config_path() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--init-config"),
        temp_dir.path().as_os_str().to_owned(),
    ]);

    let written_path = Config::write_default_config(&cli, &matches).expect("write default config");
    let expected_path = temp_dir.path().join("config.yaml");

    assert_eq!(written_path, expected_path);
    assert!(expected_path.exists());
    assert_eq!(
        std::fs::read_to_string(&expected_path).expect("read generated config"),
        DEFAULT_CONFIG_TEMPLATE
    );
}

#[test]
fn write_default_config_uses_config_file_path() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--init-config"),
        OsString::from("--config-file"),
        temp_dir.path().join("nested").as_os_str().to_owned(),
    ]);

    let written_path = Config::write_default_config(&cli, &matches).expect("write default config");
    let expected_path = temp_dir.path().join("nested").join("config.yaml");

    assert_eq!(written_path, expected_path);
    assert!(expected_path.exists());
}

#[test]
fn write_default_config_uses_environment_config_path() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, temp_dir.path().as_os_str());
    let (cli, matches) =
        parse_cli_from(vec![OsString::from("mdv"), OsString::from("--init-config")]);

    let written_path =
        Config::write_default_config(&cli, &matches).expect("write default config from env");
    let expected_path = temp_dir.path().join("config.yaml");

    assert_eq!(written_path, expected_path);
    assert!(expected_path.exists());
}

#[test]
fn write_default_config_prefers_init_config_path_over_config_file_and_environment() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let env_path = temp_dir.path().join("env");
    let config_file_path = temp_dir.path().join("config-file");
    let init_path = temp_dir.path().join("init");
    let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, env_path.as_os_str());

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--init-config"),
        init_path.clone().into_os_string(),
        OsString::from("--config-file"),
        config_file_path.clone().into_os_string(),
    ]);

    let written_path = Config::write_default_config(&cli, &matches).expect("write default config");
    let expected_path = init_path.join("config.yaml");

    assert_eq!(written_path, expected_path);
    assert!(expected_path.exists());
    assert!(!config_file_path.join("config.yaml").exists());
    assert!(!env_path.join("config.yaml").exists());
}

#[test]
fn write_default_config_refuses_existing_file() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "theme: \"monokai\"\n").expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--init-config"),
        temp_dir.path().as_os_str().to_owned(),
    ]);

    let error =
        Config::write_default_config(&cli, &matches).expect_err("existing config must fail");
    assert!(error.to_string().contains("Config file already exists"));
    assert_eq!(
        std::fs::read_to_string(&config_path).expect("read existing config"),
        "theme: \"monokai\"\n"
    );
}
